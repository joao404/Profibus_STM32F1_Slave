#include "profibus/CProfibusSlave.h"

#define DEBUG


///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief Profibus Timer und Variablen initialisieren
 */
void CProfibusSlave::init_Profibus (uint8_t identHigh, uint8_t identLow, void (*func)(volatile uint8_t *outputbuf,volatile uint8_t *inputbuf), void (*printfunc)(uint8_t* buffer, uint8_t len))
{
  m_datafunc=func;

  if(NULL==printfunc)
  {
    configErrorLed();
    errorLedOn();
    return;
  }
  
  if(NULL==m_datafunc)
  {
    #ifdef DEBUG
    //m_printfunc("No Datafunc");
    #endif
    return;
  }

  m_printfunc = printfunc;

  

  m_identHigh = identHigh;
  m_identLow = identLow;
 
  // Variablen initialisieren
  stream_status = PROFIBUS_WAIT_SYN;
  slave_status = POR;
  diagnose_status_1 = STATION_NOT_READY_;
  //Input_Data_size = 0;
  //Output_Data_size = 0;
  User_Para_size = 0;
  Vendor_Data_size = 0;
  group = 0;
  
  // Slave Adresse holen
  slave_addr = 0x0B;//get_Address();

  // Keine ungueltigen Adresse zulassen
  if((slave_addr == 0) || (slave_addr > 126)) 
    slave_addr = DEFAULT_ADD;
  
  uint8_t cnt = 0;

  // Datenregister loeschen
  #if (OUTPUT_DATA_SIZE > 0)
  for (cnt = 0; cnt < OUTPUT_DATA_SIZE; cnt++)
  {
    output_register[cnt] = 0x00;
  }
  #endif
  #if (INPUT_DATA_SIZE > 0)
  for (cnt = 0; cnt < INPUT_DATA_SIZE; cnt++)
  {
    input_register[cnt] = 0xFF;
  }
  #endif
  #if (USER_PARA_SIZE > 0)
  for (cnt = 0; cnt < USER_PARA_SIZE; cnt++)
  {
    User_Para[cnt] = 0x00;
  }
  #endif
  #if (DIAG_DATA_SIZE > 0)
  for (cnt = 0; cnt < DIAG_DATA_SIZE; cnt++)
  {
    Diag_Data[cnt] = 0x00;
  }
  #endif

  watchdog_time=0xFFFFFF;
  last_connection_time=millis();

  // Timer init
  configTimer();
  setTimerCounter(0);
  setTimerMax(timeoutMaxSynTime);
  m_pbUartRxCnt = 0;
  m_pbUartTxCnt = 0;
  //LED Status
  configErrorLed();
  //Pin Init
  configRs485Pin();
  //RxRs485Enable();
  //Uart Init
  configUart();
  runTimer();
  activateRxInterrupt();
  //activateTxInterrupt();
  
  

 

  
  
#ifdef DEBUG
  //m_printfunc("Client configured with ");
  //m_printfunc("%d\n",slave_addr);
#endif

  // Interrupts freigeben
  //sei();

}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief ISR UART Receive
 */
void CProfibusSlave::interruptPbRx(void)
{
  
  // Erst mal Byte in Buffer sichern
  m_pbUartRxBuffer[m_pbUartRxCnt] = getUartValue();
   
  // Nur einlesen wenn TSYN abgelaufen
  if (PROFIBUS_WAIT_DATA == stream_status)
  {
    // TSYN abgelaufen, Daten einlesen
    stream_status = PROFIBUS_GET_DATA;
  }
  
  // Einlesen erlaubt?
  if (PROFIBUS_GET_DATA == stream_status)
  {
    m_pbUartRxCnt++;
    
    // Nicht mehr einlesen als in Buffer reinpasst 
    if(m_pbUartRxCnt == MAX_BUFFER_SIZE) m_pbUartRxCnt--;
  }

  
  // Profibus Timer ruecksetzen
  setTimerCounter(0);
  clearOverflowFlag();
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief Profibus auswertung
 */
void CProfibusSlave::profibus_RX (void)
{
  uint8_t cnt;
  uint8_t telegramm_type;
  uint8_t process_data;

  // Profibus Datentypen
  uint8_t destination_add;
  uint8_t source_add;
  uint8_t function_code;
  uint8_t FCS_data;   // Frame Check Sequence
  uint8_t PDU_size = 0;   // PDU Groesse
  uint8_t DSAP_data;  // SAP Destination
  uint8_t SSAP_data;  // SAP Source


  process_data = false;


  telegramm_type = m_pbUartRxBuffer[0];

  switch (telegramm_type)
  {
    case SD1: // Telegramm ohne Daten, max. 6 Byte

        if (m_pbUartRxCnt != 6) break;

        destination_add = m_pbUartRxBuffer[1];
        source_add      = m_pbUartRxBuffer[2];
        function_code   = m_pbUartRxBuffer[3];
        FCS_data        = m_pbUartRxBuffer[4];

        if (check_destination_addr(destination_add)       == false) break;
        if (calc_checksum(&m_pbUartRxBuffer[1], 3) != FCS_data) break;


        //FCV und FCB loeschen, da vorher überprüft
        function_code&=0xCF;
        process_data = true;

        break;

    case SD2: // Telegramm mit variabler Datenlaenge

        if (m_pbUartRxCnt != m_pbUartRxBuffer[1] + 6U) break;

        PDU_size        = m_pbUartRxBuffer[1]; // DA+SA+FC+Nutzdaten
        destination_add = m_pbUartRxBuffer[4];
        source_add      = m_pbUartRxBuffer[5];
        function_code   = m_pbUartRxBuffer[6];
        FCS_data        = m_pbUartRxBuffer[PDU_size + 4U];
        
        if (check_destination_addr(destination_add)              == false) break;
        if (calc_checksum(&m_pbUartRxBuffer[4], PDU_size) != FCS_data) 
        {
          //m_printfunc((int)calc_checksum(&pb_uart_buffer[4], PDU_size));
          break;
        }

        //FCV und FCB loeschen, da vorher überprüft
        function_code&=0xCF;
        process_data = true;

        break;

    case SD3: // Telegramm mit 5 Byte Daten, max. 11 Byte

        if (m_pbUartRxCnt != 11) break;

        PDU_size        = 8;              // DA+SA+FC+Nutzdaten
        destination_add = m_pbUartRxBuffer[1];
        source_add      = m_pbUartRxBuffer[2];
        function_code   = m_pbUartRxBuffer[3];
        FCS_data        = m_pbUartRxBuffer[9];

        if (check_destination_addr(destination_add)       == false) break;
        if (calc_checksum(&m_pbUartRxBuffer[1], 8) != FCS_data) break;


        //FCV und FCB loeschen, da vorher überprüft
        function_code&=0xCF;
        process_data = true;

        break;

    case SD4: // Token mit 3 Byte Daten
      
        if (m_pbUartRxCnt != 3) break;

        destination_add = m_pbUartRxBuffer[1];
        source_add      = m_pbUartRxBuffer[2];
      
        if (check_destination_addr(destination_add)       == false) break;
        
        break;
        
    default:

        break;

  } // Switch Ende



  
  // Nur auswerten wenn Daten OK sind
  if (process_data == true)
  {
    last_connection_time=millis();//letzte Zeit eines Telegramms sichern
    
    #ifdef DEBUG
      //m_printfunc("O");
    #endif
    master_addr = source_add; // Master Adresse ist Source Adresse


    if((function_code&0x30)==FCB_)//Startbedingung
    {
      fcv_act=true;
      fcb_last=true;
    }
    else if(true==fcv_act)
    {
      //Adresse wie vorher?
      if(source_add!=source_add_last)
      {
        //neue Verbindung und damit FCV ungültig
        fcv_act=false;
      }
      else if((function_code&FCB_)==fcb_last)//FCB ist gleich geblieben
      {
        //Nachricht wiederholen
        profibus_TX(&m_pbUartTxBuffer[0], m_pbUartTxCnt);
        //die Nachricht liegt noch im Speicher
      }
      else//Speichern des neuen FCB
      {
        fcb_last=!fcb_last;//das negierte bit speichern, da sonst die vorherige Bedingung angeschlagen hätte
      }
    }
    else//wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
    {
      fcv_act=false;
    }


    
      
    


    //letzte Adresse sichern
    source_add_last=source_add;
    
    // Service Access Point erkannt?
    if ((destination_add & 0x80) && (source_add & 0x80))
    {
      DSAP_data = m_pbUartRxBuffer[7];
      SSAP_data = m_pbUartRxBuffer[8];


      // Ablauf Reboot:
      // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
      // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
      // 3) SSAP 62 -> DSAP 62 (Check Config Request)
      // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
      // 5) Data Exchange Request (normaler Zyklus)
      
      // Siehe Felser 8/2009 Kap. 4.1
      //m_printfunc((int)DSAP_data);
      switch (DSAP_data)
      {
        case SAP_SET_SLAVE_ADR: // Set Slave Address (SSAP 62 -> DSAP 55)
        #ifdef DEBUG
            //m_printfunc("%d\n",SAP_SET_SLAVE_ADR);
        #endif
            // Siehe Felser 8/2009 Kap. 4.2

            // Nur im Zustand "Wait Parameter" (WPRM) moeglich

            if(WPRM == slave_status)
            {
                //adresse ändern
                //new_addr = pb_uart_buffer[9];
                //IDENT_HIGH_BYTE = m_pbUartRxBuffer[10];
                //IDENT_LOW_BYTE = m_pbUartRxBuffer[11];
                //if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
            }


            profibus_send_CMD(SC, 0, SAP_OFFSET, &m_pbUartTxBuffer[0], 0);

            break;

        case SAP_GLOBAL_CONTROL: // Global Control Request (SSAP 62 -> DSAP 58)
        #ifdef DEBUG
            //m_printfunc("%d\n",SAP_GLOBAL_CONTROL);
        #endif
            // Siehe Felser 8/2009 Kap. 4.6.2

            // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
            if (m_pbUartRxBuffer[9] & CLEAR_DATA_)
            {
              errorLedOn();  // Status "SPS nicht bereit"
            }
            else
            {
              errorLedOff(); // Status "SPS OK"
            }
          
            // Gruppe berechnen
            //for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;
            
            // Wenn Befehl fuer uns ist
            if ((m_pbUartRxBuffer[10]&group)!=0)//(cnt == group)
            {
              if (m_pbUartRxBuffer[9] & UNFREEZE_)
              {
                // FREEZE Zustand loeschen
                freeze=false;
                //m_datafunc(NULL,&(m_pbUartTxBuffer[7]));//outputs,inputs
              }
              else if (m_pbUartRxBuffer[9] & UNSYNC_)
              {
                // SYNC Zustand loeschen
                sync=false;
                m_datafunc(&(output_register[0]),NULL);//outputs,inputs
              }
              else if (m_pbUartRxBuffer[9] & FREEZE_)
              {
                // Eingaenge nicht mehr neu einlesen
                if(freeze)
                {
                  m_datafunc(NULL,&(input_register[0]));//outputs,inputs
                }
                freeze=true;
              }
              else if (m_pbUartRxBuffer[9] & SYNC_)
              {
                // Ausgaenge nur bei SYNC Befehl setzen
                
                if(sync)
                {
                  m_datafunc(&(output_register[0]),NULL);//outputs,inputs
                }
                sync=true;
              }
            }

            break;

        case SAP_SLAVE_DIAGNOSIS: // Get Diagnostics Request (SSAP 62 -> DSAP 60)
          #ifdef DEBUG
            //m_printfunc("%d\n",SAP_SLAVE_DIAGNOSIS);
          #endif
            // Siehe Felser 8/2009 Kap. 4.5.2

            // Nach dem Erhalt der Diagnose wechselt der DP-Slave vom Zustand
            // "Power on Reset" (POR) in den Zustand "Wait Parameter" (WPRM)

            // Am Ende der Initialisierung (Zustand "Data Exchange" (DXCHG))
            // sendet der Master ein zweites mal ein Diagnostics Request um die
            // korrekte Konfiguration zu pruefen
            //m_printfunc((int)function_code);
            //m_printfunc(REQUEST_ + SRD_HIGH);
            if ((function_code == (REQUEST_ + SRD_HIGH))||
                (function_code == (REQUEST_ + SRD_LOW )))
            {
              // Erste Diagnose Abfrage (Aufruf Telegramm)
              //pb_uart_buffer[4]  = master_addr;                  // Ziel Master (mit SAP Offset)
              //pb_uart_buffer[5]  = slave_addr + SAP_OFFSET;      // Quelle Slave (mit SAP Offset)
              //pb_uart_buffer[6]  = SLAVE_DATA;
              m_pbUartTxBuffer[7]  = SSAP_data;                    // Ziel SAP Master
              m_pbUartTxBuffer[8]  = DSAP_data;                    // Quelle SAP Slave
              m_pbUartTxBuffer[9]  = diagnose_status_1;            // Status 1
              if(POR == slave_status)
              {
                m_pbUartTxBuffer[10] = STATUS_2_DEFAULT + PRM_REQ_ + 0x04;  // Status 2
                m_pbUartTxBuffer[12] = MASTER_ADD_DEFAULT;           // Adresse Master
              }
              else
              {
                m_pbUartTxBuffer[10] = STATUS_2_DEFAULT + 0x04;             // Status 2
                m_pbUartTxBuffer[12] = master_addr - SAP_OFFSET;     // Adresse Master
              }

              if(watchdog_act)
              {
                m_pbUartTxBuffer[10] |= WD_ON_; 
              }

              if(freeze_act)
              {
                m_pbUartTxBuffer[10] |= FREEZE_MODE_; 
              }

              if(sync_act)
              {
                m_pbUartTxBuffer[10] |= SYNC_MODE_; 
              }
              
              m_pbUartTxBuffer[11] = DIAG_SIZE_OK;                 // Status 3
              m_pbUartTxBuffer[13] = m_identHigh;              // Ident high
              m_pbUartTxBuffer[14] = m_identLow;               // Ident low
              #if (EXT_DIAG_DATA_SIZE > 0)
              m_pbUartTxBuffer[15] = EXT_DIAG_GERAET+EXT_DIAG_DATA_SIZE+1; // Diagnose (Typ und Anzahl Bytes)
              for (cnt = 0; cnt < EXT_DIAG_DATA_SIZE; cnt++)
              {
                m_pbUartTxBuffer[16+cnt] = Diag_Data[cnt];
              }
              
              profibus_send_CMD(SD2, DATA_LOW, SAP_OFFSET, &m_pbUartTxBuffer[7], 9 + EXT_DIAG_DATA_SIZE);
              #else
              
              profibus_send_CMD(SD2, DATA_LOW, SAP_OFFSET, &m_pbUartTxBuffer[7], 8);
              #endif
              #ifdef DEBUG
              //m_printfunc("AD");
              #endif
              
            }

            //Status aendern
            if(POR == slave_status)
            {
              slave_status=WPRM;
              #ifdef DEBUG
                static uint8_t wprmStr[] = "WPRM\n";
                m_printfunc(wprmStr,5);
                //m_printfunc("WPRM");
              #endif
            }
            
            break;

        case SAP_SET_PRM: // Set Parameters Request (SSAP 62 -> DSAP 61)
          #ifdef DEBUG
            //m_printfunc("%d\n",SAP_SET_PRM);
          #endif
            // Siehe Felser 8/2009 Kap. 4.3.1

            // Nach dem Erhalt der Parameter wechselt der DP-Slave vom Zustand
            // "Wait Parameter" (WPRM) in den Zustand "Wait Configuration" (WCFG)
            //m_printfunc((int)pb_uart_buffer[13]);
            //m_printfunc(":");
            //m_printfunc((int)pb_uart_buffer[14]);
            // Nur Daten fuer unser Geraet akzeptieren
            //m_printfunc((int)pb_uart_buffer[13]);
            //m_printfunc((int)IDENT_HIGH_BYTE);
            //m_printfunc((int)pb_uart_buffer[14]);
            //m_printfunc((int)IDENT_LOW_BYTE);
            if ((m_pbUartRxBuffer[13] == m_identHigh) && (m_pbUartRxBuffer[14] == m_identLow))
            {
              //pb_uart_buffer[9]  = Befehl
              //pb_uart_buffer[10] = Watchdog 1
              //pb_uart_buffer[11] = Watchdog 2
              //pb_uart_buffer[12] = Min TSDR
              //pb_uart_buffer[13] = Ident H
              //pb_uart_buffer[14] = Ident L
              //pb_uart_buffer[15] = Gruppe
              //pb_uart_buffer[16] = User Parameter
              
              // Bei DPV1 Unterstuetzung:
              //pb_uart_buffer[16] = DPV1 Status 1 
              //pb_uart_buffer[17] = DPV1 Status 2
              //pb_uart_buffer[18] = DPV1 Status 3
              //pb_uart_buffer[19] = User Parameter

              if(!(m_pbUartRxBuffer[9]&ACTIVATE_WATCHDOG_))//Watchdog aktivieren
              {
                watchdog_act=true;
              }
              else
              {
                watchdog_act=false;
              }

              if(!(m_pbUartRxBuffer[9]&ACTIVATE_FREEZE_))
              {
                freeze_act=true;
              }
              else
              {
                freeze_act=false;
              }

              if(!(m_pbUartRxBuffer[9]&ACTIVATE_SYNC_))
              {
                sync_act=true;
              }
              else
              {
                sync_act=false;
              }

              //watchdog1 = m_pbUartRxBuffer[10];
              //watchdog2 = m_pbUartRxBuffer[11];

              watchdog_time=(unsigned long)m_pbUartRxBuffer[10]*(unsigned long)m_pbUartRxBuffer[11]*10;

              if(m_pbUartRxBuffer[12]>10)
              {
                minTSDR = m_pbUartRxBuffer[12]-11;
              }
              else
              {
                minTSDR=0;
              }

              m_identHigh = m_pbUartRxBuffer[13];
              m_identLow = m_pbUartRxBuffer[14];
              
              // User Parameter groe�e = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
              User_Para_size = PDU_size - 12;
              
              // User Parameter einlesen
              #if (USER_PARA_SIZE > 0)
              for (cnt = 0; cnt < User_Para_size; cnt++) User_Para[cnt] = m_pbUartRxBuffer[16+cnt];
              #endif
              
              // Gruppe einlesen
              //for (group = 0; pb_uart_buffer[15] != 0; group++) pb_uart_buffer[15]>>=1;

              group=m_pbUartRxBuffer[15];//wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

              // Kurzquittung 
              profibus_send_CMD(SC, 0, SAP_OFFSET, &m_pbUartTxBuffer[0], 0);
              //m_printfunc("Quittung");
              if(WPRM == slave_status)
              {
                slave_status=WCFG;
                #ifdef DEBUG
                //m_printfunc("WCFG");
                static uint8_t wcfgStr[] = "WCFG\n";
                //HAL_UART_Transmit_IT(&huart1, wcfgStr, 5);
                m_printfunc(wcfgStr, 5);
                #endif
              }
              
            }

            

            break;

        case SAP_CHK_CFG: // Check Config Request (SSAP 62 -> DSAP 62)
          #ifdef DEBUG
            //m_printfunc("%d\n",SAP_CHK_CFG);
          #endif
            // Siehe Felser 8/2009 Kap. 4.4.1

            // Nach dem Erhalt der Konfiguration wechselt der DP-Slave vom Zustand
            // "Wait Configuration" (WCFG) in den Zustand "Data Exchange" (DXCHG)

            // IO Konfiguration:
            // Kompaktes Format fuer max. 16/32 Byte IO
            // Spezielles Format fuer max. 64/132 Byte IO

            Module_cnt = 0;
            Vendor_Data_size = 0;
          
            // Je nach PDU Datengroesse mehrere Bytes auswerten
            // LE/LEr - (DA+SA+FC+DSAP+SSAP) = Anzahl Config Bytes
            for (cnt = 0; cnt < m_pbUartRxBuffer[1] - 5; cnt++)
            {
              switch (m_pbUartRxBuffer[9+cnt] & CFG_DIRECTION_)
              {
                case CFG_INPUT:
                    
                    //Input_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    //if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                    //  Input_Data_size = Input_Data_size*2;
  
                    Module_Data_size[Module_cnt][0] = (m_pbUartRxBuffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    if (m_pbUartRxBuffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                      Module_Data_size[Module_cnt][0] = Module_Data_size[Module_cnt][0]*2;
                      
                    Module_cnt++;
                    
                    break;
  
                case CFG_OUTPUT:
  
                    //Output_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    //if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                    //  Output_Data_size = Output_Data_size*2;
  
                    Module_Data_size[Module_cnt][1] = (m_pbUartRxBuffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    if (m_pbUartRxBuffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                      Module_Data_size[Module_cnt][1] = Module_Data_size[Module_cnt][1]*2;
                    
                    Module_cnt++;
                    
                    break;
  
                case CFG_INPUT_OUTPUT:
  
                    //Input_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    //Output_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    //if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                    //{
                    //  Input_Data_size = Input_Data_size*2;
                    //  Output_Data_size = Output_Data_size*2;
                    //}
  
                    Module_Data_size[Module_cnt][0] = (m_pbUartRxBuffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    Module_Data_size[Module_cnt][1] = (m_pbUartRxBuffer[9+cnt] & CFG_BYTE_CNT_) + 1;
                    if (m_pbUartRxBuffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
                    {
                      Module_Data_size[Module_cnt][0] = Module_Data_size[Module_cnt][0]*2;
                      Module_Data_size[Module_cnt][1] = Module_Data_size[Module_cnt][1]*2;
                    }
                    
                    Module_cnt++;
                    
                    break;
  
                case CFG_SPECIAL:
  
                    // Spezielles Format
                  
                    // Herstellerspezifische Bytes vorhanden?
                    if (m_pbUartRxBuffer[9+cnt] & CFG_SP_VENDOR_CNT_)
                    {
                      // Anzahl Herstellerdaten sichern
                      Vendor_Data_size += m_pbUartRxBuffer[9+cnt] & CFG_SP_VENDOR_CNT_;
                      
                      //Vendor_Data[] = pb_uart_buffer[];
                      
                      // Anzahl von Gesamtanzahl abziehen
                      m_pbUartRxBuffer[1] -= m_pbUartRxBuffer[9+cnt] & CFG_SP_VENDOR_CNT_;
                    }
                    
                    // I/O Daten
                    switch (m_pbUartRxBuffer[9+cnt] & CFG_SP_DIRECTION_)
                    {
                      case CFG_SP_VOID: // Leeres Modul (1 Byte)
                          
                          Module_Data_size[Module_cnt][0] = 0;
                          Module_Data_size[Module_cnt][1] = 0;
                            
                          Module_cnt++;
                        
                          break;
  
                      case CFG_SP_INPUT: // Eingang (2 Byte)
  
                          //Input_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          //if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                          //  Input_Data_size = Input_Data_size*2;
                          
                          Module_Data_size[Module_cnt][0] = (m_pbUartRxBuffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          if (m_pbUartRxBuffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                            Module_Data_size[Module_cnt][0] = Module_Data_size[Module_cnt][0]*2;
                          
                          Module_cnt++;
                          
                          cnt++;  // Zweites Byte haben wir jetzt schon
                          
                          break;
  
                      case CFG_SP_OUTPUT: // Ausgang (2 Byte)
  
                          //Output_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          //if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                          //  Output_Data_size = Output_Data_size*2;
  
                          Module_Data_size[Module_cnt][1] = (m_pbUartRxBuffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          if (m_pbUartRxBuffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                            Module_Data_size[Module_cnt][1] = Module_Data_size[Module_cnt][1]*2;
                          
                          Module_cnt++;
                          
                          cnt++;  // Zweites Byte haben wir jetzt schon
                          
                          break;
  
                      case CFG_SP_INPUT_OUTPUT: // Ein- und Ausgang (3 Byte)
  
                          // Erst Ausgang...
                          //Output_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          //if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                          //  Output_Data_size = Output_Data_size*2;
                          
                          // Dann Eingang
                          //Input_Data_size = (pb_uart_buffer[11+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          //if (pb_uart_buffer[11+cnt] & CFG_WIDTH_ & CFG_WORD)
                          //  Input_Data_size = Input_Data_size*2;
  
                          // Erst Ausgang...
                          Module_Data_size[Module_cnt][0] = (m_pbUartRxBuffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          if (m_pbUartRxBuffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
                            Module_Data_size[Module_cnt][0] = Module_Data_size[Module_cnt][0]*2;
                          
                          // Dann Eingang
                          Module_Data_size[Module_cnt][1] = (m_pbUartRxBuffer[11+cnt] & CFG_SP_BYTE_CNT_) + 1;
                          if (m_pbUartRxBuffer[11+cnt] & CFG_WIDTH_ & CFG_WORD)
                            Module_Data_size[Module_cnt][1] = Module_Data_size[Module_cnt][1]*2;
                          
                          Module_cnt++;
                          
                          cnt += 2;  // Zweites und drittes Bytes haben wir jetzt schon
                          
                          break;
  
                    } // Switch Ende
  
                    break;
  
                default:
  
                    //Input_Data_size = 0;
                    //Output_Data_size = 0;
                      
                    break;
  
              } // Switch Ende
            } // For Ende
            
            if (Vendor_Data_size != 0)
            {
              // Auswerten
            }
            
            
            // Bei Fehler -> CFG_FAULT_ in Diagnose senden
            #if (VENDOR_DATA_SIZE > 0)
            if (Module_cnt > MODULE_CNT || Vendor_Data_size != VENDOR_DATA_SIZE)
              diagnose_status_1 |= CFG_FAULT_;
            #else
            if (Module_cnt > MODULE_CNT)
              diagnose_status_1 |= CFG_FAULT_;
            #endif
            else
              diagnose_status_1 &= ~(STATION_NOT_READY_ + CFG_FAULT_);
            
            
            // Kurzquittung 
            profibus_send_CMD(SC, 0, SAP_OFFSET, &m_pbUartTxBuffer[0], 0);


            if(WCFG == slave_status)
            {
                slave_status=DXCHG;
                #ifdef DEBUG
                //m_printfunc("DXCHG");
                static uint8_t dxchgStr[] = "DXCHG\n";
                //HAL_UART_Transmit_IT(&huart1, dxchgStr, 6);
                m_printfunc(dxchgStr, 6);
                #endif
            }

            break;

        default:

            // Unbekannter SAP
          
            break;

      } // Switch DSAP_data Ende

    }
    // Ziel: Slave Adresse, but no SAP
    else if (destination_add == slave_addr)
    {

      // Status Abfrage
      if (function_code == (REQUEST_ + FDL_STATUS))
      {
        profibus_send_CMD(SD1, FDL_STATUS_OK, 0, &m_pbUartTxBuffer[0], 0);
      }
      // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
      /*
      else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) || 
               function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
      {
       */
       else if (function_code == (REQUEST_ + SRD_HIGH) || 
               function_code == (REQUEST_ +  SRD_LOW))
      {

        /*
        // Daten von Master einlesen
        #if (OUTPUT_DATA_SIZE > 0)
        for (cnt = 0; cnt < OUTPUT_DATA_SIZE; cnt++)
        {
          output_register[cnt] = pb_uart_buffer[cnt + 7];
        }
        #endif
        

        
        // Daten fuer Master in Buffer schreiben
        #if (INPUT_DATA_SIZE > 0)
        for (cnt = 0; cnt < INPUT_DATA_SIZE; cnt++)
        {
          pb_uart_buffer[cnt + 7] = input_register[cnt];
        }
        #endif
        */
        /*
        if((!sync)||(sync_act&&sync))//set outputs if no sync
        {
          m_datafunc(&(m_pbUartRxBuffer[7]),NULL);//outputs,inputs
        }
        if((!freeze)||(freeze_act&&freeze))//stops reading inputs if freeze= true
        {
          m_datafunc(NULL,&(m_pbUartTxBuffer[7]));//outputs,inputs
        }
        */
        
        if(sync_act && sync)//write data in output_register when sync
        {
          for (cnt = 0; cnt < OUTPUT_DATA_SIZE; cnt++)
          {
            output_register[cnt] = m_pbUartRxBuffer[cnt + 7];
          }
        }
        else//normaler Betrieb
        {
          m_datafunc(&(m_pbUartRxBuffer[7]),NULL);//outputs,inputs
        }
        
        
        if(freeze_act && freeze)//write input_register in telegram when freeze
        {
          for (cnt = 0; cnt < INPUT_DATA_SIZE; cnt++)
          {
            m_pbUartTxBuffer[cnt + 7] = input_register[cnt];
          }
        }
        else//normaler Betrieb
        {
          m_datafunc(NULL,&(m_pbUartTxBuffer[7]));//outputs,inputs
        }
        
        
        
        #if (INPUT_DATA_SIZE > 0)
        if (diagnose_status_1 & EXT_DIAG_)
          profibus_send_CMD(SD2, DATA_HIGH, 0, &m_pbUartTxBuffer[7], 0); // Diagnose Abfrage anfordern
        else
          profibus_send_CMD(SD2, DATA_LOW, 0, &m_pbUartTxBuffer[7], INPUT_DATA_SIZE);  // Daten senden
        #else
        if (diagnose_status_1 & EXT_DIAG_ || (get_Address() & 0x80))
          profibus_send_CMD(SD1, DATA_HIGH, 0, &m_pbUartTxBuffer[7], 0); // Diagnose Abfrage anfordern
        else        
          profibus_send_CMD(SC, 0, 0, &m_pbUartTxBuffer[7], 0);          // Kurzquittung 
        #endif
      }
    }
    
  } // Daten gueltig Ende
  else// Daten nicht gueltig
  {
    
    #ifdef DEBUG
      //m_printfunc("E\n");
      static uint8_t dxchgStr[10];
      snprintf((char*)dxchgStr, 10, "ERROR%lu\n", m_pbUartRxCnt);
      m_printfunc(dxchgStr, 10);
    #endif
    m_pbUartRxCnt = 0;
  }

}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief Profibus Telegramm zusammenstellen und senden
 * \param type          Telegrammtyp (SD1, SD2 usw.)
 * \param function_code Function Code der uebermittelt werden soll
 * \param sap_offset    Wert des SAP-Offset
 * \param pdu           Pointer auf Datenfeld (PDU)
 * \param length_pdu    Laenge der reinen PDU ohne DA, SA oder FC
 */
void CProfibusSlave::profibus_send_CMD (uint8_t type, 
                        uint8_t function_code, 
                        uint8_t sap_offset,
                        volatile uint8_t *pdu, 
                        uint8_t length_pdu)
{
  uint8_t length_data = 0;
  
    
  switch(type)
  {
    case SD1:

      m_pbUartTxBuffer[0] = SD1;
      m_pbUartTxBuffer[1] = master_addr;
      m_pbUartTxBuffer[2] = slave_addr + sap_offset;
      m_pbUartTxBuffer[3] = function_code;
      m_pbUartTxBuffer[4] = calc_checksum(&m_pbUartTxBuffer[1], 3);
      m_pbUartTxBuffer[5] = ED;

      length_data = 6;

      break;

    case SD2:

      m_pbUartTxBuffer[0] = SD2;
      m_pbUartTxBuffer[1] = length_pdu + 3;  // Laenge der PDU inkl. DA, SA und FC
      m_pbUartTxBuffer[2] = length_pdu + 3;
      m_pbUartTxBuffer[3] = SD2;
      m_pbUartTxBuffer[4] = master_addr;
      m_pbUartTxBuffer[5] = slave_addr + sap_offset;
      m_pbUartTxBuffer[6] = function_code;
      
      // Daten werden vor Aufruf der Funktion schon aufgefuellt

      m_pbUartTxBuffer[7+length_pdu] = calc_checksum(&m_pbUartTxBuffer[4], length_pdu + 3);
      m_pbUartTxBuffer[8+length_pdu] = ED;

      length_data = length_pdu + 9;

      break;

    case SD3:

      m_pbUartTxBuffer[0] = SD3;
      m_pbUartTxBuffer[1] = master_addr;
      m_pbUartTxBuffer[2] = slave_addr + sap_offset;
      m_pbUartTxBuffer[3] = function_code;

      // Daten werden vor Aufruf der Funktion schon aufgefuellt

      m_pbUartTxBuffer[9] = calc_checksum(&m_pbUartTxBuffer[4], 8);
      m_pbUartTxBuffer[10] = ED;

      length_data = 11;

      break;

    case SD4:

      m_pbUartTxBuffer[0] = SD4;
      m_pbUartTxBuffer[1] = master_addr;
      m_pbUartTxBuffer[2] = slave_addr + sap_offset;

      length_data = 3;

      break;

    case SC:

      m_pbUartTxBuffer[0] = SC;

      length_data = 1;

      break;

    default:

      break;

  }
  
  profibus_TX(&m_pbUartTxBuffer[0], length_data);

}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief Telegramm senden
 * \param data    Pointer auf Datenfeld
 * \param length  Laenge der Daten
 */
void CProfibusSlave::profibus_TX (volatile uint8_t *data, uint8_t datalength)
{
// Mit Interrupt
  //m_printfunc(datalength);
  
  m_pbUartTxCnt = datalength;         // Anzahl zu sendender Bytes
  pb_tx_cnt = 0;          // Zahler fuer gesendete Bytes


  if(0 != minTSDR)
  {
    stream_status = PROFIBUS_WAIT_MINTSDR;
    setTimerMax(minTSDR*bitTimeINcycle/2);
  }
  else
  {
    setTimerMax(timeoutMaxTxTime); 
    stream_status = PROFIBUS_SEND_DATA;
    //activate Send Interrupt
    waitForActivTransmission();
    TxRs485Enable();
    activateTxInterrupt();
    setUartValue(m_pbUartTxBuffer[pb_tx_cnt]); 
    pb_tx_cnt++;
  }
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief calc_checksumme berechnen. Einfaches addieren aller Datenbytes im Telegramm.
 * \param data    Pointer auf Datenfeld
 * \param length  Laenge der Daten
 * \return calc_checksumme
 */
uint8_t CProfibusSlave::calc_checksum(volatile uint8_t *data, uint8_t length)
{
  uint8_t csum = 0;

  while(length--)
  {
    csum += data[length];
  }

  return csum;
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief Zieladresse ueberpruefen. Adresse muss mit Slave Adresse oder Broadcast (inkl. SAP Offset)
          uebereinstimmen
 * \param destination Zieladresse
 * \return true wenn Zieladresse unsere ist, false wenn nicht
 */
uint8_t CProfibusSlave::check_destination_addr (uint8_t destination)
{
  if (((destination&0x7F) != slave_addr) &&                // Slave
      ((destination&0x7F) != BROADCAST_ADD))             // Broadcast
    return false;
  
  return true;
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief ISR UART Transmit
 */
void CProfibusSlave::interruptPbTx(void)
{

  // Alles gesendet?
  if (pb_tx_cnt < m_pbUartTxCnt) 
  {
    // TX Buffer fuellen
    setUartValue(m_pbUartTxBuffer[pb_tx_cnt++]);
    //m_printfunc(pb_tx_cnt);
  }
  else
  {
    TxRs485Disable();
    // Alles gesendet, Interrupt wieder aus
    deactivateTxInterrupt();
    //clear Flag because we are not writing to buffer
    clearTxFlag();
    //m_printfunc("E");
    #ifdef DEBUG
    //m_printfunc("a");
    static uint8_t sendStr = 'a';
    m_printfunc(&sendStr, 1);
    #endif
  }
  
  setTimerCounter(0);
  clearOverflowFlag();
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
/*!
 * \brief ISR TIMER
 */
void CProfibusSlave::interruptTimer(void)
{
  
  
  // Timer A Stop  
  stopTimer();
  setTimerCounter(0);
  
  switch (stream_status)
  {
    case PROFIBUS_WAIT_SYN: // TSYN abgelaufen
      
        stream_status = PROFIBUS_WAIT_DATA;        
        m_pbUartRxCnt = 0;
        RxRs485Enable();          // Auf Receive umschalten   
        //activateRxInterrupt();
        setTimerMax(timeoutMaxSdrTime);
        // activateRxInterrupt();
        //RS485_RX_EN          // Auf Receive umschalten  
        break;
        
    case PROFIBUS_WAIT_DATA:  // TSDR abgelaufen aber keine Daten da
        //ACITVATE_RX_INTERRUPT
        //RS485_RX_EN          // Auf Receive umschalten
        break;
        
    case PROFIBUS_GET_DATA:   // TSDR abgelaufen und Daten da

        //m_printfunc(stream_status);
        stream_status = PROFIBUS_WAIT_SYN;
        setTimerMax(timeoutMaxSynTime);
          

        /*
        for(uint8_t i=0;i<m_pbUartRxCnt;i++)
        {
          m_printfunc("%u:",m_pbUartRxBuffer[i]);
        }
        m_printfunc("\n");
        */
        
        
        deactivateRxInterrupt();
        #ifdef DEBUG
        //m_printfunc("%u\n",m_pbUartRxCnt);
        #endif
        profibus_RX();
        activateRxInterrupt();
        
        
        break;
    case PROFIBUS_WAIT_MINTSDR:

        //TIMER_MAX=minTSDR*TIME_BIT;
        setTimerMax(timeoutMaxTxTime); 
        stream_status = PROFIBUS_SEND_DATA;
        //activate Send Interrupt
        waitForActivTransmission();
        TxRs485Enable();
        activateTxInterrupt();
        setUartValue(m_pbUartTxBuffer[pb_tx_cnt]); 
        pb_tx_cnt++;
        
        break;    
    case PROFIBUS_SEND_DATA:  // Sende-Timeout abgelaufen, wieder auf Empfang gehen

        stream_status = PROFIBUS_WAIT_SYN;
        setTimerMax(timeoutMaxSynTime);
        
        RxRs485Enable();          // Auf Receive umschalten   
        
        break;
    
    default:
      break;
    
  }
  
  
  

  if(watchdog_act)
  {
    if((millis()-last_connection_time)>watchdog_time)
    {
      for (int cnt = 0; cnt < OUTPUT_DATA_SIZE; cnt++)
      {
        output_register[cnt] = 0;//sicherer Zustand
      }
      m_datafunc(&(output_register[0]),NULL);//outputs,inputs
    }
  }
  // Timer A STIMER_COUNTERT
  runTimer();
}