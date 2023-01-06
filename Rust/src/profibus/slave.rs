use heapless::Vec;

use super::hwinterface::HwInterface;
use super::types::{
    dpv1_status_byte1, dpv1_status_byte2, dpv1_status_byte3, fc_request, fc_response,
    sap_check_config_request, sap_diagnose_byte1, sap_diagnose_byte2, sap_diagnose_byte3,
    sap_diagnose_ext, sap_set_parameter_request, CmdType, DpSlaveState, StreamState, SAP,
};

pub struct Config {
    ident_high: u8,
    ident_low: u8,
    addr: u8,
    counter_frequency: u32,
    baudrate: u32,
    module_count: u8,
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

    pub fn module_count(mut self, module_count: u8) -> Self {
        self.module_count = module_count;
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
            module_count: 0,
        }
    }
}

const SAP_OFFSET : u8 = 128;
const BROADCAST_ADD :u8 = 127;
const DEFAULT_ADD :u8 = 126;

#[allow(dead_code)]
pub struct PbDpSlave<
    T,
    const BUF_SIZE: usize,
    const INPUT_DATA_SIZE: usize,
    const OUTPUT_DATA_SIZE: usize,
    const USER_PARA_SIZE: usize,
    const EXTERN_DIAG_PARA_SIZE: usize,
    const VENDOR_DATA_SIZE: usize,
> {
    config: Config,
    interface: T,
    buffer: Vec<u8, BUF_SIZE>,
    slave_state: DpSlaveState,
    stream_state: StreamState,
    timeout_max_syn_time_in_us: u32,
    timeout_max_rx_time_in_us: u32,
    timeout_max_tx_time_in_us: u32,
    timeout_max_sdr_time_in_us: u32,

    timer_timeout_in_us: u32,

    input_data: [u8; INPUT_DATA_SIZE],
    output_data: [u8; OUTPUT_DATA_SIZE],
    user_para: [u8; USER_PARA_SIZE],
    extern_diag_para: [u8; EXTERN_DIAG_PARA_SIZE],
    vendor_data: [u8; VENDOR_DATA_SIZE],

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

impl<
        T,
        const BUF_SIZE: usize,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const VENDOR_DATA_SIZE: usize,
    >
    PbDpSlave<
        T,
        BUF_SIZE,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        VENDOR_DATA_SIZE,
    >
where
    T: HwInterface,
{
    pub fn new(mut config: Config, mut interface: T) -> Self {
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
            config.counter_frequency = 1_000_000_u32;
        }

        if 0 == config.baudrate {
            config.baudrate = 500_000_u32;
        }

        let timeout_max_syn_time_in_us = (33 * config.counter_frequency) / config.baudrate; // 33 TBit = TSYN
        let timeout_max_rx_time_in_us = (15 * config.counter_frequency) / config.baudrate;
        let timeout_max_tx_time_in_us = (15 * config.counter_frequency) / config.baudrate;
        let timeout_max_sdr_time_in_us = (15 * config.counter_frequency) / config.baudrate; // 15 Tbit = TSDR

        if (0 == config.addr) || (config.addr > DEFAULT_ADD) {
            config.addr = DEFAULT_ADD;
        }

        // let input_data = Vec::<u8, InputDataSize>::new();
        // let output_data = Vec::<u8, OutputDataSize>::new();
        // let user_para = Vec::<u8, UserParaSize>::new();
        // let extern_diag_para = Vec::<u8, ExternDiagParaSize>::new();
        // let vendor_data = Vec::<u8, VendorDataSize>::new();

        let input_data = [0; INPUT_DATA_SIZE];
        let output_data = [0; OUTPUT_DATA_SIZE];
        let user_para = [0; USER_PARA_SIZE];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];
        let vendor_data = [0; VENDOR_DATA_SIZE];

        // Timer init
        interface.config_timer();
        // LED Status
        interface.config_error_led();
        // Pin Init
        interface.config_rs485_pin();

        // Uart Init
        interface.config_uart();
        interface.run_timer(timeout_max_syn_time_in_us);
        interface.activate_rx_interrupt();
        // activateTxInterrupt();
        interface.tx_rs485_enable();

        let current_time = interface.millis();

        Self {
            config,
            interface,
            buffer: Vec::<u8, BUF_SIZE>::new(),
            slave_state: DpSlaveState::Por,
            stream_state: StreamState::WaitSyn,
            timeout_max_syn_time_in_us,
            timeout_max_rx_time_in_us,
            timeout_max_tx_time_in_us,
            timeout_max_sdr_time_in_us,
            timer_timeout_in_us: timeout_max_syn_time_in_us,
            input_data,
            output_data,
            user_para,
            extern_diag_para,
            vendor_data,
            diagnose_status_1: sap_diagnose_byte1::STATION_NOT_READY,
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
            last_connection_time: current_time,
            watchdog_time: 0xFFFFFF,
        }
    }

    pub fn get_interface(&self) -> &dyn HwInterface {
        &self.interface
    }

    pub fn interrupt_handler(&mut self) {
        if self.interface.rx_data_received() {
            self.rx_interrupt_handler();
        } else if self.interface.tx_data_send() {
            self.tx_interrupt_handler();
        }
    }

    pub fn rx_interrupt_handler(&mut self) {
        loop {
            match self.interface.get_uart_value() {
                Some(data) => {
                    self.handle_rx_byte(data);
                }
                None => break,
            }
        }
    }

    pub fn handle_rx_byte(&mut self, data: u8) {
        self.buffer.push(data).unwrap();

        // if we waited for TSYN, data can be saved
        if StreamState::WaitData == self.stream_state {
            self.stream_state = StreamState::GetData;
        }

        // Einlesen erlaubt?
        if StreamState::GetData == self.stream_state {
            self.buffer.push(data).unwrap();
        }

        // Profibus Timer ruecksetzen
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn tx_interrupt_handler(&mut self) {
        match self.buffer.pop() {
            Some(b) => {
                self.interface.set_uart_value(b);
            }
            _ => {
                self.interface.tx_rs485_disable();
                // Alles gesendet, Interrupt wieder aus
                self.interface.deactivate_tx_interrupt();
                // clear Flag because we are not writing to buffer
                self.interface.clear_tx_flag();
            }
        }

        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn handle_message_timeout(&mut self) {
        let _test = self.buffer.len();

        self.buffer.clear();
    }

    pub fn timer_interrupt_handler(&mut self) {
        // Timer A Stop
        self.interface.stop_timer();

        match self.stream_state {
            StreamState::WaitSyn => {
                self.stream_state = StreamState::WaitData;
                self.buffer.clear();
                self.interface.rx_rs485_enable(); // Auf Receive umschalten
                self.timer_timeout_in_us = self.timeout_max_sdr_time_in_us;
            }
            StreamState::GetData => {
                self.stream_state = StreamState::WaitSyn;
                self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                self.interface.deactivate_rx_interrupt();
                self.handle_receive();
                self.interface.activate_rx_interrupt();
            }
            StreamState::WaitMinTsdr => {
                self.stream_state = StreamState::SendData;
                self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
                self.interface.wait_for_activ_transmission();
                self.interface.tx_rs485_enable();
                self.interface.activate_tx_interrupt();
                match self.buffer.pop() {
                    Some(data) => self.interface.set_uart_value(data),
                    None => (),
                }
            }
            StreamState::SendData => {
                self.stream_state = StreamState::WaitSyn;
                self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                self.interface.rx_rs485_enable();
            }
            _ => (),
        }

        if self.watchdog_act {
            if (self.interface.millis() - self.last_connection_time) > self.watchdog_time {
                self.output_data.fill(0);
                //TODO:
                // std::vector<uint8_t> unUsed;
                // m_datafunc(m_outputReg, unUsed); // outputs,inputs
            }
        }

        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn handle_receive(&mut self)
    {
        
    }

    pub fn calc_checksum(data : Vec<u8, BUF_SIZE>, length : usize) -> u8
    {
      let checksum = 0;

      //TODO
    //   while (length--)
    //   {
    //     csum += data[length];
    //   }

      checksum
    }

    fn check_destination_addr(&self, destination: u8) -> bool {
        if ((destination & 0x7F) != self.config.addr) &&  // Slave
      ((destination & 0x7F) != BROADCAST_ADD)
        // Broadcast
        {
            false
        } else {
            true
        }
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
