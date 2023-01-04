use heapless::Vec;

pub trait HwInterface {
    fn config_timer(&mut self) {}

    fn run_timer(&mut self) {}

    fn stop_timer(&mut self) {}

    fn set_timer_counter(&mut self, _value: u16) {}

    fn set_timer_max(&mut self, _value: u16) {}

    fn clear_overflow_flag(&mut self) {}

    fn config_uart(&mut self) {}

    fn activate_tx_interrupt(&mut self) {}

    fn deactivate_tx_interrupt(&mut self) {}

    fn activate_rx_interrupt(&mut self) {}

    fn deactivate_rx_interrupt(&mut self) {}

    fn set_tx_flag(&mut self) {}

    fn clear_tx_flag(&mut self) {}

    fn clear_rx_flag(&mut self) {}

    fn wait_for_activ_transmission(&mut self) {}

    fn rx_data_received(&mut self) -> bool {
        false
    }

    fn tx_data_send(&mut self) -> bool {
        false
    }

    fn tx_rs485_enable(&mut self) {}

    fn tx_rs485_disable(&mut self) {}

    fn rx_rs485_enable(&mut self) {}

    fn config_rs485_pin(&mut self) {}

    fn get_uart_value(&mut self) -> Result<u8> {
        Ok(0)
    }

    fn set_uart_value(&mut self, _value: u8) {}

    fn config_error_led(&mut self) {}

    fn error_led_on(&mut self) {}

    fn error_led_off(&mut self) {}

    fn millis(&mut self) -> u32 {
        0
    }
}

#[derive(PartialEq, Eq)]
enum DpSlaveState {
    Por = 1,    // Power on reset
    Wrpm = 2,   // Wait for parameter
    Wcfg = 3,   // Wait for config
    Ddxchg = 4, // Data exchange
}

#[derive(PartialEq, Eq)]
enum StreamState {
    WaitSyn,
    WaitData,
    GetData,
    WaitMinTsdr,
    SendData,
}

#[derive(PartialEq, Eq)]
enum CmdType {
    SD1 = 0x10, // Telegramm ohne Datenfeld
    SD2 = 0x68, // Daten Telegramm variabel
    SD3 = 0xA2, // Daten Telegramm fest
    SD4 = 0xDC, // Token
    SC = 0xE5,  // Kurzquittung
    ED = 0x16,  // Ende
}

pub mod FcRequest {
    pub const FDL_STATUS: u8 = 0x09; // SPS: Status Abfrage
    pub const SDR_LOW: u8 = 0x0C; // SPS: Ausgaenge setzen, Eingaenge lesen
    pub const SDR_HIGH: u8 = 0x0D; // SPS: Ausgaenge setzen, Eingaenge lesen
    pub const FCV: u8 = 0x10;
    pub const FCB: u8 = 0x20;
    pub const REQUEST: u8 = 0x40;
}

pub mod FcResponse {
    pub const FDL_STATUS_OK: u8 = 0x00; // SLA: OK
    pub const DATA_LOW: u8 = 0x08; // SLA: (Data low) Daten Eingaenge senden
    pub const DATA_HIGH: u8 = 0x0A; // SLA: (Data high) Diagnose anstehend
}

#[derive(PartialEq, Eq)]
enum SAP {
    SetSlaveAdr = 55,     // Master setzt Slave Adresse, Slave Anwortet mit SC
    RdInp = 56,           // Master fordert Input Daten, Slave sendet Input Daten
    RdOutp = 57,          // Master fordert Output Daten, Slave sendet Output Daten
    GlobalControl = 58,   // Master Control, Slave Antwortet nicht
    GetCfg = 59,          // Master fordert Konfig., Slave sendet Konfiguration
    SlaveDiagnostic = 60, // Master fordert Diagnose, Slave sendet Diagnose Daten
    SetPrm = 61,          // Master sendet Parameter, Slave sendet SC
    ChkCfg = 62,          // Master sendet Konfuguration, Slave sendet SC
}

pub mod SapDiagnoseByte1 {
    pub const STATUS_1_DEFAULT: u8 = 0x00;
    pub const STATION_NOT_EXISTENT: u8 = 0x01;
    pub const STATION_NOT_READY: u8 = 0x02;
    pub const CFG_FAULT: u8 = 0x04;
    pub const EXT_DIAG: u8 = 0x08; // Erweiterte Diagnose vorhanden
    pub const NOT_SUPPORTED: u8 = 0x10;
    pub const INV_SLAVE_RESPONSE: u8 = 0x20;
    pub const PRM_FAULT: u8 = 0x40;
    pub const MASTER_LOCK: u8 = 0x80;
}

pub mod SapDiagnoseByte2 {
    pub const STATUS_2_DEFAULT: u8 = 0x04;
    pub const PRM_REQ: u8 = 0x01;
    pub const STAT_DIAG: u8 = 0x02;
    pub const WD_ON: u8 = 0x08;
    pub const FREEZE_MODE: u8 = 0x10;
    pub const SYNC_MODE: u8 = 0x20;
    //pub const  free                  0x40
    pub const DEACTIVATED: u8 = 0x80;
}

pub mod SapDiagnoseByte3 {
    pub const DIAG_SIZE_OK: u8 = 0x00;
    pub const DIAG_SIZE_ERROR: u8 = 0x80;
}

pub mod SapDiagnoseExt {
    pub const EXT_DIAG_TYPE: u8 = 0xC0; // Bit 6-7 ist Diagnose Typ
    pub const EXT_DIAG_BYTE_CNT: u8 = 0x3F; // Bit 0-5 sind Anzahl der Diagnose Bytes

    pub const EXT_DIAG_GERAET: u8 = 0x00; // Wenn Bit 7 und 6 = 00, dann Geraetebezogen
    pub const EXT_DIAG_KENNUNG: u8 = 0x40; // Wenn Bit 7 und 6 = 01, dann Kennungsbezogen
    pub const EXT_DIAG_KANAL: u8 = 0x80; // Wenn Bit 7 und 6 = 10, dann Kanalbezogen
}

pub mod SapSetparameterRequest {
    pub const LOCK_SLAVE: u8 = 0x80; // Slave fuer andere Master gesperrt
    pub const UNLOCK_SLAVE: u8 = 0x40; // Slave fuer andere Master freigegeben
    pub const ACTIVATE_SYNC: u8 = 0x20;
    pub const ACTIVATE_FREEZE: u8 = 0x10;
    pub const ACTIVATE_WATCHDOG: u8 = 0x08;
}

pub mod Dpv1StatusByte1 {
    pub const DPV1_MODE: u8 = 0x80;
    pub const FAIL_SAVE_MODE: u8 = 0x40;
    pub const PUBLISHER_MODE: u8 = 0x20;
    pub const WATCHDOG_TB_1MS: u8 = 0x04;
}

pub mod Dpv1StatusByte2 {
    pub const PULL_PLUG_ALARM: u8 = 0x80;
    pub const PROZESS_ALARM: u8 = 0x40;
    pub const DIAGNOSE_ALARM: u8 = 0x20;
    pub const VENDOR_ALARM: u8 = 0x10;
    pub const STATUS_ALARM: u8 = 0x08;
    pub const UPDATE_ALARM: u8 = 0x04;
    pub const CHECK_CONFIG_MODE: u8 = 0x01;
}

pub mod Dpv1StatusByte3 {
    pub const PARAMETER_CMD_ON: u8 = 0x80;
    pub const ISOCHRON_MODE_ON: u8 = 0x10;
    pub const PARAMETER_BLOCK: u8 = 0x08;
}

pub mod SapCheckConfigRequest {
    pub const CFG_DIRECTION: u8 = 0x30; // Bit 4-5 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
    pub const CFG_INPUT: u8 = 0x10; // Eingang
    pub const CFG_OUTPUT: u8 = 0x20; // Ausgang
    pub const CFG_INPUT_OUTPUT: u8 = 0x30; // Eingang/Ausgang
    pub const CFG_SPECIAL: u8 = 0x00; // Spezielles Format wenn mehr als 16/32Byte uebertragen werden sollen

    pub const CFG_KONSISTENZ: u8 = 0x80; // Bit 7 ist Konsistenz. 0 = Byte oder Wort, 1 = Ueber gesamtes Modul
    pub const CFG_KONS_BYTE_WORT: u8 = 0x00; // Byte oder Wort
    pub const CFG_KONS_MODUL: u8 = 0x80; // Modul

    pub const CFG_WIDTH: u8 = 0x40; // Bit 6 ist IO Breite. 0 = Byte (8bit), 1 = Wort (16bit)
    pub const CFG_BYTE: u8 = 0x00; // Byte
    pub const CFG_WORD: u8 = 0x40; // Wort

    /* Kompaktes Format */
    pub const CFG_BYTE_CNT: u8 = 0x0F; // Bit 0-3 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.

    /* Spezielles Format */
    pub const CFG_SP_DIRECTION: u8 = 0xC0; // Bit 6-7 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
    pub const CFG_SP_VOID: u8 = 0x00; // Leerplatz
    pub const CFG_SP_INPUT: u8 = 0x40; // Eingang
    pub const CFG_SP_OUTPUT: u8 = 0x80; // Ausgang
    pub const CFG_SP_INPUT_OUTPUT: u8 = 0xC0; // Eingang/Ausgang

    pub const CFG_SP_VENDOR_CNT: u8 = 0x0F; // Bit 0-3 sind Anzahl der herstellerspezifischen Bytes. 0 = keine

    /* Spezielles Format / Laengenbyte */
    pub const CFG_SP_BYTE_CNT: u8 = 0x3F; // Bit 0-5 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.
}

pub struct Config {
    ident_high: u8,
    ident_low: u8,
    addr: u8,
    counter_frequency: u32,
    baudrate: u32,
    transmit_buffer: &mut Vec<u8, _>,
    buf_size: u8,
    input_data_size: u8,
    output_data_size: u8,
    module_count: u8,
    user_para_size: u8,
    extern_diag_para_size: u8,
    vendor_data_size: u8,
}

impl Config {
    pub fn ident_high(mut self, ident_high: u8) -> Self {
        self.ident_high = ident_high;
        self
    }
    pub fn ident_low(mut self, ident_low: u8) -> Self {
        self.ident_low = ident_low;
        self
    }
    pub fn addr(mut self, addr: u8) -> Self {
        self.addr = addr;
        self
    }
    pub fn counter_frequency(mut self, counter_frequency: u32) -> Self {
        self.counter_frequency = counter_frequency;
        self
    }
    pub fn baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = baudrate;
        self
    }

    pub fn transmit_buffer(mut self, transmit_buffer: &mut Vec<u8, _>) -> Self {
        self.transmit_buffer = transmit_buffer;
        self
    }

    pub fn buf_size(mut self, buf_size: u8) -> Self {
        self.buf_size = buf_size;
        self
    }
    pub fn input_data_size(mut self, input_data_size: u8) -> Self {
        self.input_data_size = input_data_size;
        self
    }
    pub fn output_data_size(mut self, output_data_size: u8) -> Self {
        self.output_data_size = output_data_size;
        self
    }
    pub fn module_count(mut self, module_count: u8) -> Self {
        self.module_count = module_count;
        self
    }
    pub fn user_para_size(mut self, user_para_size: u8) -> Self {
        self.user_para_size = user_para_size;
        self
    }
    pub fn extern_diag_para_size(mut self, extern_diag_para_size: u8) -> Self {
        self.extern_diag_para_size = extern_diag_para_size;
        self
    }
    pub fn vendor_data_size(mut self, vendor_data_size: u8) -> Self {
        self.vendor_data_size = vendor_data_size;
        self
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            ident_high: 0,
            ident_low: 0,
            addr: 126,
            counter_frequency: 0,
            baudrate: 500_000_u32,
            transmit_buffer: &mut Vec::<u8, 1>::new(),
            buf_size: 0,
            input_data_size: 0,
            output_data_size: 0,
            module_count: 0,
            user_para_size: 0,
            extern_diag_para_size: 0,
            vendor_data_size: 0,
        }
    }
}

pub struct PbDpSlave<T> {
    config: Config,
    interface: T,
    buffer: Vec<u8, 255>,
    slave_state: DpSlaveState,
    stream_state: StreamState,
    bit_time_in_cycle: u32,
    timeout_max_syn_time: u32,
    timeout_max_rx_time: u32,
    timeout_max_tx_time: u32,
    timeout_max_sdr_time: u32,
    diagnose_status_1: u8,
    master_addr: u8,
    group: u8,

    source_add_last: u8,
    fcv_act: bool,
    fcb_last: bool,

    freeze: bool,
    sync: bool,
    watchdog_act: bool,
    min_tdsr: u8,

    freeze_act: bool,
    sync_act: bool,

    last_connection_time: u32,
    watchdog_time: u32,
}

impl<T> PbDpSlave<T>
where
    T: HwInterface,
{
    pub fn new(config: Config, interface: T) -> Self {
        m_datafunc = func;

        //     if (nullptr == printfunc)
        //     {
        //         interface.config_error_led();
        //         interface.error_led_on();
        //       return;
        //     }

        //     if (nullptr == m_datafunc)
        //     {
        //   #ifdef DEBUG
        //       m_printfunc("No Datafunc\n");
        //   #endif
        //       return;
        //     }

        // m_printfunc = printfunc;

        // m_config = config;

        //   #ifdef DEBUG
        //     m_printfunc("%u\n", m_config.counterFrequency);
        //   #endif

        //     if (0 == m_config.counterFrequency)
        //     {
        //       return;
        //     }

        if 0 == config.counter_frequency {
            config.counter_frequency = 56_000_000_u32;
        }

        if 0 == config.baudrate {
            config.baudrate = 500_000_u32;
        }

        let bit_time_in_cycle = config.counter_frequency / config.baudrate;
        let timeout_max_syn_time = 33 * bit_time_in_cycle; // 33 TBit = TSYN
        let timeout_max_rx_time = 15 * bit_time_in_cycle;
        let timeout_max_tx_time = 15 * bit_time_in_cycle;
        let timeout_max_sdr_time = 15 * bit_time_in_cycle; // 15 Tbit = TSDR

        if 0 == config.buf_size {
            config.buf_size = 1;
        } else if (config.buf_size > 255) {
            config.buf_size = 255;
        }

        diagnose_status_1 = STATION_NOT_READY_;
        // Input_Data_size = 0;
        // Output_Data_size = 0;
        User_Para_size = 0;
        Vendor_Data_size = 0;

        if (0 == addr) || (addr > 126) {
            addr = 126;
        }

        // Datenregister loeschen
        m_outputReg.resize(m_config.outputDataSize);
        m_inputReg.resize(m_config.inputDataSize);
        m_userPara.resize(m_config.userParaSize);
        m_diagData.resize(m_config.externDiagParaSize);
        m_VendorData.resize(m_config.vendorDataSize);

        // Timer init
        interface.config_timer();
        interface.set_timer_counter(0);
        interface.set_timer_max(timeout_max_syn_time);
        rx_buf_cnt = 0;
        tx_buf_cnt = 0;
        // LED Status
        interface.config_error_led();
        // Pin Init
        interface.config_rs485_pin();

        // Uart Init
        interface.config_uart();
        interface.run_timer();
        interface.activate_rx_interrupt();
        // activateTxInterrupt();
        interface.tx_rs485_enable();

        Self {
            config,
            interface,
            buffer: Vec::<u8, 255>::new(),
            slave_state: DpSlaveState::Por,
            stream_state: StreamState::WaitSyn,
            bit_time_in_cycle,
            timeout_max_syn_time,
            timeout_max_rx_time,
            timeout_max_tx_time,
            timeout_max_sdr_time,
            diagnose_status_1: SapDiagnoseByte1::STATUS_1_DEFAULT,
            master_addr: 0xFF,
            group: 0,
            source_add_last: 0xFF,
            fcv_act: false,
            fcb_last: false,
            freeze: false,
            sync: false,
            watchdog_act: false,
            min_tdsr: 0,
            freeze_act: false,
            sync_act: false,
            last_connection_time: interface.millis(),
            watchdog_time: 0xFFFFFF,
        }
    }

    pub fn get_interface(&self) -> &dyn HwInterface {
        &self.interface
    }

    pub fn interrupt_handler(&self) {
        if self.interface.rx_data_received() {
            loop {
                match self.interface.get_uart_value() {
                    Ok(b) => {
                        self.handle_rx(b);
                    }
                    Err(_err) => break,
                }
            }
        } else if self.interface.tx_data_send() {

        }
    }

    pub fn handle_rx(&mut self, data: u8) {
        self.buffer.push(data).unwrap();

        // if we waited for TSYN, data can be saved
        if StreamState::WaitData == self.stream_state {
            self.stream_state = StreamState::GetData;
        }

        // Einlesen erlaubt?
        if StreamState::GetData == self.stream_state {
            // m_rxBufCnt++;

            // Nicht mehr einlesen als in Buffer reinpasst
            // if (m_rxBufCnt >= m_config.bufSize)
            //   m_rxBufCnt--;
        }

        // Profibus Timer ruecksetzen
        self.interface.set_timer_counter(0);
        self.interface.clear_overflow_flag();
    }

    pub fn handle_tx(&mut self) {
      // Alles gesendet?
  if (m_txCnt < m_txBufCnt)
  {
    // TX Buffer fuellen
    self.interface.set_uart_value(m_txBuffer[m_txCnt++]);
    // m_printfunc(m_txCnt);
  }
  else
  {
    self.interface.tx_rs485_disable();
    // Alles gesendet, Interrupt wieder aus
    self.interface.deactivate_tx_interrupt();
    // clear Flag because we are not writing to buffer
    self.interface.clear_tx_flag();
  }

  setTimerCounter(0);
  clearOverflowFlag();
}

    pub fn handle_message_timeout(&mut self) {
        let _test = self.buffer.len();

        self.buffer.clear();
    }
}

// void CProfibusSlave::rxFunc(void)
// {
//   uint8_t cnt;
//   uint8_t process_data;

//   // Profibus Datentypen
//   uint8_t destination_add;
//   uint8_t source_add;
//   uint8_t function_code;
//   uint8_t FCS_data;     // Frame Check Sequence
//   uint8_t PDU_size = 0; // PDU Groesse
//   uint8_t DSAP_data;    // SAP Destination
//   uint8_t SSAP_data;    // SAP Source

//   process_data = false;

//   switch (m_rxBuffer[0])
//   {
//   case static_cast<uint8_t>(CmdType::SD1): // Telegramm ohne Daten, max. 6 Byte

//     if (m_rxBufCnt != 6)
//       break;

//     destination_add = m_rxBuffer[1];
//     source_add = m_rxBuffer[2];
//     function_code = m_rxBuffer[3];
//     FCS_data = m_rxBuffer[4];

//     if (checkDestinationAdr(destination_add) == false)
//       break;
//     if (calcChecksum(&m_rxBuffer[1], 3) != FCS_data)
//       break;

//     // FCV und FCB loeschen, da vorher überprüft
//     function_code &= 0xCF;
//     process_data = true;

//     break;

//   case static_cast<uint8_t>(CmdType::SD2): // Telegramm mit variabler Datenlaenge

//     if (m_rxBufCnt != m_rxBuffer[1] + 6U)
//       break;

//     PDU_size = m_rxBuffer[1]; // DA+SA+FC+Nutzdaten
//     destination_add = m_rxBuffer[4];
//     source_add = m_rxBuffer[5];
//     function_code = m_rxBuffer[6];
//     FCS_data = m_rxBuffer[PDU_size + 4U];

//     if (checkDestinationAdr(destination_add) == false)
//       break;
//     if (calcChecksum(&m_rxBuffer[4], PDU_size) != FCS_data)
//     {
//       // m_printfunc((int)calc_checksum(&pb_uart_buffer[4], PDU_size));
//       break;
//     }

//     // FCV und FCB loeschen, da vorher überprüft
//     function_code &= 0xCF;
//     process_data = true;

//     break;

//   case static_cast<uint8_t>(CmdType::SD3): // Telegramm mit 5 Byte Daten, max. 11 Byte

//     if (m_rxBufCnt != 11)
//       break;

//     PDU_size = 8; // DA+SA+FC+Nutzdaten
//     destination_add = m_rxBuffer[1];
//     source_add = m_rxBuffer[2];
//     function_code = m_rxBuffer[3];
//     FCS_data = m_rxBuffer[9];

//     if (checkDestinationAdr(destination_add) == false)
//       break;
//     if (calcChecksum(&m_rxBuffer[1], 8) != FCS_data)
//       break;

//     // FCV und FCB loeschen, da vorher überprüft
//     function_code &= 0xCF;
//     process_data = true;

//     break;

//   case static_cast<uint8_t>(CmdType::SD4): // Token mit 3 Byte Daten

//     if (m_rxBufCnt != 3)
//       break;

//     destination_add = m_rxBuffer[1];
//     source_add = m_rxBuffer[2];

//     if (checkDestinationAdr(destination_add) == false)
//       break;

//     break;

//   default:

//     break;

//   } // Switch Ende

//   // Nur auswerten wenn Daten OK sind
//   if (process_data == true)
//   {
//     last_connection_time = millis(); // letzte Zeit eines Telegramms sichern

// #ifdef DEBUG
//                                      // m_printfunc("O");
// #endif
//     master_addr = source_add; // Master Adresse ist Source Adresse

//     if ((function_code & 0x30) == FCB_) // Startbedingung
//     {
//       fcv_act = true;
//       fcb_last = true;
//     }
//     else if (true == fcv_act)
//     {
//       // Adresse wie vorher?
//       if (source_add != source_add_last)
//       {
//         // neue Verbindung und damit FCV ungültig
//         fcv_act = false;
//       }
//       else if ((function_code & FCB_) == fcb_last) // FCB ist gleich geblieben
//       {
//         // Nachricht wiederholen
//         txFunc(&m_txBuffer[0], m_txBufCnt);
//         // die Nachricht liegt noch im Speicher
//       }
//       else // Speichern des neuen FCB
//       {
//         fcb_last = !fcb_last; // das negierte bit speichern, da sonst die vorherige Bedingung angeschlagen hätte
//       }
//     }
//     else // wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
//     {
//       fcv_act = false;
//     }

//     // letzte Adresse sichern
//     source_add_last = source_add;

//     // Service Access Point erkannt?
//     if ((destination_add & 0x80) && (source_add & 0x80))
//     {
//       DSAP_data = m_rxBuffer[7];
//       SSAP_data = m_rxBuffer[8];

//       // Ablauf Reboot:
//       // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
//       // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
//       // 3) SSAP 62 -> DSAP 62 (Check Config Request)
//       // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
//       // 5) Data Exchange Request (normaler Zyklus)

//       // Siehe Felser 8/2009 Kap. 4.1
//       // m_printfunc((int)DSAP_data);
//       switch (DSAP_data)
//       {
//       case SAP_SET_SLAVE_ADR: // Set Slave Address (SSAP 62 -> DSAP 55)
// #ifdef DEBUG
//                               // m_printfunc("%d\n",SAP_SET_SLAVE_ADR);
// #endif
//                               // Siehe Felser 8/2009 Kap. 4.2

//         // Nur im Zustand "Wait Parameter" (WPRM) moeglich

//         if (DpSlaveState::WPRM == slave_status)
//         {
//           // adresse ändern
//           // new_addr = pb_uart_buffer[9];
//           // IDENT_HIGH_BYTE = m_pbUartRxBuffer[10];
//           // IDENT_LOW_BYTE = m_pbUartRxBuffer[11];
//           // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
//         }

//         sendCmd(CmdType::SC, 0, SAP_OFFSET, &m_txBuffer[0], 0);

//         break;

//       case SAP_GLOBAL_CONTROL: // Global Control Request (SSAP 62 -> DSAP 58)
// #ifdef DEBUG
//                                // m_printfunc("%d\n",SAP_GLOBAL_CONTROL);
// #endif
//                                // Siehe Felser 8/2009 Kap. 4.6.2

//         // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
//         if (m_rxBuffer[9] & CLEAR_DATA_)
//         {
//           errorLedOn(); // Status "SPS nicht bereit"
//         }
//         else
//         {
//           errorLedOff(); // Status "SPS OK"
//         }

//         // Gruppe berechnen
//         // for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;

//         // Wenn Befehl fuer uns ist
//         if ((m_rxBuffer[10] & group) != 0) //(cnt == group)
//         {
//           if (m_rxBuffer[9] & UNFREEZE_)
//           {
//             // FREEZE Zustand loeschen
//             freeze = false;
//             // m_datafunc(NULL,&(m_txBuffer[7]));//outputs,inputs
//           }
//           else if (m_rxBuffer[9] & UNSYNC_)
//           {
//             // SYNC Zustand loeschen
//             sync = false;
//             std::vector<uint8_t> inputDelete;
//             m_datafunc(m_outputReg, inputDelete); // outputs,inputs
//           }
//           else if (m_rxBuffer[9] & FREEZE_)
//           {
//             // Eingaenge nicht mehr neu einlesen
//             if (freeze)
//             {
//               std::vector<uint8_t> outputFreeze;
//               m_datafunc(outputFreeze, m_inputReg); // outputs,inputs
//             }
//             freeze = true;
//           }
//           else if (m_rxBuffer[9] & SYNC_)
//           {
//             // Ausgaenge nur bei SYNC Befehl setzen

//             if (sync)
//             {
//               std::vector<uint8_t> inputNotUsed;
//               m_datafunc(m_outputReg, inputNotUsed); // outputs,inputs
//             }
//             sync = true;
//           }
//         }

//         break;

//       case SAP_SLAVE_DIAGNOSIS: // Get Diagnostics Request (SSAP 62 -> DSAP 60)
// #ifdef DEBUG
//                                 // m_printfunc("%d\n",SAP_SLAVE_DIAGNOSIS);
// #endif
//                                 // Siehe Felser 8/2009 Kap. 4.5.2

//         // Nach dem Erhalt der Diagnose wechselt der DP-Slave vom Zustand
//         // "Power on Reset" (POR) in den Zustand "Wait Parameter" (WPRM)

//         // Am Ende der Initialisierung (Zustand "Data Exchange" (DXCHG))
//         // sendet der Master ein zweites mal ein Diagnostics Request um die
//         // korrekte Konfiguration zu pruefen
//         // m_printfunc((int)function_code);
//         // m_printfunc(REQUEST_ + SRD_HIGH);
//         if ((function_code == (REQUEST_ + SRD_HIGH)) ||
//             (function_code == (REQUEST_ + SRD_LOW)))
//         {
//           // Erste Diagnose Abfrage (Aufruf Telegramm)
//           // pb_uart_buffer[4]  = master_addr;                  // Ziel Master (mit SAP Offset)
//           // pb_uart_buffer[5]  = slave_addr + SAP_OFFSET;      // Quelle Slave (mit SAP Offset)
//           // pb_uart_buffer[6]  = SLAVE_DATA;
//           m_txBuffer[7] = SSAP_data;         // Ziel SAP Master
//           m_txBuffer[8] = DSAP_data;         // Quelle SAP Slave
//           m_txBuffer[9] = diagnose_status_1; // Status 1
//           if (DpSlaveState::POR == slave_status)
//           {
//             m_txBuffer[10] = STATUS_2_DEFAULT + PRM_REQ_ + 0x04; // Status 2
//             m_txBuffer[12] = MASTER_ADD_DEFAULT;                 // Adresse Master
//           }
//           else
//           {
//             m_txBuffer[10] = STATUS_2_DEFAULT + 0x04;  // Status 2
//             m_txBuffer[12] = master_addr - SAP_OFFSET; // Adresse Master
//           }

//           if (watchdog_act)
//           {
//             m_txBuffer[10] |= WD_ON_;
//           }

//           if (freeze_act)
//           {
//             m_txBuffer[10] |= FREEZE_MODE_;
//           }

//           if (sync_act)
//           {
//             m_txBuffer[10] |= SYNC_MODE_;
//           }

//           m_txBuffer[11] = DIAG_SIZE_OK;       // Status 3
//           m_txBuffer[13] = m_config.identHigh; // Ident high
//           m_txBuffer[14] = m_config.identLow;  // Ident low
//           if (m_diagData.size() > 0)
//           {
//             m_txBuffer[15] = EXT_DIAG_GERAET + m_diagData.size() + 1; // Diagnose (Typ und Anzahl Bytes)
//             for (cnt = 0; cnt < m_diagData.size(); cnt++)
//             {
//               m_txBuffer[16 + cnt] = m_diagData[cnt];
//             }

//             sendCmd(CmdType::SD2, DATA_LOW, SAP_OFFSET, &m_txBuffer[7], 9 + m_diagData.size());
//           }
//           else
//           {

//             sendCmd(CmdType::SD2, DATA_LOW, SAP_OFFSET, &m_txBuffer[7], 8);
//           }
// #ifdef DEBUG
// // m_printfunc("AD");
// #endif
//         }

//         // Status aendern
//         if (DpSlaveState::POR == slave_status)
//         {
//           slave_status = DpSlaveState::WPRM;
// #ifdef DEBUG
//           m_printfunc("WPRM\n");
// #endif
//         }

//         break;

//       case SAP_SET_PRM: // Set Parameters Request (SSAP 62 -> DSAP 61)
// #ifdef DEBUG
//                         // m_printfunc("%d\n",SAP_SET_PRM);
// #endif
//                         // Siehe Felser 8/2009 Kap. 4.3.1

//         // Nach dem Erhalt der Parameter wechselt der DP-Slave vom Zustand
//         // "Wait Parameter" (WPRM) in den Zustand "Wait Configuration" (WCFG)
//         // m_printfunc((int)pb_uart_buffer[13]);
//         // m_printfunc(":");
//         // m_printfunc((int)pb_uart_buffer[14]);
//         // Nur Daten fuer unser Geraet akzeptieren
//         // m_printfunc((int)pb_uart_buffer[13]);
//         // m_printfunc((int)IDENT_HIGH_BYTE);
//         // m_printfunc((int)pb_uart_buffer[14]);
//         // m_printfunc((int)IDENT_LOW_BYTE);
//         if ((m_rxBuffer[13] == m_config.identHigh) && (m_rxBuffer[14] == m_config.identLow))
//         {
//           // pb_uart_buffer[9]  = Befehl
//           // pb_uart_buffer[10] = Watchdog 1
//           // pb_uart_buffer[11] = Watchdog 2
//           // pb_uart_buffer[12] = Min TSDR
//           // pb_uart_buffer[13] = Ident H
//           // pb_uart_buffer[14] = Ident L
//           // pb_uart_buffer[15] = Gruppe
//           // pb_uart_buffer[16] = User Parameter

//           // Bei DPV1 Unterstuetzung:
//           // pb_uart_buffer[16] = DPV1 Status 1
//           // pb_uart_buffer[17] = DPV1 Status 2
//           // pb_uart_buffer[18] = DPV1 Status 3
//           // pb_uart_buffer[19] = User Parameter

//           if (!(m_rxBuffer[9] & ACTIVATE_WATCHDOG_)) // Watchdog aktivieren
//           {
//             watchdog_act = true;
//           }
//           else
//           {
//             watchdog_act = false;
//           }

//           if (!(m_rxBuffer[9] & ACTIVATE_FREEZE_))
//           {
//             freeze_act = true;
//           }
//           else
//           {
//             freeze_act = false;
//           }

//           if (!(m_rxBuffer[9] & ACTIVATE_SYNC_))
//           {
//             sync_act = true;
//           }
//           else
//           {
//             sync_act = false;
//           }

//           // watchdog1 = m_pbUartRxBuffer[10];
//           // watchdog2 = m_pbUartRxBuffer[11];

//           watchdog_time = (unsigned long)m_rxBuffer[10] * (unsigned long)m_rxBuffer[11] * 10;

//           if (m_rxBuffer[12] > 10)
//           {
//             minTSDR = m_rxBuffer[12] - 11;
//           }
//           else
//           {
//             minTSDR = 0;
//           }

//           m_config.identHigh = m_rxBuffer[13];
//           m_config.identLow = m_rxBuffer[14];

//           // User Parameter groe�e = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
//           User_Para_size = PDU_size - 12;

//           // User Parameter einlesen
//           if (m_userPara.size() > 0)
//           {
//             for (cnt = 0; cnt < m_userPara.size(); cnt++)
//               m_userPara[cnt] = m_rxBuffer[16 + cnt];
//           }

//           // Gruppe einlesen
//           // for (group = 0; pb_uart_buffer[15] != 0; group++) pb_uart_buffer[15]>>=1;

//           group = m_rxBuffer[15]; // wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

//           // Kurzquittung
//           sendCmd(CmdType::SC, 0, SAP_OFFSET, &m_txBuffer[0], 0);
//           // m_printfunc("Quittung");
//           if (DpSlaveState::WPRM == slave_status)
//           {
//             slave_status = DpSlaveState::WCFG;
// #ifdef DEBUG
//             m_printfunc("WCFG\n");
// #endif
//           }
//         }

//         break;

//       case SAP_CHK_CFG: // Check Config Request (SSAP 62 -> DSAP 62)
// #ifdef DEBUG
//                         // m_printfunc("%d\n",SAP_CHK_CFG);
// #endif
//                         // Siehe Felser 8/2009 Kap. 4.4.1

//         // Nach dem Erhalt der Konfiguration wechselt der DP-Slave vom Zustand
//         // "Wait Configuration" (WCFG) in den Zustand "Data Exchange" (DXCHG)

//         // IO Konfiguration:
//         // Kompaktes Format fuer max. 16/32 Byte IO
//         // Spezielles Format fuer max. 64/132 Byte IO

//         Module_cnt = 0;
//         Vendor_Data_size = 0;

//         // Je nach PDU Datengroesse mehrere Bytes auswerten
//         // LE/LEr - (DA+SA+FC+DSAP+SSAP) = Anzahl Config Bytes
//         for (cnt = 0; cnt < m_rxBuffer[1] - 5; cnt++)
//         {
//           switch (m_rxBuffer[9 + cnt] & CFG_DIRECTION_)
//           {
//           case CFG_INPUT:

//             // Input_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
//             // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
//             //   Input_Data_size = Input_Data_size*2;

//             m_moduleData[Module_cnt][0] = (m_rxBuffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
//             if (m_rxBuffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
//               m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

//             Module_cnt++;

//             break;

//           case CFG_OUTPUT:

//             // Output_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
//             // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
//             //   Output_Data_size = Output_Data_size*2;

//             m_moduleData[Module_cnt][1] = (m_rxBuffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
//             if (m_rxBuffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
//               m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

//             Module_cnt++;

//             break;

//           case CFG_INPUT_OUTPUT:

//             // Input_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
//             // Output_Data_size = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
//             // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
//             //{
//             //   Input_Data_size = Input_Data_size*2;
//             //   Output_Data_size = Output_Data_size*2;
//             // }

//             m_moduleData[Module_cnt][0] = (m_rxBuffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
//             m_moduleData[Module_cnt][1] = (m_rxBuffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
//             if (m_rxBuffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
//             {
//               m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;
//               m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;
//             }

//             Module_cnt++;

//             break;

//           case CFG_SPECIAL:

//             // Spezielles Format

//             // Herstellerspezifische Bytes vorhanden?
//             if (m_rxBuffer[9 + cnt] & CFG_SP_VENDOR_CNT_)
//             {
//               // Anzahl Herstellerdaten sichern
//               Vendor_Data_size += m_rxBuffer[9 + cnt] & CFG_SP_VENDOR_CNT_;

//               // Vendor_Data[] = pb_uart_buffer[];

//               // Anzahl von Gesamtanzahl abziehen
//               m_rxBuffer[1] -= m_rxBuffer[9 + cnt] & CFG_SP_VENDOR_CNT_;
//             }

//             // I/O Daten
//             switch (m_rxBuffer[9 + cnt] & CFG_SP_DIRECTION_)
//             {
//             case CFG_SP_VOID: // Leeres Modul (1 Byte)

//               m_moduleData[Module_cnt][0] = 0;
//               m_moduleData[Module_cnt][1] = 0;

//               Module_cnt++;

//               break;

//             case CFG_SP_INPUT: // Eingang (2 Byte)

//               // Input_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
//               // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
//               //   Input_Data_size = Input_Data_size*2;

//               m_moduleData[Module_cnt][0] = (m_rxBuffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
//               if (m_rxBuffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
//                 m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

//               Module_cnt++;

//               cnt++; // Zweites Byte haben wir jetzt schon

//               break;

//             case CFG_SP_OUTPUT: // Ausgang (2 Byte)

//               // Output_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
//               // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
//               //   Output_Data_size = Output_Data_size*2;

//               m_moduleData[Module_cnt][1] = (m_rxBuffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
//               if (m_rxBuffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
//                 m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

//               Module_cnt++;

//               cnt++; // Zweites Byte haben wir jetzt schon

//               break;

//             case CFG_SP_INPUT_OUTPUT: // Ein- und Ausgang (3 Byte)

//               // Erst Ausgang...
//               // Output_Data_size = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
//               // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
//               //  Output_Data_size = Output_Data_size*2;

//               // Dann Eingang
//               // Input_Data_size = (pb_uart_buffer[11+cnt] & CFG_SP_BYTE_CNT_) + 1;
//               // if (pb_uart_buffer[11+cnt] & CFG_WIDTH_ & CFG_WORD)
//               //  Input_Data_size = Input_Data_size*2;

//               // Erst Ausgang...
//               m_moduleData[Module_cnt][0] = (m_rxBuffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
//               if (m_rxBuffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
//                 m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

//               // Dann Eingang
//               m_moduleData[Module_cnt][1] = (m_rxBuffer[11 + cnt] & CFG_SP_BYTE_CNT_) + 1;
//               if (m_rxBuffer[11 + cnt] & CFG_WIDTH_ & CFG_WORD)
//                 m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

//               Module_cnt++;

//               cnt += 2; // Zweites und drittes Bytes haben wir jetzt schon

//               break;

//             } // Switch Ende

//             break;

//           default:

//             // Input_Data_size = 0;
//             // Output_Data_size = 0;

//             break;

//           } // Switch Ende
//         }   // For Ende

//         if (Vendor_Data_size != 0)
//         {
//           // Auswerten
//         }

//         // Bei Fehler -> CFG_FAULT_ in Diagnose senden
//         if ((m_VendorData.size() > 0) && (Module_cnt > m_moduleData.size() || Vendor_Data_size != m_VendorData.size()))
//           diagnose_status_1 |= CFG_FAULT_;
//         else if ((m_VendorData.size() == 0) && (Module_cnt > m_config.moduleCount))
//           diagnose_status_1 |= CFG_FAULT_;
//         else
//           diagnose_status_1 &= ~(STATION_NOT_READY_ + CFG_FAULT_);

//         // Kurzquittung
//         sendCmd(CmdType::SC, 0, SAP_OFFSET, &m_txBuffer[0], 0);

//         if (DpSlaveState::WCFG == slave_status)
//         {
//           slave_status = DpSlaveState::DXCHG;
// #ifdef DEBUG
//           m_printfunc("DXCHG\n");
// #endif
//         }

//         break;

//       default:

//         // Unbekannter SAP

//         break;

//       } // Switch DSAP_data Ende
//     }
//     // Ziel: Slave Adresse, but no SAP
//     else if (destination_add == slave_addr)
//     {

//       // Status Abfrage
//       if (function_code == (REQUEST_ + FDL_STATUS))
//       {
//         sendCmd(CmdType::SD1, FDL_STATUS_OK, 0, &m_txBuffer[0], 0);
//       }
//       // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
//       /*
//       else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) ||
//                function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
//       {
//        */
//       else if (function_code == (REQUEST_ + SRD_HIGH) ||
//                function_code == (REQUEST_ + SRD_LOW))
//       {

//         /*
//         // Daten von Master einlesen
//         #if (OUTPUT_DATA_SIZE > 0)
//         for (cnt = 0; cnt < OUTPUT_DATA_SIZE; cnt++)
//         {
//           output_register[cnt] = pb_uart_buffer[cnt + 7];
//         }
//         #endif

//         // Daten fuer Master in Buffer schreiben
//         #if (INPUT_DATA_SIZE > 0)
//         for (cnt = 0; cnt < INPUT_DATA_SIZE; cnt++)
//         {
//           pb_uart_buffer[cnt + 7] = input_register[cnt];
//         }
//         #endif
//         */
//         /*
//         if((!sync)||(sync_act&&sync))//set outputs if no sync
//         {
//           m_datafunc(&(m_rxBuffer[7]),NULL);//outputs,inputs
//         }
//         if((!freeze)||(freeze_act&&freeze))//stops reading inputs if freeze= true
//         {
//           m_datafunc(NULL,&(m_pbUartTxBuffer[7]));//outputs,inputs
//         }
//         */
//         if (sync_act && sync) // write data in output_register when sync
//         {
//           for (cnt = 0; cnt < m_outputReg.size(); cnt++)
//           {
//             m_outputReg[cnt] = m_rxBuffer[cnt + 7];
//           }
//         }
//         else // normaler Betrieb
//         {
//           for (cnt = 0; cnt < m_outputReg.size(); cnt++)
//           {
//             m_outputReg[cnt] = m_rxBuffer[cnt + 7];
//           }
//           std::vector<uint8_t> unUsed;
//           m_datafunc(m_outputReg, unUsed); // outputs,inputs
//         }

//         if (freeze_act && freeze) // write input_register in telegram when freeze
//         {
//           for (cnt = 0; cnt < m_inputReg.size(); cnt++)
//           {
//             m_txBuffer[cnt + 7] = m_inputReg[cnt];
//           }
//         }
//         else // normaler Betrieb
//         {
//           std::vector<uint8_t> unUsed;
//           m_datafunc(unUsed, m_inputReg); // outputs,inputs
//           for (cnt = 0; cnt < m_inputReg.size(); cnt++)
//           {
//             m_txBuffer[cnt + 7] = m_inputReg[cnt];
//           }
//         }

//         if (m_inputReg.size() > 0)
//         {
//           if (diagnose_status_1 & EXT_DIAG_)
//             sendCmd(CmdType::SD2, DATA_HIGH, 0, &m_txBuffer[7], 0); // Diagnose Abfrage anfordern
//           else
//             sendCmd(CmdType::SD2, DATA_LOW, 0, &m_txBuffer[7], m_inputReg.size()); // Daten senden
//         }
//         else
//         {
//           // TODO
//           //  if (diagnose_status_1 & EXT_DIAG_ || (get_Address() & 0x80))
//           //    sendCmd(CmdType::SD1, DATA_HIGH, 0, &m_txBuffer[7], 0); // Diagnose Abfrage anfordern
//           //  else
//           //    sendCmd(CmdType::SC, 0, 0, &m_txBuffer[7], 0); // Kurzquittung
//         }
//       }
//     }

//   }    // Daten gueltig Ende
//   else // Daten nicht gueltig
//   {

// #ifdef DEBUG
//     m_printfunc("ERROR%lu\n", m_rxBufCnt);
// #endif
//     m_rxBufCnt = 0;
//   }
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief Profibus Telegramm zusammenstellen und senden
//  * \param type          Telegrammtyp (SD1, SD2 usw.)
//  * \param function_code Function Code der uebermittelt werden soll
//  * \param sap_offset    Wert des SAP-Offset
//  * \param pdu           Pointer auf Datenfeld (PDU)
//  * \param length_pdu    Laenge der reinen PDU ohne DA, SA oder FC
//  */
// void CProfibusSlave::sendCmd(CmdType type,
//                              uint8_t function_code,
//                              uint8_t sap_offset,
//                              volatile uint8_t *pdu,
//                              uint8_t length_pdu)
// {
//   uint8_t length_data = 0;

//   switch (type)
//   {
//   case CmdType::SD1:

//     m_txBuffer[0] = static_cast<uint8_t>(CmdType::SD1);
//     m_txBuffer[1] = master_addr;
//     m_txBuffer[2] = slave_addr + sap_offset;
//     m_txBuffer[3] = function_code;
//     m_txBuffer[4] = calcChecksum(&m_txBuffer[1], 3);
//     m_txBuffer[5] = static_cast<uint8_t>(CmdType::ED);

//     length_data = 6;

//     break;

//   case CmdType::SD2:

//     m_txBuffer[0] = static_cast<uint8_t>(CmdType::SD2);
//     m_txBuffer[1] = length_pdu + 3; // Laenge der PDU inkl. DA, SA und FC
//     m_txBuffer[2] = length_pdu + 3;
//     m_txBuffer[3] = static_cast<uint8_t>(CmdType::SD2);
//     m_txBuffer[4] = master_addr;
//     m_txBuffer[5] = slave_addr + sap_offset;
//     m_txBuffer[6] = function_code;

//     // Daten werden vor Aufruf der Funktion schon aufgefuellt

//     m_txBuffer[7 + length_pdu] = calcChecksum(&m_txBuffer[4], length_pdu + 3);
//     m_txBuffer[8 + length_pdu] = static_cast<uint8_t>(CmdType::ED);

//     length_data = length_pdu + 9;

//     break;

//   case CmdType::SD3:

//     m_txBuffer[0] = static_cast<uint8_t>(CmdType::SD3);
//     m_txBuffer[1] = master_addr;
//     m_txBuffer[2] = slave_addr + sap_offset;
//     m_txBuffer[3] = function_code;

//     // Daten werden vor Aufruf der Funktion schon aufgefuellt

//     m_txBuffer[9] = calcChecksum(&m_txBuffer[4], 8);
//     m_txBuffer[10] = static_cast<uint8_t>(CmdType::ED);

//     length_data = 11;

//     break;

//   case CmdType::SD4:

//     m_txBuffer[0] = static_cast<uint8_t>(CmdType::SD4);
//     m_txBuffer[1] = master_addr;
//     m_txBuffer[2] = slave_addr + sap_offset;

//     length_data = 3;

//     break;

//   case CmdType::SC:

//     m_txBuffer[0] = static_cast<uint8_t>(CmdType::SC);

//     length_data = 1;

//     break;

//   default:

//     break;
//   }

//   txFunc(&m_txBuffer[0], length_data);
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief Telegramm senden
//  * \param data    Pointer auf Datenfeld
//  * \param length  Laenge der Daten
//  */
// void CProfibusSlave::txFunc(volatile uint8_t *data, uint8_t datalength)
// {
//   // Mit Interrupt
//   // m_printfunc(datalength);

//   m_txBufCnt = datalength; // Anzahl zu sendender Bytes
//   m_txCnt = 0;             // Zahler fuer gesendete Bytes

//   if (0 != minTSDR)
//   {
//     stream_status = StreamStatus::WaitMinTsdr;
//     setTimerMax(minTSDR * m_bitTimeINcycle / 2);
//   }
//   else
//   {
//     setTimerMax(m_timeoutMaxTxTime);
//     stream_status = StreamStatus::SendData;
//     // activate Send Interrupt
//     waitForActivTransmission();
//     TxRs485Enable();
//     activateTxInterrupt();
//     setUartValue(m_txBuffer[m_txCnt]);
//     m_txCnt++;
//   }
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief calc_checksumme berechnen. Einfaches addieren aller Datenbytes im Telegramm.
//  * \param data    Pointer auf Datenfeld
//  * \param length  Laenge der Daten
//  * \return calc_checksumme
//  */
// uint8_t CProfibusSlave::calcChecksum(volatile uint8_t *data, uint8_t length)
// {
//   uint8_t csum = 0;

//   while (length--)
//   {
//     csum += data[length];
//   }

//   return csum;
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief Zieladresse ueberpruefen. Adresse muss mit Slave Adresse oder Broadcast (inkl. SAP Offset)
//           uebereinstimmen
//  * \param destination Zieladresse
//  * \return true wenn Zieladresse unsere ist, false wenn nicht
//  */
// uint8_t CProfibusSlave::checkDestinationAdr(uint8_t destination)
// {
//   if (((destination & 0x7F) != slave_addr) &&  // Slave
//       ((destination & 0x7F) != BROADCAST_ADD)) // Broadcast
//     return false;

//   return true;
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief ISR UART Transmit
//  */
// void CProfibusSlave::interruptPbTx(void)
// {

//   // Alles gesendet?
//   if (m_txCnt < m_txBufCnt)
//   {
//     // TX Buffer fuellen
//     setUartValue(m_txBuffer[m_txCnt++]);
//     // m_printfunc(m_txCnt);
//   }
//   else
//   {
//     TxRs485Disable();
//     // Alles gesendet, Interrupt wieder aus
//     deactivateTxInterrupt();
//     // clear Flag because we are not writing to buffer
//     clearTxFlag();
// // m_printfunc("E");
// #ifdef DEBUG
//     m_printfunc("a\n");
// #endif
//   }

//   setTimerCounter(0);
//   clearOverflowFlag();
// }
// ///////////////////////////////////////////////////////////////////////////////////////////////////

// ///////////////////////////////////////////////////////////////////////////////////////////////////
// /*!
//  * \brief ISR TIMER
//  */
// void CProfibusSlave::interruptTimer(void)
// {

//   // Timer A Stop
//   stopTimer();
//   setTimerCounter(0);

//   switch (stream_status)
//   {
//   case StreamStatus::WaitSyn: // TSYN abgelaufen

//     stream_status = StreamStatus::WaitData;
//     m_rxBufCnt = 0;
//     RxRs485Enable(); // Auf Receive umschalten
//     // activateRxInterrupt();
//     setTimerMax(m_timeoutMaxSdrTime);
//     // activateRxInterrupt();
//     // RS485_RX_EN          // Auf Receive umschalten
//     break;

//   case StreamStatus::WaitData: // TSDR abgelaufen aber keine Daten da
//     // ACITVATE_RX_INTERRUPT
//     // RS485_RX_EN          // Auf Receive umschalten
//     break;

//   case StreamStatus::GetData: // TSDR abgelaufen und Daten da

//     // m_printfunc(stream_status);
//     stream_status = StreamStatus::WaitSyn;
//     setTimerMax(m_timeoutMaxSynTime);

//     // for(uint8_t i=0;i<m_rxBufCnt;i++)
//     // {
//     //   xprintf("%u", m_rxBuffer[i]);
//     // }
//     // xprintf("\n");

//     deactivateRxInterrupt();
// #ifdef DEBUG
// // m_printfunc("%u\n",m_rxBufCnt);
// #endif
//     rxFunc();
//     activateRxInterrupt();

//     break;
//   case StreamStatus::WaitMinTsdr:

//     // TIMER_MAX=minTSDR*TIME_BIT;
//     setTimerMax(m_timeoutMaxTxTime);
//     stream_status = StreamStatus::SendData;
//     // activate Send Interrupt
//     waitForActivTransmission();
//     TxRs485Enable();
//     activateTxInterrupt();
//     setUartValue(m_txBuffer[m_txCnt]);
//     m_txCnt++;

//     break;
//   case StreamStatus::SendData: // Sende-Timeout abgelaufen, wieder auf Empfang gehen

//     stream_status = StreamStatus::WaitSyn;
//     setTimerMax(m_timeoutMaxSynTime);

//     RxRs485Enable(); // Auf Receive umschalten

//     break;

//   default:
//     break;
//   }

//   if (watchdog_act)
//   {
//     if ((millis() - last_connection_time) > watchdog_time)
//     {
//       for (uint8_t cnt = 0; cnt < m_outputReg.size(); cnt++)
//       {
//         m_outputReg[cnt] = 0; // sicherer Zustand
//       }
//       std::vector<uint8_t> unUsed;
//       m_datafunc(m_outputReg, unUsed); // outputs,inputs
//     }
//   }
//   // Timer A STIMER_COUNTERT
//   runTimer();
// }
