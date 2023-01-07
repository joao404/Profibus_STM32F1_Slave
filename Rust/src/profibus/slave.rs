use core::ops::Add;

use super::hwinterface::HwInterface;
use super::types::{
    cmd_type, dpv1_status_byte1, dpv1_status_byte2, dpv1_status_byte3, fc_request, fc_response,
    sap_check_config_request, sap_diagnose_byte1, sap_diagnose_byte2, sap_diagnose_byte3,
    sap_diagnose_ext, sap_set_parameter_request, DpSlaveState, StreamState, SAP, sap_global_control
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

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;

#[allow(dead_code)]
pub struct PbDpSlave<
    T,
    const BUF_len: ulen,
    const INPUT_DATA_len: ulen,
    const OUTPUT_DATA_len: ulen,
    const USER_PARA_len: ulen,
    const EXTERN_DIAG_PARA_len: ulen,
    const VENDOR_DATA_len: ulen,
> {
    config: Config,
    interface: T,
    tx_buffer: [u8; BUF_len],
    rx_buffer: [u8; BUF_len],

    rx_len : ulen,
    tx_len : ulen,
    tx_pos : ulen,

    slave_state: DpSlaveState,
    stream_state: StreamState,
    timeout_max_syn_time_in_us: u32,
    timeout_max_rx_time_in_us: u32,
    timeout_max_tx_time_in_us: u32,
    timeout_max_sdr_time_in_us: u32,

    timer_timeout_in_us: u32,

    input_data: [u8; INPUT_DATA_len],
    output_data: [u8; OUTPUT_DATA_len],
    user_para: [u8; USER_PARA_len],
    extern_diag_para: [u8; EXTERN_DIAG_PARA_len],
    vendor_data: [u8; VENDOR_DATA_len],

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
        const BUF_len: ulen,
        const INPUT_DATA_len: ulen,
        const OUTPUT_DATA_len: ulen,
        const USER_PARA_len: ulen,
        const EXTERN_DIAG_PARA_len: ulen,
        const VENDOR_DATA_len: ulen,
    >
    PbDpSlave<
        T,
        BUF_len,
        INPUT_DATA_len,
        OUTPUT_DATA_len,
        USER_PARA_len,
        EXTERN_DIAG_PARA_len,
        VENDOR_DATA_len,
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

        // let input_data = Vec::<u8, InputDatalen>::new();
        // let output_data = Vec::<u8, OutputDatalen>::new();
        // let user_para = Vec::<u8, UserParalen>::new();
        // let extern_diag_para = Vec::<u8, ExternDiagParalen>::new();
        // let vendor_data = Vec::<u8, VendorDatalen>::new();

        let input_data = [0; INPUT_DATA_len];
        let output_data = [0; OUTPUT_DATA_len];
        let user_para = [0; USER_PARA_len];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_len];
        let vendor_data = [0; VENDOR_DATA_len];

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
            tx_buffer: [u8; BUF_len],
            rx_buffer: [u8; BUF_len],
            rx_len : 0,
            tx_len : 0,
            tx_pos : 0,
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

    pub fn serial_interrupt_handler(&mut self) {
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
      self.rx_buffer[self.rx_len] = data;

        // if we waited for TSYN, data can be saved
        if StreamState::WaitData == self.stream_state {
            self.stream_state = StreamState::GetData;
        }

        // Einlesen erlaubt?
        if StreamState::GetData == self.stream_state {
    if self.rx_len < BUF_len
    {
      self.rx_len+=1;
        }
      }
        // Profibus Timer ruecksetzen
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn tx_interrupt_handler(&mut self) {
      if (tx_pos < tx_len)
      {
        // TX Buffer fuellen
        self.interface.set_uart_value(tx_buffer[tx_pos]);
        self.tx_pos+=1;
        // m_printfunc(m_txCnt);
      }
      else{
                self.interface.tx_rs485_disable();
                // Alles gesendet, Interrupt wieder aus
                self.interface.deactivate_tx_interrupt();
                // clear Flag because we are not writing to buffer
                self.interface.clear_tx_flag();
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
                self.rx_len = 0;
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

    fn transmit_message_sd1(&mut self, function_code: u8, sap_offset: u8) {
        self.tx_buffer[0] = cmd_type::SD1;
        self.tx_buffer[1] = self.master_addr;
        self.tx_buffer[2] = self.config.addr + sap_offset;
        self.tx_buffer[3] =function_code;
        let checksum = self.calc_checksum(&self.tx_buffer[1..4]);
        self.tx_buffer[4] = checksum;
        self.tx_buffer[5] = cmd_type::ED;
        self.tx_len = 6;
        self.transmit();
    }

    fn transmit_message_sd2(&mut self, function_code: u8, sap_offset: u8, pdu: &[u8]) {
        self.tx_buffer[0] = cmd_type::SD2;
        self.tx_buffer[1] = pdu.len();
        self.tx_buffer[2] = pdu.len();
        self.tx_buffer[3] = cmd_type::SD2;
      self.tx_buffer[4] = self.master_addr;
      self.tx_buffer[5] = self.config.addr + sap_offset;
      self.tx_buffer[6] =function_code;

      let checksum = self.calc_checksum(&self.tx_buffer[4..(7+pdu.len())]);
      self.tx_buffer[7 + pdu.len()] = checksum;
      self.tx_buffer[8 + pdu.len()] = cmd_type::ED;
      self.tx_len = 9 + pdu.len();
      self.transmit();
    }

    fn transmit_message_sd3(&mut self, function_code: u8, sap_offset: u8, pdu: &[u8]) {
      self.tx_buffer[0] = cmd_type::SD3;
      self.tx_buffer[1] = self.master_addr;
      self.tx_buffer[2] = self.config.addr + sap_offset;
      self.tx_buffer[3] =function_code;
      let checksum = self.calc_checksum(&self.tx_buffer[4..9]);
      self.tx_buffer[9] = checksum;
      self.tx_buffer[10] = cmd_type::ED;
      self.tx_len = 11;
      self.transmit();
    }

    fn transmit_message_sd4(&mut self, sap_offset: u8) {
      self.tx_buffer[0] = cmd_type::SD4;
      self.tx_buffer[1] = self.master_addr;
      self.tx_buffer[2] = self.config.addr + sap_offset;
      self.tx_len = 3;
      self.transmit();
    }

    fn transmit_message_sc(&mut self) {
        self.tx_buffer[0]=cmd_type::SC;
        self.tx_len = 1;
        self.transmit();
    }

    fn transmit(&mut self) {
      self.tx_pos = 0;
        if 0 != self.min_tdsr {
            self.stream_state = StreamState::WaitMinTsdr;
            self.timer_timeout_in_us = (self.config.counter_frequency * u32::from(self.min_tdsr))
                / self.config.baudrate
                / 2u32;
        } else {
            self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
            self.stream_state = StreamState::SendData;
            // activate Send Interrupt
            self.interface.wait_for_activ_transmission();
            self.interface.tx_rs485_enable();
            self.interface.activate_tx_interrupt();
            self.interface.set_uart_value[self.tx_buffer[self.tx_pos]];
            self.tx_pos+=1;
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    fn calc_checksum(data: &[u8]) -> u8 {
        let checksum = 0;
        for x in data{
          checksum += *x;
        }
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

    fn handle_receive(&mut self) {
  let cnt : u8 = 0;
  let process_data = false;

  // Profibus Datentypen
  let destination_add : u8 = 0;
  let source_add : u8 = 0;
  let function_code : u8 = 0;
  let fcs_data : u8 = 0;     // Frame Check Sequence
  let pdu_len : u8 = 0; // PDU Groesse
  let dsap_data : u8 = 0;    // SAP Destination
  let ssap_data : u8 = 0;    // SAP Source

  match self.buffer[0]
  {
   cmd_type::SD1 => {
    if 6 == self.rx_len
    {
    destination_add = self.rx_buffer[1];
    source_add = self.rx_buffer[2];
    function_code = self.rx_buffer[3];
    fcs_data = self.rx_buffer[4];

    if self.check_destination_addr(destination_add)
    {    
      if fcs_data == self.calc_checksum(&self.rx_buffer[1..4])
    {
// FCV und FCB loeschen, da vorher überprüft
function_code &= 0xCF;
process_data = true;
    }
  }
    }
   }

  cmd_type::SD2 => {
    if self.rx_len == (self.rx_buffer[1] + 6ulen)
    {

    pdu_len = self.rx_buffer[1]; // DA+SA+FC+Nutzdaten
    destination_add = self.rx_buffer[4];
    source_add = self.rx_buffer[5];
    function_code = self.rx_buffer[6];
    fcs_data = self.rx_buffer[pdu_len + 4ulen];

    if self.check_destination_addr(destination_add)
    {    
      if fcs_data == self.calc_checksum(&self.rx_buffer[4..(7+pdu_len)])
    {
// FCV und FCB loeschen, da vorher überprüft
function_code &= 0xCF;
process_data = true;
    }
  }
  }

 }

  cmd_type::SD3 => {

    if 11 == self.rx_len
{
    pdu_len = 8; // DA+SA+FC+Nutzdaten
    destination_add = self.rx_len[1];
    source_add = self.rx_len[2];
    function_code = self.rx_len[3];
    fcs_data = self.rx_len[9];

    if self.check_destination_addr(destination_add)
    {    
      if fcs_data == self.calc_checksum(&self.rx_buffer[4..9])
    {
// FCV und FCB loeschen, da vorher überprüft
function_code &= 0xCF;
process_data = true;
    }
  }
}
  }

  cmd_type::SD4 => {

    if 3 == self.rx_len
{
    destination_add = self.rx_buffer[1];
    source_add = self.rx_buffer[2];

    if self.check_destination_addr(destination_add)
    {
      //TODO  
    }
}
  } 
}// match self.buffer[0]

  if process_data
  {
    self.last_connection_time = self.interface.millis(); // letzte Zeit eines Telegramms sichern

    self.master_addr = source_add; // Master Adresse ist Source Adresse

    if (function_code & 0x30) == fc_request::FCB // Startbedingung
    {
      self.fcv_act = true;
      self.fcb_last = true;
    }
    else if self.fcv_act
    {
      // Adresse wie vorher?
      if source_add != self.source_add_last
      {
        // neue Verbindung und damit FCV ungültig
        fcv_act = false;
      }
      else if (function_code & fc_request::FCB) == self.fcb_last // FCB ist gleich geblieben
      {
        // Nachricht wiederholen
        self.transmit();
        // die Nachricht liegt noch im Speicher
      }
      else // Speichern des neuen FCB
      {
        self.fcb_last = !self.fcb_last; // das negierte bit speichern, da sonst die vorherige Bedingung angeschlagen hätte
      }
    }
    else // wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
    {
      self.fcv_act = false;
    }

    // letzte Adresse sichern
    self.source_add_last = source_add;

    // Service Access Point erkannt?
    if (destination_add & 0x80) && (source_add & 0x80)
    {
      dsap_data = self.rx_buffer[7];
      ssap_data = self.rx_buffer[8];

      // Ablauf Reboot:
      // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
      // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
      // 3) SSAP 62 -> DSAP 62 (Check Config Request)
      // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
      // 5) Data Exchange Request (normaler Zyklus)

      // Siehe Felser 8/2009 Kap. 4.1
      // m_printfunc((int)DSAP_data);
      match dsap_data
      {
      SAP::SetSlaveAdr => { // Set Slave Address (SSAP 62 -> DSAP 55)
                              // Siehe Felser 8/2009 Kap. 4.2

        // Nur im Zustand "Wait Parameter" (WPRM) moeglich

        if DpSlaveState::Wrpm == self.slave_state
        {
          // adresse ändern
          // new_addr = pb_uart_buffer[9];
          // IDENT_HIGH_BYTE = m_pbUartRxBuffer[10];
          // IDENT_LOW_BYTE = m_pbUartRxBuffer[11];
          // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
        }

        self.transmit_message_sc();
     }

     SAP::GlobalControl => { // Global Control Request (SSAP 62 -> DSAP 58)
                               // Siehe Felser 8/2009 Kap. 4.6.2

        // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
        if (self.rx_buffer[9] & sap_global_control::CLEAR_DATA) != 0
        {
          self.interface.error_led_on(); // Status "SPS nicht bereit"
        }
        else
        {
          self.interface.error_led_off(); // Status "SPS OK"
        }

        // Gruppe berechnen
        // for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;

        // Wenn Befehl fuer uns ist
        if (self.rx_buffer[10] & self.group) != 0 //(cnt == group)
        {
          if (self.rx_buffer[9] & sap_global_control::UNFREEZE) != 0
          {
            // FREEZE Zustand loeschen
            self.freeze = false;
            // m_datafunc(NULL,&(self.tx_buffer[7]));//outputs,inputs
          }
          else if (self.rx_buffer[9] & sap_global_control::UNSYNC) != 0
          {
            // SYNC Zustand loeschen
            self.sync = false;
            std::vector<uint8_t> inputDelete;
            m_datafunc(m_outputReg, inputDelete); // outputs,inputs
          }
          else if (self.rx_buffer[9] & sap_global_control::FREEZE) != 0
          {
            // Eingaenge nicht mehr neu einlesen
            if (self.freeze)
            {
              std::vector<uint8_t> outputFreeze;
              m_datafunc(outputFreeze, m_inputReg); // outputs,inputs
            }
            self.freeze = true;
          }
          else if (self.rx_buffer[9] & sap_global_control::SYNC) != 0
          {
            // Ausgaenge nur bei SYNC Befehl setzen

            if self.sync
            {
              std::vector<uint8_t> inputNotUsed;
              m_datafunc(m_outputReg, inputNotUsed); // outputs,inputs
            }
            self.sync = true;
          }
        }

     }

     SAP::SlaveDiagnostic => { // Get Diagnostics Request (SSAP 62 -> DSAP 60)
                                // Siehe Felser 8/2009 Kap. 4.5.2

        // Nach dem Erhalt der Diagnose wechselt der DP-Slave vom Zustand
        // "Power on Reset" (POR) in den Zustand "Wait Parameter" (WPRM)

        // Am Ende der Initialisierung (Zustand "Data Exchange" (DXCHG))
        // sendet der Master ein zweites mal ein Diagnostics Request um die
        // korrekte Konfiguration zu pruefen
        // m_printfunc((int)function_code);
        // m_printfunc(REQUEST_ + SRD_HIGH);
        if (function_code == (fc_request::REQUEST + fc_request::SRD_HIGH)) ||
            (function_code == (fc_request::REQUEST + fc_request::SRD_LOW))
        {
          // Erste Diagnose Abfrage (Aufruf Telegramm)
          // pb_uart_buffer[4]  = master_addr;                  // Ziel Master (mit SAP Offset)
          // pb_uart_buffer[5]  = slave_addr + SAP_OFFSET;      // Quelle Slave (mit SAP Offset)
          // pb_uart_buffer[6]  = SLAVE_DATA;
          self.tx_buffer[7] = ssap_data;         // Ziel SAP Master
          self.tx_buffer[8] = dsap_data;         // Quelle SAP Slave
          self.tx_buffer[9] = diagnose_status_1; // Status 1
          if DpSlaveState::Por == slave_status
          {
            self.tx_buffer[10] = STATUS_2_DEFAULT + PRM_REQ_ + 0x04; // Status 2
            self.tx_buffer[12] = MASTER_ADD_DEFAULT;                 // Adresse Master
          }
          else
          {
            self.tx_buffer[10] = STATUS_2_DEFAULT + 0x04;  // Status 2
            self.tx_buffer[12] = master_addr - SAP_OFFSET; // Adresse Master
          }

          if self.watchdog_act
          {
            self.tx_buffer[10] |= WD_ON_;
          }

          if self.freeze_act
          {
            self.tx_buffer[10] |= FREEZE_MODE_;
          }

          if self.sync_act
          {
            self.tx_buffer[10] |= SYNC_MODE_;
          }

          self.tx_buffer[11] = DIAG_len_OK;       // Status 3
          self.tx_buffer[13] = m_config.identHigh; // Ident high
          self.tx_buffer[14] = m_config.identLow;  // Ident low
          if self.extern_diag_para.len() > 0
          {
            self.tx_buffer[15] = sap_diagnose_ext::EXT_DIAG_GERAET + self.extern_diag_para.len() + 1; // Diagnose (Typ und Anzahl Bytes)
            for (cnt = 0; cnt < self.extern_diag_para.len(); cnt++)
            {
              self.tx_buffer[16 + cnt] = self.extern_diag_para[cnt];
            }

            sendCmd(cmd_type::SD2, DATA_LOW, SAP_OFFSET, &self.tx_buffer[7], 9 + m_diagData.len());
          }
          else
          {

            sendCmd(cmd_type::SD2, DATA_LOW, SAP_OFFSET, &self.tx_buffer[7], 8);
          }
        }

        // Status aendern
        if (DpSlaveState::Por == self.slave_state)
        {
          self.slave_state = DpSlaveState::Wprm;
        }

     }

     SAP::SetPrm => { // Set Parameters Request (SSAP 62 -> DSAP 61)
                        // Siehe Felser 8/2009 Kap. 4.3.1

        // Nach dem Erhalt der Parameter wechselt der DP-Slave vom Zustand
        // "Wait Parameter" (WPRM) in den Zustand "Wait Configuration" (WCFG)
        // m_printfunc((int)pb_uart_buffer[13]);
        // m_printfunc(":");
        // m_printfunc((int)pb_uart_buffer[14]);
        // Nur Daten fuer unser Geraet akzeptieren
        // m_printfunc((int)pb_uart_buffer[13]);
        // m_printfunc((int)IDENT_HIGH_BYTE);
        // m_printfunc((int)pb_uart_buffer[14]);
        // m_printfunc((int)IDENT_LOW_BYTE);
        if ((self.rx_buffer[13] == self.config.ident_high) && (self.rx_buffer[14] == self.config.ident_low))
        {
          // pb_uart_buffer[9]  = Befehl
          // pb_uart_buffer[10] = Watchdog 1
          // pb_uart_buffer[11] = Watchdog 2
          // pb_uart_buffer[12] = Min TSDR
          // pb_uart_buffer[13] = Ident H
          // pb_uart_buffer[14] = Ident L
          // pb_uart_buffer[15] = Gruppe
          // pb_uart_buffer[16] = User Parameter

          // Bei DPV1 Unterstuetzung:
          // pb_uart_buffer[16] = DPV1 Status 1
          // pb_uart_buffer[17] = DPV1 Status 2
          // pb_uart_buffer[18] = DPV1 Status 3
          // pb_uart_buffer[19] = User Parameter

          if (!(self.rx_buffer[9] & ACTIVATE_WATCHDOG_)) // Watchdog aktivieren
          {
            self.watchdog_act = true;
          }
          else
          {
            self.watchdog_act = false;
          }

          if (!(self.rx_buffer[9] & ACTIVATE_FREEZE_))
          {
            self.freeze_act = true;
          }
          else
          {
            self.freeze_act = false;
          }

          if (!(self.rx_buffer[9] & ACTIVATE_SYNC_))
          {
            self.sync_act = true;
          }
          else
          {
            self.sync_act = false;
          }

          // watchdog1 = m_pbUartRxBuffer[10];
          // watchdog2 = m_pbUartRxBuffer[11];

          self.watchdog_time = self.rx_buffer[10] * self.rx_buffer[11] * 10;

          if (self.rx_buffer[12] > 10)
          {
            self.min_tsdr = self.rx_buffer[12] - 11;
          }
          else
          {
            self.min_tsdr = 0;
          }

           self.config.ident_high = self.rx_buffer[13];
          self.config.ident_low = self.rx_buffer[14];

          // User Parameter groe�e = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
          User_Para_len = PDU_len - 12;

          // User Parameter einlesen
          if (m_userPara.len() > 0)
          {
            for (cnt = 0; cnt < m_userPara.len(); cnt++)
              m_userPara[cnt] = self.rx_buffer[16 + cnt];
          }

          // Gruppe einlesen
          // for (group = 0; pb_uart_buffer[15] != 0; group++) pb_uart_buffer[15]>>=1;

          group = self.rx_buffer[15]; // wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

          // Kurzquittung
          sendCmd(cmd_type::SC, 0, SAP_OFFSET, &self.tx_buffer[0], 0);
          // m_printfunc("Quittung");
          if (DpSlaveState::Wprm == self.slave_state)
          {
            self.slave_state = DpSlaveState::Wcfg;
          }
        }

      }

      SAP::ChkCfg => { // Check Config Request (SSAP 62 -> DSAP 62)
                        // Siehe Felser 8/2009 Kap. 4.4.1

        // Nach dem Erhalt der Konfiguration wechselt der DP-Slave vom Zustand
        // "Wait Configuration" (WCFG) in den Zustand "Data Exchange" (DXCHG)

        // IO Konfiguration:
        // Kompaktes Format fuer max. 16/32 Byte IO
        // Spezielles Format fuer max. 64/132 Byte IO

        Module_cnt = 0;
        Vendor_Data_len = 0;

        // Je nach PDU Datengroesse mehrere Bytes auswerten
        // LE/LEr - (DA+SA+FC+DSAP+SSAP) = Anzahl Config Bytes
        for (cnt = 0; cnt < self.rx_buffer[1] - 5; cnt++)
        {
           match (self.rx_buffer[9 + cnt] & sap_check_config_request::CFG_DIRECTION) {
            sap_check_config_request::CFG_INPUT => {

            // Input_Data_len = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
            // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
            //   Input_Data_len = Input_Data_len*2;

            m_moduleData[Module_cnt][0] = (self.rx_buffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
            if (self.rx_buffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
              m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

            Module_cnt++;

            }

            sap_check_config_request::CFG_OUTPUT => {

            // Output_Data_len = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
            // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
            //   Output_Data_len = Output_Data_len*2;

            m_moduleData[Module_cnt][1] = (self.rx_buffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
            if (self.rx_buffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
              m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

            Module_cnt++;

            }

            sap_check_config_request::CFG_INPUT_OUTPUT => {

            // Input_Data_len = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
            // Output_Data_len = (pb_uart_buffer[9+cnt] & CFG_BYTE_CNT_) + 1;
            // if (pb_uart_buffer[9+cnt] & CFG_WIDTH_ & CFG_WORD)
            //{
            //   Input_Data_len = Input_Data_len*2;
            //   Output_Data_len = Output_Data_len*2;
            // }

            m_moduleData[Module_cnt][0] = (self.rx_buffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
            m_moduleData[Module_cnt][1] = (self.rx_buffer[9 + cnt] & CFG_BYTE_CNT_) + 1;
            if (self.rx_buffer[9 + cnt] & CFG_WIDTH_ & CFG_WORD)
            {
              m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;
              m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;
            }

            Module_cnt++;

          }

          sap_check_config_request::CFG_SPECIAL => {

            // Spezielles Format

            // Herstellerspezifische Bytes vorhanden?
            if (self.rx_buffer[9 + cnt] & CFG_SP_VENDOR_CNT_)
            {
              // Anzahl Herstellerdaten sichern
              Vendor_Data_len += self.rx_buffer[9 + cnt] & CFG_SP_VENDOR_CNT_;

              // Vendor_Data[] = pb_uart_buffer[];

              // Anzahl von Gesamtanzahl abziehen
              self.rx_buffer[1] -= self.rx_buffer[9 + cnt] & CFG_SP_VENDOR_CNT_;
            }

            // I/O Daten
            match (self.rx_buffer[9 + cnt] & sap_check_config_request::CFG_SP_DIRECTION)
            {
                sap_check_config_request::CFG_SP_VOID => { // Leeres Modul (1 Byte)

              m_moduleData[Module_cnt][0] = 0;
              m_moduleData[Module_cnt][1] = 0;

              Module_cnt++;

                  }

                  sap_check_config_request::CFG_SP_INPUT => { // Eingang (2 Byte)

              // Input_Data_len = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
              // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
              //   Input_Data_len = Input_Data_len*2;

              m_moduleData[Module_cnt][0] = (self.rx_buffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
              if (self.rx_buffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
                m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

              Module_cnt++;

              cnt++; // Zweites Byte haben wir jetzt schon

                   }

                   sap_check_config_request::CFG_SP_OUTPUT => { // Ausgang (2 Byte)

              // Output_Data_len = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
              // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
              //   Output_Data_len = Output_Data_len*2;

              m_moduleData[Module_cnt][1] = (self.rx_buffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
              if (self.rx_buffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
                m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

              Module_cnt+=1;

              cnt+=1; // Zweites Byte haben wir jetzt schon

                    }

                    sap_check_config_request::CFG_SP_INPUT_OUTPUT =>{ // Ein- und Ausgang (3 Byte)

              // Erst Ausgang...
              // Output_Data_len = (pb_uart_buffer[10+cnt] & CFG_SP_BYTE_CNT_) + 1;
              // if (pb_uart_buffer[10+cnt] & CFG_WIDTH_ & CFG_WORD)
              //  Output_Data_len = Output_Data_len*2;

              // Dann Eingang
              // Input_Data_len = (pb_uart_buffer[11+cnt] & CFG_SP_BYTE_CNT_) + 1;
              // if (pb_uart_buffer[11+cnt] & CFG_WIDTH_ & CFG_WORD)
              //  Input_Data_len = Input_Data_len*2;

              // Erst Ausgang...
              m_moduleData[Module_cnt][0] = (self.rx_buffer[10 + cnt] & CFG_SP_BYTE_CNT_) + 1;
              if (self.rx_buffer[10 + cnt] & CFG_WIDTH_ & CFG_WORD)
                m_moduleData[Module_cnt][0] = m_moduleData[Module_cnt][0] * 2;

              // Dann Eingang
              m_moduleData[Module_cnt][1] = (self.rx_buffer[11 + cnt] & CFG_SP_BYTE_CNT_) + 1;
              if (self.rx_buffer[11 + cnt] & CFG_WIDTH_ & CFG_WORD)
                m_moduleData[Module_cnt][1] = m_moduleData[Module_cnt][1] * 2;

              Module_cnt++;

              cnt += 2; // Zweites und drittes Bytes haben wir jetzt schon

                    }

            } // Switch Ende

        }

        _=> (),

        }   // For Ende

        if (Vendor_Data_len != 0)
        {
          // Auswerten
        }

        // Bei Fehler -> CFG_FAULT_ in Diagnose senden
        if (self.vendor_data.len() > 0) && (Module_cnt > m_moduleData.len() || Vendor_Data_len != self.vendor_data.len())
        {
          diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
        }
        else if ((self.vendor_data.len() == 0) && (Module_cnt > m_config.moduleCount))
        {
          diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
        }
        else
        {
          diagnose_status_1 &= !(sap_diagnose_byte1::STATION_NOT_READY + sap_diagnose_byte1::CFG_FAULT);
        }

        // Kurzquittung
        sendCmd(cmd_type::SC, 0, SAP_OFFSET, &self.tx_buffer[0], 0);

        if (DpSlaveState::Wcfg == self.slave_state)
        {
          self.slave_state = DpSlaveState::Dxchg;
        }

    }

    _ => (),

      } // Switch DSAP_data Ende
    }
    // Ziel: Slave Adresse, but no SAP
    else if (destination_add == self.config.addr)
    {

      // Status Abfrage
      if function_code == (fc_request::REQUEST + fc_request::FDL_STATUS)
      {
        self.transmit_message_sd1(fc_response::FDL_STATUS_OK, 0);
      }
      // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
      /*
      else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) ||
               function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
      {
       */
      else if function_code == (fc_request::REQUEST + fc_request::SRD_HIGH) ||
               function_code == (fc_request::REQUEST + fc_request::SRD_LOW)
      {
        if self.sync_act && self.sync // write data in output_register when sync
        {
          for (cnt = 0; cnt < m_outputReg.len(); cnt++)
          {
            m_outputReg[cnt] = self.rx_buffer[cnt + 7];
          }
        }
        else // normaler Betrieb
        {
          for (cnt = 0; cnt < m_outputReg.len(); cnt++)
          {
            m_outputReg[cnt] = self.rx_buffer[cnt + 7];
          }
          std::vector<uint8_t> unUsed;
          m_datafunc(m_outputReg, unUsed); // outputs,inputs
        }

        if self.freeze_act && self.freeze // write input_register in telegram when freeze
        {
          for (cnt = 0; cnt < m_inputReg.len(); cnt++)
          {
            self.tx_buffer[cnt + 7] = m_inputReg[cnt];
          }
        }
        else // normaler Betrieb
        {
          std::vector<uint8_t> unUsed;
          m_datafunc(unUsed, m_inputReg); // outputs,inputs
          for (cnt = 0; cnt < m_inputReg.len(); cnt++)
          {
            self.tx_buffer[cnt + 7] = m_inputReg[cnt];
          }
        }

        if self.input_data.len() > 0
        {
          if (self.diagnose_status_1 & sap_diagnose_byte1::EXT_DIAG) != 0
          {
            self.transmit_message_sd2(fc_response::DATA_HIGH, 0, &self.tx_buffer[7]); // Diagnose Abfrage anfordern
          }
          else
          {
            sendCmd(cmd_type::SD2, DATA_LOW, 0, &self.tx_buffer[7], m_inputReg.len()); // Daten senden
          }
        }
        else
        {
          // TODO
          //  if (diagnose_status_1 & EXT_DIAG_ || (get_Address() & 0x80))
          //    sendCmd(cmd_type::SD1, DATA_HIGH, 0, &self.tx_buffer[7], 0); // Diagnose Abfrage anfordern
          //  else
          //    sendCmd(cmd_type::SC, 0, 0, &self.tx_buffer[7], 0); // Kurzquittung
        }
      }
    }

  }
  else // data not valid
  {
    self.rx_len = 0;
  }
}
}
}
