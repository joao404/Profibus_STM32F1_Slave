use super::codec::{Codec, Config as CodecConfig, ReceiveHandling, UartAccess, CodecVariables};
use super::data_handling_interface::DataHandlingInterface;
use super::hw_interface::HwInterface;

use super::types::{
     fc_request, fc_response, sap, sap_diagnose_byte1, sap_diagnose_byte2,
    sap_diagnose_byte3, sap_diagnose_ext, sap_global_control, sap_set_parameter_request,
    DpSlaveState, StreamState,
};

// This class shall implement all traits for fdl and codec and combine them
// it also provides the getter for codec to get parameters like addr etc. as inline code
// only this class is template with given parameters

pub struct Config {
    ident_high: u8,
    ident_low: u8,
    codec_config: CodecConfig,
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
        self.codec_config = self.codec_config.t_s(addr);
        self
    }

    pub fn t_sl(mut self, t_sl: u16) -> Self {
        self.codec_config = self.codec_config.t_sl(t_sl);
        self
    }

    pub fn t_sdr_min(mut self, t_sdr_min: u16) -> Self {
        self.codec_config = self.codec_config.t_sdr_min(t_sdr_min);
        self
    }

    pub fn rx_handling(mut self, rx_handling: UartAccess) -> Self {
        self.codec_config = self.codec_config.rx_handling(rx_handling);
        self
    }

    pub fn tx_handling(mut self, tx_handling: UartAccess) -> Self {
        self.codec_config = self.codec_config.tx_handling(tx_handling);
        self
    }

    pub fn receive_handling(mut self, receive_handling: ReceiveHandling) -> Self {
        self.codec_config = self.codec_config.receive_handling(receive_handling);
        self
    }
}

impl Default for Config {
    fn default() -> Config {
        let codec_config = CodecConfig::default()
            .t_s(126)
            .rx_handling(UartAccess::SingleByte)
            .tx_handling(UartAccess::SingleByte)
            .receive_handling(ReceiveHandling::Interrupt);
        Config {
            ident_high: 0,
            ident_low: 0,
            codec_config,
        }
    }
}

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;
const MASTER_ADD_DEFAULT: u8 = 0xFF;

#[allow(dead_code)]
pub struct PbDpSlave<
    // 'a,
    Serial,
    DataHandling,
    const INPUT_DATA_SIZE: usize,
    const OUTPUT_DATA_SIZE: usize,
    const USER_PARA_SIZE: usize,
    const EXTERN_DIAG_PARA_SIZE: usize,
    const MODULE_CONFIG_SIZE: usize,
> {
    config: Config,
    data_handling_interface: DataHandling,

    
    codec_interface: Serial,
    codec_variables : CodecVariables,

    slave_state: DpSlaveState,
    input_data: [u8; INPUT_DATA_SIZE],
    input_data_buffer: [u8; INPUT_DATA_SIZE],
    output_data: [u8; OUTPUT_DATA_SIZE],
    output_data_buffer: [u8; OUTPUT_DATA_SIZE],
    user_para: [u8; USER_PARA_SIZE],
    extern_diag_para: [u8; EXTERN_DIAG_PARA_SIZE],
    module_config: [u8; MODULE_CONFIG_SIZE],

    diagnose_status_1: u8,
    master_addr: u8,
    group: u8,

    source_addr: u8,
    fcv_activated: bool,
    fcb_last: bool,

    freeze: bool,
    sync: bool,
    watchdog_act: bool,
    min_tsdr: u8,

    freeze_configured: bool,
    sync_configured: bool,

    last_connection_time: u32,
    watchdog_time: u32,
}

impl<
        // 'a,
        Serial,
        DataHandling,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const MODULE_CONFIG_SIZE: usize,
    >
    PbDpSlave<
        // 'a,
        Serial,
        DataHandling,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    Serial: HwInterface,
    DataHandling: DataHandlingInterface,
{
    pub fn new(
        mut serial_interface: Serial,
        mut data_handling_interface: DataHandling,
        mut config: Config,
        module_config: [u8; MODULE_CONFIG_SIZE],
    ) -> Self {

        let input_data = [0; INPUT_DATA_SIZE];
        let input_data_buffer = [0; INPUT_DATA_SIZE];
        let output_data = [0; OUTPUT_DATA_SIZE];
        let output_data_buffer = [0; OUTPUT_DATA_SIZE];
        let user_para = [0; USER_PARA_SIZE];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];
        // LED Status
        data_handling_interface.config_error_led();

        let current_time = data_handling_interface.millis();

        data_handling_interface.debug_write("Profi");

        let mut instance = Self {
            config,
            data_handling_interface,

            codec_interface:serial_interface,
            codec_variables : CodecVariables::default(),//TODo

            slave_state: DpSlaveState::Por,
            input_data,
            input_data_buffer,
            output_data,
            output_data_buffer,
            user_para,
            extern_diag_para,
            module_config,
            diagnose_status_1: sap_diagnose_byte1::STATION_NOT_READY,
            master_addr: 0xFF,
            group: 0,
            source_addr: 0xFF,
            fcv_activated: false,
            fcb_last: false,
            freeze: false,
            sync: false,
            watchdog_act: false,
            min_tsdr: 0,
            freeze_configured: false,
            sync_configured: false,
            last_connection_time: current_time,
            watchdog_time: 0xFFFFFF,
        };
        //TODO
        instance.codec_config(&mut instance.config.codec_config);
        instance
    }

    pub fn access_output(&mut self) -> &mut [u8; OUTPUT_DATA_SIZE] {
        &mut self.output_data
    }

    pub fn access_input(&mut self) -> &mut [u8; INPUT_DATA_SIZE] {
        &mut self.input_data
    }
}

impl<
        Serial,
        DataHandling,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const MODULE_CONFIG_SIZE: usize,
    > Codec
    for PbDpSlave<
        Serial,
        DataHandling,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    Serial: HwInterface,
    DataHandling: DataHandlingInterface,
{
    fn fdl_handle_data(
        &mut self,
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        pdu: &[u8],
    ) -> bool
    {
        false
    }

    fn access_codec_variables(&mut self)->&mut CodecVariables
    {
        &mut self.codec_variables
    }

    fn config_timer(&mut self)
    {
       self.codec_interface.config_timer(); 
    }

    fn run_timer(&mut self, _timeout_in_us: u32)
    {
        self.codec_interface.run_timer(_timeout_in_us);
    }

    fn stop_timer(&mut self)
    {
        self.codec_interface.stop_timer();
    }

    fn clear_overflow_flag(&mut self)
    {
        self.codec_interface.clear_overflow_flag();
    }

    fn config_uart(&mut self)
    {
        self.codec_interface.config_uart();
    }

    fn activate_tx_interrupt(&mut self)
    {
        self.codec_interface.activate_tx_interrupt();
    }

    fn deactivate_tx_interrupt(&mut self)
    {
        self.codec_interface.deactivate_tx_interrupt();
    }

    fn activate_rx_interrupt(&mut self)
    {
        self.codec_interface.activate_rx_interrupt();
    }

    fn deactivate_rx_interrupt(&mut self)
    {
        self.codec_interface.deactivate_rx_interrupt();
    }

    fn activate_idle_interrupt(&mut self)
    {
        self.codec_interface.activate_idle_interrupt();
    }

    fn deactivate_idle_interrupt(&mut self)
    {
        self.codec_interface.deactivate_idle_interrupt();
    }

    fn set_tx_flag(&mut self)
    {
        self.codec_interface.set_tx_flag();
    }

    fn clear_tx_flag(&mut self)
    {
        self.codec_interface.clear_tx_flag();
    }

    fn clear_rx_flag(&mut self)
    {
        self.codec_interface.clear_rx_flag();
    }

    fn clear_idle_flag(&mut self)
    {
        self.codec_interface.clear_idle_flag();
    }

    fn wait_for_activ_transmission(&mut self)
    {
        self.codec_interface.wait_for_activ_transmission();
    }

    fn is_rx_received(&mut self) -> bool
    {
        self.codec_interface.is_rx_received()
    }

    fn is_rx_idle(&mut self) -> bool
    {
        self.codec_interface.is_rx_idle()
    }

    fn is_tx_done(&mut self) -> bool
    {
        self.codec_interface.is_tx_done()
    }

    fn tx_rs485_enable(&mut self)
    {
        self.codec_interface.tx_rs485_enable();
    }

    fn tx_rs485_disable(&mut self)
    {
        self.codec_interface.tx_rs485_disable();
    }

    fn rx_rs485_enable(&mut self)
    {
        self.codec_interface.rx_rs485_enable();
    }

    fn config_rs485_pin(&mut self)
    {
        self.codec_interface.config_rs485_pin();
    }

    fn get_uart_value(&mut self) -> Option<u8>
    {
        self.codec_interface.get_uart_value()
    }

    fn set_uart_value(&mut self, _value: u8)
    {
        self.codec_interface.set_uart_value(_value);
    }

    fn send_uart_data(&mut self, _len: usize)
    {
        self.codec_interface.send_uart_data(_len);
    }

    fn get_uart_data(&mut self) -> usize
    {
        self.codec_interface.get_uart_data()
    }

    fn schedule_receive_handling(&mut self)
    {
        self.codec_interface.schedule_receive_handling();
    }

    fn get_rx_buffer(&mut self) -> &mut [u8]
    {
        self.codec_interface.get_rx_buffer()
    }

    fn get_tx_buffer(&mut self) -> &mut [u8]
    {
        self.codec_interface.get_tx_buffer()
    }

    fn get_timer_frequency(&self) -> u32
    {
        self.codec_interface.get_timer_frequency()
    }
    
    fn get_baudrate(&self) -> u32 {
        self.codec_interface.get_baudrate()
    }

    fn debug_write(&mut self, _debug: &str)
    {
        self.codec_interface.debug_write(_debug);
    }
}

// pub(crate) fn fdl_handle_data(
//     source_addr: u8,
//     destination_addr: u8,
//     function_code: u8,
//     pdu: &[u8],
// ) -> bool
// {
//     false
// }

//     // if self.watchdog_act {
//     //     if (self.data_handling_interface.millis() - self.last_connection_time)
//     //         > self.watchdog_time
//     //     {
//     //         self.output_data.fill(0);
//     //         //TODO:
//     //         // std::vector<uint8_t> unUsed;
//     //         // m_datafunc(m_outputReg, unUsed); // outputs,inputs
//     //         self.data_handling_interface
//     //             .data_processing(&mut [0; 0], &self.output_data[..]);
//     //     }
//     // }
// }

// impl<
//         // 'a,
//         // Serial,
//         DataHandling,
//         // const BUF_SIZE: usize,
//         const INPUT_DATA_SIZE: usize,
//         const OUTPUT_DATA_SIZE: usize,
//         const USER_PARA_SIZE: usize,
//         const EXTERN_DIAG_PARA_SIZE: usize,
//         const MODULE_CONFIG_SIZE: usize,
//     > Fdl
//     for PbDpSlave<
//         // 'a,
//         // Serial,
//         DataHandling,
//         // BUF_SIZE,
//         INPUT_DATA_SIZE,
//         OUTPUT_DATA_SIZE,
//         USER_PARA_SIZE,
//         EXTERN_DIAG_PARA_SIZE,
//         MODULE_CONFIG_SIZE,
//     >
// where
//     // Serial: HwInterface,
//     DataHandling: DataHandlingInterface,
// {
//     fn handle_data_receive(
//         &mut self,
//         source_addr: u8,
//         destination_addr: u8,
//         function_code: u8,
//         pdu: &[u8],
//     ) -> bool {
//         // let mut response = false;

//         // self.last_connection_time = self.data_handling_interface.millis(); // letzte Zeit eines Telegramms sichern

//         // let pdu_len = pdu.len();

//         // if (function_code & 0x30) == fc_request::FCB
//         // // Startbedingung
//         // {
//         //     self.fcv_activated = true;
//         //     self.fcb_last = true;
//         // } else if self.fcv_activated {
//         //     if source_addr != self.source_addr {
//         //         // new address so fcv is deactivated
//         //         self.fcv_activated = false;
//         //     } else if ((function_code & fc_request::FCB) != 0) == self.fcb_last {
//         //         // FCB is identical, repeat message
//         //         response = true;
//         //         self.codec.transmit();
//         //     } else {
//         //         // save new FCB bit
//         //         self.fcb_last = !self.fcb_last;
//         //     }
//         // } else
//         // // wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
//         // {
//         //     self.fcv_activated = false;
//         // }

//         // // letzte Adresse sichern
//         // self.source_addr = source_addr;

//         // // Service Access Point erkannt?
//         // if ((destination_addr & 0x80) != 0) && ((source_addr & 0x80) != 0) {
//         //     let dsap_data = pdu[0]; // sap destination
//         //     let ssap_data = pdu[1]; // sap source

//         //     // Ablauf Reboot:
//         //     // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
//         //     // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
//         //     // 3) SSAP 62 -> DSAP 62 (Check Config Request)
//         //     // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
//         //     // 5) Data Exchange Request (normaler Zyklus)

//         //     // Siehe Felser 8/2009 Kap. 4.1
//         //     match dsap_data {
//         //         sap::SET_SLAVE_ADR => {
//         //             // Set Slave Address (SSAP 62 -> DSAP 55)
//         //             // Siehe Felser 8/2009 Kap. 4.2
//         //             if DpSlaveState::Wrpm == self.slave_state {
//         //                 if 9 == pdu_len {
//         //                     self.config.codec_config = self.config.codec_config.t_s(pdu[2]);
//         //                     self.config.ident_high = pdu[3];
//         //                     self.config.ident_low = pdu[4];
//         //                     //TODO
//         //                     // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
//         //                     // trigger value saving
//         //                 }
//         //             }
//         //             response = true;
//         //             self.codec.transmit_message_sc();
//         //         }

//         //         sap::GLOBAL_CONTROL => {
//         //             // Global Control Request (SSAP 62 -> DSAP 58)
//         //             // Siehe Felser 8/2009 Kap. 4.6.2

//         //             // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
//         //             if (pdu[3] & sap_global_control::CLEAR_DATA) != 0 {
//         //                 self.data_handling_interface.error_led_on(); // Status "SPS nicht bereit"
//         //             } else {
//         //                 self.data_handling_interface.error_led_off(); // Status "SPS OK"
//         //             }

//         //             // Gruppe berechnen
//         //             // for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;

//         //             // Wenn Befehl fuer uns ist
//         //             if (pdu[3] & self.group) != 0
//         //             //(cnt == group)
//         //             {
//         //                 if (pdu[2] & sap_global_control::UNFREEZE) != 0 {
//         //                     // FREEZE Zustand loeschen
//         //                     self.freeze = false;
//         //                     self.data_handling_interface
//         //                         .data_processing(&mut self.input_data[..], &[0; 0]);
//         //                     //TODO: only a copy is given
//         //                     self.input_data_buffer = self.input_data;
//         //                 } else if (pdu[2] & sap_global_control::UNSYNC) != 0 {
//         //                     // SYNC Zustand loeschen
//         //                     self.sync = false;
//         //                     self.output_data = self.output_data_buffer;
//         //                     self.data_handling_interface
//         //                         .data_processing(&mut [0; 0], &self.output_data[..]);
//         //                 } else if (pdu[2] & sap_global_control::FREEZE) != 0 {
//         //                     // Eingaenge nicht mehr neu einlesen
//         //                     self.freeze = true;
//         //                     self.data_handling_interface
//         //                         .data_processing(&mut self.input_data[..], &[0; 0]);
//         //                     //TODO: only a copy is given
//         //                     self.input_data_buffer = self.input_data;
//         //                 } else if (pdu[2] & sap_global_control::SYNC) != 0 {
//         //                     // Ausgaenge nur bei SYNC Befehl setzen
//         //                     self.sync = true;
//         //                     self.output_data = self.output_data_buffer;
//         //                     self.data_handling_interface
//         //                         .data_processing(&mut [0; 0], &self.output_data[..]);
//         //                 }
//         //             }
//         //         }

//         //         sap::SLAVE_DIAGNOSTIC => {
//         //             // Get Diagnostics Request (SSAP 62 -> DSAP 60)
//         //             // Siehe Felser 8/2009 Kap. 4.5.2

//         //             // Nach dem Erhalt der Diagnose wechselt der DP-Slave vom Zustand
//         //             // "Power on Reset" (POR) in den Zustand "Wait Parameter" (WPRM)

//         //             // Am Ende der Initialisierung (Zustand "Data Exchange" (DXCHG))
//         //             // sendet der Master ein zweites mal ein Diagnostics Request um die
//         //             // korrekte Konfiguration zu pruefen
//         //             // m_printfunc((int)function_code);
//         //             // m_printfunc(REQUEST_ + SRD_HIGH);
//         //             if (function_code == (fc_request::REQUEST + fc_request::SRD_HIGH))
//         //                 || (function_code == (fc_request::REQUEST + fc_request::SRD_LOW))
//         //             {
//         //                 // Erste Diagnose Abfrage (Aufruf Telegramm)
//         //                 let mut diagnose_data: [u8; (8)] = [0; 8];
//         //                 diagnose_data[0] = ssap_data; // Ziel SAP Master
//         //                 diagnose_data[1] = dsap_data; // Quelle SAP Slave
//         //                 diagnose_data[2] = self.diagnose_status_1; // Status 1
//         //                 if DpSlaveState::Por == self.slave_state {
//         //                     diagnose_data[3] = sap_diagnose_byte2::STATUS_2_DEFAULT
//         //                         + sap_diagnose_byte2::PRM_REQ
//         //                         + 0x04; // Status 2
//         //                     diagnose_data[5] = MASTER_ADD_DEFAULT; // Adresse Master
//         //                 } else {
//         //                     diagnose_data[3] = sap_diagnose_byte2::STATUS_2_DEFAULT + 0x04; // Status 2
//         //                     diagnose_data[5] = self.master_addr - SAP_OFFSET;
//         //                     // Adresse Master
//         //                 }

//         //                 if self.watchdog_act {
//         //                     diagnose_data[3] |= sap_diagnose_byte2::WD_ON;
//         //                 }

//         //                 if self.freeze_configured {
//         //                     diagnose_data[3] |= sap_diagnose_byte2::FREEZE_MODE;
//         //                 }

//         //                 if self.sync_configured {
//         //                     diagnose_data[3] |= sap_diagnose_byte2::SYNC_MODE;
//         //                 }

//         //                 diagnose_data[4] = sap_diagnose_byte3::DIAG_SIZE_OK; // Status 3
//         //                 diagnose_data[6] = self.config.ident_high; // Ident high
//         //                 diagnose_data[7] = self.config.ident_low; // Ident low
//         //                 if self.extern_diag_para.len() > 0 {
//         //                     self.extern_diag_para[0] = sap_diagnose_ext::EXT_DIAG_GERAET
//         //                         + self.extern_diag_para.len().to_le_bytes()[0]; // Diagnose (Typ und Anzahl Bytes)
//         //                     self.codec.transmit_message_sd2(
//         //                         source_addr,
//         //                         fc_response::DATA_LOW,
//         //                         true,
//         //                         &diagnose_data[..],
//         //                         &self.extern_diag_para[..],
//         //                     );
//         //                     response = true;
//         //                 } else {
//         //                     self.codec.transmit_message_sd2(
//         //                         source_addr,
//         //                         fc_response::DATA_LOW,
//         //                         true,
//         //                         &diagnose_data[..],
//         //                         &[0; 0],
//         //                     );
//         //                     response = true;
//         //                 }
//         //             }

//         //             // Status aendern
//         //             if DpSlaveState::Por == self.slave_state {
//         //                 self.slave_state = DpSlaveState::Wrpm;
//         //             }
//         //         }

//         //         sap::SET_PRM => {
//         //             // Set Parameters Request (SSAP 62 -> DSAP 61)
//         //             // Siehe Felser 8/2009 Kap. 4.3.1

//         //             // Nach dem Erhalt der Parameter wechselt der DP-Slave vom Zustand
//         //             // "Wait Parameter" (WPRM) in den Zustand "Wait Configuration" (WCFG)
//         //             if (pdu[6] == self.config.ident_high) && (pdu[7] == self.config.ident_low) {
//         //                 self.master_addr = source_addr - SAP_OFFSET;

//         //                 if (pdu[2] & sap_set_parameter_request::ACTIVATE_WATCHDOG) != 0
//         //                 // Watchdog aktivieren
//         //                 {
//         //                     self.watchdog_act = true;
//         //                 } else {
//         //                     self.watchdog_act = false;
//         //                 }

//         //                 if (pdu[2] & sap_set_parameter_request::ACTIVATE_FREEZE) != 0 {
//         //                     self.freeze_configured = true;
//         //                 } else {
//         //                     self.freeze_configured = false;
//         //                 }

//         //                 if (pdu[2] & sap_set_parameter_request::ACTIVATE_SYNC) != 0 {
//         //                     self.sync_configured = true;
//         //                 } else {
//         //                     self.sync_configured = false;
//         //                 }

//         //                 // watchdog1 = m_pbUartRxBuffer[10];
//         //                 // watchdog2 = m_pbUartRxBuffer[11];

//         //                 self.watchdog_time = u32::from(pdu[3]) * u32::from(pdu[4]) * 10;

//         //                 if pdu[5] > 10 {
//         //                     self.min_tsdr = pdu[5] - 11;
//         //                 } else {
//         //                     self.min_tsdr = 0;
//         //                 }

//         //                 self.config.ident_high = pdu[6];
//         //                 self.config.ident_low = pdu[7];

//         //                 self.group = pdu[8]; // wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

//         //                 // TODO DPV1 etc.

//         //                 // User Parameter einlesen
//         //                 if self.user_para.len() > 0 {
//         //                     // User Parameter groesse = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
//         //                     let user_para_len: usize = usize::from(pdu_len) - 12;
//         //                     if user_para_len <= self.user_para.len() {
//         //                         for i in 0..user_para_len {
//         //                             self.user_para[i] = pdu[9 + i];
//         //                         }
//         //                     }
//         //                 }
//         //                 // Kurzquittung
//         //                 self.codec.transmit_message_sc();
//         //                 response = true;
//         //                 // m_printfunc("Quittung");
//         //                 if DpSlaveState::Wrpm == self.slave_state {
//         //                     self.slave_state = DpSlaveState::Wcfg;
//         //                 }
//         //             }
//         //         }

//         //         sap::GET_CFG => {
//         //             // Send configuration to master (SSAP 62 -> DSAP 59)
//         //             // see Felser 8/2009 chapter 4.4.3.1

//         //             if (function_code == (fc_request::REQUEST + fc_request::SRD_HIGH))
//         //                 || (function_code == (fc_request::REQUEST + fc_request::SRD_LOW))
//         //             {
//         //                 // Erste Diagnose Abfrage (Aufruf Telegramm)
//         //                 let sap_data = [ssap_data, dsap_data];
//         //                 self.codec.transmit_message_sd2(
//         //                     source_addr,
//         //                     fc_response::DATA_LOW,
//         //                     true,
//         //                     &sap_data[..],
//         //                     &self.module_config[..],
//         //                 );
//         //                 response = true;
//         //             }
//         //         }

//         //         sap::CHK_CFG => {
//         //             // Check Config Request (SSAP 62 -> DSAP 62)
//         //             // Siehe Felser 8/2009 Kap. 4.4.1

//         //             // Nach dem Erhalt der Konfiguration wechselt der DP-Slave vom Zustand
//         //             // "Wait Configuration" (WCFG) in den Zustand "Data Exchange" (DXCHG)

//         //             // IO Konfiguration:
//         //             // Kompaktes Format fuer max. 16/32 Byte IO
//         //             // Spezielles Format fuer max. 64/132 Byte IO

//         //             // Je nach PDU Datengroesse mehrere Bytes auswerten
//         //             // LE/LEr - (DA+SA+FC+DSAP+SSAP) = Anzahl Config Bytes
//         //             let config_len: usize = pdu.len() - 2;
//         //             let mut config_is_valid: bool = true;
//         //             if self.module_config.len() == config_len {
//         //                 if self.module_config.len() > 0 {
//         //                     for i in 0..config_len {
//         //                         let config_data: u8 = pdu[2 + i];
//         //                         if self.module_config[i] != config_data {
//         //                             config_is_valid = false;
//         //                         }
//         //                     }
//         //                 }
//         //                 if !config_is_valid {
//         //                     self.diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
//         //                 } else {
//         //                     self.diagnose_status_1 &= !(sap_diagnose_byte1::STATION_NOT_READY
//         //                         + sap_diagnose_byte1::CFG_FAULT);
//         //                 }
//         //             } else {
//         //                 self.diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
//         //             }

//         //             // Kurzquittung
//         //             self.codec.transmit_message_sc();
//         //             response = true;
//         //             if DpSlaveState::Wcfg == self.slave_state {
//         //                 self.slave_state = DpSlaveState::Dxchg;
//         //             }
//         //         }

//         //         _ => (),
//         //     } // Switch DSAP_data Ende
//         // }
//         // // Ziel: Slave Adresse, but no SAP
//         // else if destination_addr == self.codec.get_TS() {
//         //     // Status Abfrage

//         //     if function_code == (fc_request::REQUEST + fc_request::FDL_STATUS) {
//         //         self.codec
//         //             .transmit_message_sd1(source_addr, fc_response::FDL_STATUS_OK, false);
//         //         response = true;
//         //     }
//         //     // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
//         //     /*
//         //     else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) ||
//         //              function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
//         //     {
//         //      */
//         //     else if function_code == (fc_request::REQUEST + fc_request::SRD_HIGH)
//         //         || function_code == (fc_request::REQUEST + fc_request::SRD_LOW)
//         //     {
//         //         let output_data_len = usize::from(pdu_len - 3);

//         //         if self.sync_configured && self.sync
//         //         // write data in output_register when sync
//         //         {
//         //             if self.output_data_buffer.len() > 0 {
//         //                 if output_data_len == self.output_data_buffer.len() {
//         //                     for i in 0..output_data_len {
//         //                         self.output_data_buffer[i] = pdu[i];
//         //                     }
//         //                 }
//         //             }
//         //         } else
//         //         // normaler Betrieb
//         //         {
//         //             if self.output_data_buffer.len() > 0 {
//         //                 if output_data_len == self.output_data_buffer.len() {
//         //                     for i in 0..output_data_len {
//         //                         self.output_data_buffer[i] = pdu[i];
//         //                     }
//         //                 }
//         //             }
//         //             self.output_data = self.output_data_buffer;
//         //             self.data_handling_interface
//         //                 .data_processing(&mut [0; 0], &self.output_data[..]);
//         //         }

//         //         if !(self.freeze_configured && self.freeze)
//         //         // normaler Betrieb
//         //         {
//         //             self.data_handling_interface
//         //                 .data_processing(&mut self.input_data[..], &[0; 0]);
//         //             self.input_data_buffer = self.input_data;
//         //             if self.input_data.len() > 0 {
//         //                 // self.input_data[0] = 1;
//         //             }
//         //         }

//         //         if self.input_data_buffer.len() > 0 {
//         //             if (self.diagnose_status_1 & sap_diagnose_byte1::EXT_DIAG) != 0 {
//         //                 //TODO
//         //                 self.codec.transmit_message_sd2(
//         //                     source_addr,
//         //                     fc_response::DATA_HIGH,
//         //                     false,
//         //                     &[0; 0],
//         //                     &[0; 0],
//         //                 );
//         //                 response = true;
//         //                 // Diagnose Abfrage anfordern
//         //             } else {
//         //                 self.codec.transmit_message_sd2(
//         //                     source_addr,
//         //                     0,
//         //                     false,
//         //                     &self.input_data_buffer[..],
//         //                     &[0; 0],
//         //                 );
//         //                 response = true;
//         //             }
//         //         } else {
//         //             // TODO
//         //             //  if (diagnose_status_1 & EXT_DIAG_ || (get_Address() & 0x80))
//         //             //    sendCmd(cmd_type::SD1, DATA_HIGH, 0, &self.tx_buffer[7], 0); // Diagnose Abfrage anfordern
//         //             //  else
//         //             //    sendCmd(cmd_type::SC, 0, 0, &self.tx_buffer[7], 0); // Kurzquittung
//         //         }
//         //     }
//         // }
//         // response
//         false
//     }
// }



// pub struct PbDpSlave<
//     // 'a,
//     // Serial,
//     DataHandling,
//     // const BUF_SIZE: usize,
//     const INPUT_DATA_SIZE: usize,
//     const OUTPUT_DATA_SIZE: usize,
//     const USER_PARA_SIZE: usize,
//     const EXTERN_DIAG_PARA_SIZE: usize,
//     const MODULE_CONFIG_SIZE: usize,
// > {
//     config: Config,
//     //pub codec: Codec<'a, Serial, BUF_SIZE>,
//     data_handling_interface: DataHandling,

//     slave_state: DpSlaveState,
//     input_data: [u8; INPUT_DATA_SIZE],
//     input_data_buffer: [u8; INPUT_DATA_SIZE],
//     output_data: [u8; OUTPUT_DATA_SIZE],
//     output_data_buffer: [u8; OUTPUT_DATA_SIZE],
//     user_para: [u8; USER_PARA_SIZE],
//     extern_diag_para: [u8; EXTERN_DIAG_PARA_SIZE],
//     module_config: [u8; MODULE_CONFIG_SIZE],

//     diagnose_status_1: u8,
//     master_addr: u8,
//     group: u8,

//     source_addr: u8,
//     fcv_activated: bool,
//     fcb_last: bool,

//     freeze: bool,
//     sync: bool,
//     watchdog_act: bool,
//     min_tsdr: u8,

//     freeze_configured: bool,
//     sync_configured: bool,

//     last_connection_time: u32,
//     watchdog_time: u32,
// }

// impl<
//         // 'a,
//         // Serial,
//         DataHandling,
//         // const BUF_SIZE: usize,
//         const INPUT_DATA_SIZE: usize,
//         const OUTPUT_DATA_SIZE: usize,
//         const USER_PARA_SIZE: usize,
//         const EXTERN_DIAG_PARA_SIZE: usize,
//         const MODULE_CONFIG_SIZE: usize,
//     >
//     PbDpSlave<
//         // 'a,
//         // Serial,
//         DataHandling,
//         // BUF_SIZE,
//         INPUT_DATA_SIZE,
//         OUTPUT_DATA_SIZE,
//         USER_PARA_SIZE,
//         EXTERN_DIAG_PARA_SIZE,
//         MODULE_CONFIG_SIZE,
//     >
// where
//     // Serial: HwInterface,
//     DataHandling: DataHandlingInterface,
// {
//     pub fn new(
//         // mut serial_interface: Serial,
//         mut data_handling_interface: DataHandling,
//         mut config: Config,
//         module_config: [u8; MODULE_CONFIG_SIZE],
//     ) -> Self {
//         // let codec: Codec<Serial, BUF_SIZE> =
//         //     Codec::new(serial_interface, config.codec_config.clone());

//         let input_data = [0; INPUT_DATA_SIZE];
//         let input_data_buffer = [0; INPUT_DATA_SIZE];
//         let output_data = [0; OUTPUT_DATA_SIZE];
//         let output_data_buffer = [0; OUTPUT_DATA_SIZE];
//         let user_para = [0; USER_PARA_SIZE];
//         let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];
//         // LED Status
//         data_handling_interface.config_error_led();

//         let current_time = data_handling_interface.millis();

//         data_handling_interface.debug_write("Profi");

//         let instance = Self {
//             config,
//             //codec,
//             data_handling_interface,
//             slave_state: DpSlaveState::Por,
//             input_data,
//             input_data_buffer,
//             output_data,
//             output_data_buffer,
//             user_para,
//             extern_diag_para,
//             module_config,
//             diagnose_status_1: sap_diagnose_byte1::STATION_NOT_READY,
//             master_addr: 0xFF,
//             group: 0,
//             source_addr: 0xFF,
//             fcv_activated: false,
//             fcb_last: false,
//             freeze: false,
//             sync: false,
//             watchdog_act: false,
//             min_tsdr: 0,
//             freeze_configured: false,
//             sync_configured: false,
//             last_connection_time: current_time,
//             watchdog_time: 0xFFFFFF,
//         };
//         // instance.codec.set_fdl(&instance);
//         instance
//     }

//     pub fn access_output(&mut self) -> &mut [u8; OUTPUT_DATA_SIZE] {
//         &mut self.output_data
//     }

//     pub fn access_input(&mut self) -> &mut [u8; INPUT_DATA_SIZE] {
//         &mut self.input_data
//     }

//     // if self.watchdog_act {
//     //     if (self.data_handling_interface.millis() - self.last_connection_time)
//     //         > self.watchdog_time
//     //     {
//     //         self.output_data.fill(0);
//     //         //TODO:
//     //         // std::vector<uint8_t> unUsed;
//     //         // m_datafunc(m_outputReg, unUsed); // outputs,inputs
//     //         self.data_handling_interface
//     //             .data_processing(&mut [0; 0], &self.output_data[..]);
//     //     }
//     // }
// }

// #[allow(dead_code)]
// pub struct PbDpSlave<
// > {
// }

// impl PbDpSlave
// {
//     pub fn new()-> Self {
//         Self{}
//     }
// }

// impl Fdl for PbDpSlave
// {
//     fn handle_data_receive(
//         & self,
//         source_addr: u8,
//         destination_addr: u8,
//         function_code: u8,
//         pdu: &[u8],
//     ) -> bool {
//         true
//     }
// }