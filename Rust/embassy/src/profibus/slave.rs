use super::codec::{CodecConfig, Codec};
use super::codec_hw_interface::CodecHwInterface;
use super::data_handling_interface::DataHandlingInterface;

use super::types::{
    fc_request, fc_response, sap, sap_diagnose_byte1, sap_diagnose_byte2,
    sap_diagnose_byte3, sap_diagnose_ext, sap_global_control, sap_set_parameter_request,
    DpSlaveState,
};

const MASTER_ADD_DEFAULT: u8 = 0xFF;
const SAP_OFFSET: u8 = 128;

pub struct FdlConfig {
    ident_high: u8,
    ident_low: u8,
}

impl FdlConfig {
    #[allow(dead_code)]
    pub fn ident_high(mut self, ident_high: u8) -> Self {
        self.ident_high = ident_high;
        self
    }
    #[allow(dead_code)]
    pub fn ident_low(mut self, ident_low: u8) -> Self {
        self.ident_low = ident_low;
        self
    }
}

impl Default for FdlConfig {
    fn default() -> FdlConfig {
        FdlConfig {
            ident_high: 0,
            ident_low: 0,
        }
    }
}

pub struct ProfibusConfig {
    codec: CodecConfig,
    fdl : FdlConfig,
}

impl ProfibusConfig {
    pub fn ident_high(mut self, ident_high: u8) -> Self {
        self.fdl.ident_high = ident_high;
        self
    }
    pub fn ident_low(mut self, ident_low: u8) -> Self {
        self.fdl.ident_low = ident_low;
        self
    }

    pub fn t_s(mut self, t_s: u8) -> Self {
        self.codec.t_s = t_s;
        self
    }

    pub fn t_sl(mut self, t_sl: u16) -> Self {
        self.codec.t_sl = t_sl;
        self
    }

    pub fn t_sdr_min(mut self, t_sdr_min: u16) -> Self {
        self.codec.t_sdr_min = t_sdr_min;
        self
    }
}

impl Default for ProfibusConfig {
    fn default() -> ProfibusConfig {
        ProfibusConfig {
            codec: CodecConfig::default(),
            fdl : FdlConfig::default(),
        }
    }
}


#[allow(dead_code)]
pub struct PbDpSlave<
    SerialInterface,
    DataHandling,
    const BUF_SIZE: usize,
    const INPUT_DATA_SIZE: usize,
    const OUTPUT_DATA_SIZE: usize,
    const USER_PARA_SIZE: usize,
    const EXTERN_DIAG_PARA_SIZE: usize,
    const MODULE_CONFIG_SIZE: usize,
> {
    pub data_handling_interface: DataHandling,
    pub tx_buffer: [u8; BUF_SIZE],

    pub codec: Codec<SerialInterface>,

    fdl : FdlConfig,

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
        SerialInterface,
        DataHandling,
        const BUF_SIZE: usize,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const MODULE_CONFIG_SIZE: usize,
    >
    PbDpSlave<
        SerialInterface,
        DataHandling,
        BUF_SIZE,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    SerialInterface: CodecHwInterface,
    DataHandling: DataHandlingInterface,
{
    pub fn new(
        mut hw_interface: SerialInterface,
        mut data_handling_interface: DataHandling,
        config: ProfibusConfig,
        module_config: [u8; MODULE_CONFIG_SIZE],
    ) -> Self {
        // let input_data = Vec::<u8, InputDatalen>::new();
        // let output_data = Vec::<u8, OutputDatalen>::new();
        // let user_para = Vec::<u8, UserParalen>::new();
        // let extern_diag_para = Vec::<u8, ExternDiagParalen>::new();
        // let vendor_data = Vec::<u8, VendorDatalen>::new();

        let input_data = [0; INPUT_DATA_SIZE];
        let input_data_buffer = [0; INPUT_DATA_SIZE];
        let output_data = [0; OUTPUT_DATA_SIZE];
        let output_data_buffer = [0; OUTPUT_DATA_SIZE];
        let user_para = [0; USER_PARA_SIZE];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];

        let mut codec = Codec::<SerialInterface>::new(
            hw_interface,
            config.codec,
        );

        let current_time = data_handling_interface.millis();

        Self {
            data_handling_interface,
            tx_buffer: [0; BUF_SIZE],
            codec,
            fdl: config.fdl,
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
        }
    }

    pub fn access_output(&mut self) -> &mut [u8; OUTPUT_DATA_SIZE] {
        &mut self.output_data
    }

    pub fn access_input(&mut self) -> &mut [u8; INPUT_DATA_SIZE] {
        &mut self.input_data
    }

    pub async fn fdl_handle_data(
        &mut self,
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        pdu: &[u8],
    ) -> bool {
        let mut response: bool = false;
        self.last_connection_time = self.data_handling_interface.millis(); // letzte Zeit eines Telegramms sichern

                if (function_code & 0x30) == fc_request::FCB
                // Startbedingung
                {
                    self.fcv_activated = true;
                    self.fcb_last = true;
                } else if self.fcv_activated {
                    if source_addr != self.source_addr {
                        // new address so fcv is deactivated
                        self.fcv_activated = false;
                    } else if ((function_code & fc_request::FCB) != 0) == self.fcb_last {
                        // FCB is identical, repeat message
                        response = true;
                        self.transmit().await;
                    } else {
                        // save new FCB bit
                        self.fcb_last = !self.fcb_last;
                    }
                } else
                // wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
                {
                    self.fcv_activated = false;
                }

                // letzte Adresse sichern
                self.source_addr = source_addr;

                // Service Access Point erkannt?
                if ((destination_addr & 0x80) != 0) && ((source_addr & 0x80) != 0) {
                    let dsap_data = pdu[0]; // sap destination
                    let ssap_data = pdu[1]; // sap source

                    // Ablauf Reboot:
                    // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
                    // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
                    // 3) SSAP 62 -> DSAP 62 (Check Config Request)
                    // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
                    // 5) Data Exchange Request (normaler Zyklus)

                    // Siehe Felser 8/2009 Kap. 4.1
                    match dsap_data {
                        sap::SET_SLAVE_ADR => {
                            // Set Slave Address (SSAP 62 -> DSAP 55)
                            // Siehe Felser 8/2009 Kap. 4.2
                            if DpSlaveState::Wrpm == self.slave_state {
                                if 6 == pdu.len() {
                                    self.codec.config.t_s = pdu[2];
                                    self.fdl.ident_high = pdu[3];
                                    self.fdl.ident_low = pdu[4];
                                    //TODO
                                    // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
                                    // trigger value saving
                                }
                            }
                            response = true;
                            self.transmit_message_sc();
                        }

                        sap::GLOBAL_CONTROL => {
                            // Global Control Request (SSAP 62 -> DSAP 58)
                            // Siehe Felser 8/2009 Kap. 4.6.2

                            // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
                            if (pdu[2] & sap_global_control::CLEAR_DATA) != 0 {
                                self.data_handling_interface.error_led_on(); // Status "SPS nicht bereit"
                            } else {
                                self.data_handling_interface.error_led_off(); // Status "SPS OK"
                            }

                            // Gruppe berechnen
                            // for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;

                            // Wenn Befehl fuer uns ist
                            if (pdu[3] & self.group) != 0
                            //(cnt == group)
                            {
                                if (pdu[2] & sap_global_control::UNFREEZE) != 0 {
                                    // FREEZE Zustand loeschen
                                    self.freeze = false;
                                    self.data_handling_interface
                                        .data_processing(&mut self.input_data[..], &[0; 0]);
                                    //TODO: only a copy is given
                                    self.input_data_buffer = self.input_data;
                                } else if (pdu[2] & sap_global_control::UNSYNC) != 0 {
                                    // SYNC Zustand loeschen
                                    self.sync = false;
                                    self.output_data = self.output_data_buffer;
                                    self.data_handling_interface
                                        .data_processing(&mut [0; 0], &self.output_data[..]);
                                } else if (pdu[2] & sap_global_control::FREEZE) != 0 {
                                    // Eingaenge nicht mehr neu einlesen
                                    self.freeze = true;
                                    self.data_handling_interface
                                        .data_processing(&mut self.input_data[..], &[0; 0]);
                                    //TODO: only a copy is given
                                    self.input_data_buffer = self.input_data;
                                } else if (pdu[2] & sap_global_control::SYNC) != 0 {
                                    // Ausgaenge nur bei SYNC Befehl setzen
                                    self.sync = true;
                                    self.output_data = self.output_data_buffer;
                                    self.data_handling_interface
                                        .data_processing(&mut [0; 0], &self.output_data[..]);
                                }
                            }
                        }

                        sap::SLAVE_DIAGNOSTIC => {
                            // Get Diagnostics Request (SSAP 62 -> DSAP 60)
                            // Siehe Felser 8/2009 Kap. 4.5.2

                            // Nach dem Erhalt der Diagnose wechselt der DP-Slave vom Zustand
                            // "Power on Reset" (POR) in den Zustand "Wait Parameter" (WPRM)

                            // Am Ende der Initialisierung (Zustand "Data Exchange" (DXCHG))
                            // sendet der Master ein zweites mal ein Diagnostics Request um die
                            // korrekte Konfiguration zu pruefen
                            // m_printfunc((int)function_code);
                            // m_printfunc(REQUEST_ + SRD_HIGH);
                            if (function_code == (fc_request::REQUEST + fc_request::SRD_HIGH))
                                || (function_code == (fc_request::REQUEST + fc_request::SRD_LOW))
                            {
                                // Erste Diagnose Abfrage (Aufruf Telegramm)
                                let mut diagnose_data: [u8; 8] = [0; 8];
                                diagnose_data[0] = ssap_data; // Ziel SAP Master
                                diagnose_data[1] = dsap_data; // Quelle SAP Slave
                                diagnose_data[2] = self.diagnose_status_1; // Status 1
                                if DpSlaveState::Por == self.slave_state {
                                    diagnose_data[3] = sap_diagnose_byte2::STATUS_2_DEFAULT
                                        + sap_diagnose_byte2::PRM_REQ
                                        + 0x04; // Status 2
                                    diagnose_data[5] = MASTER_ADD_DEFAULT; // Adresse Master
                                } else {
                                    diagnose_data[3] = sap_diagnose_byte2::STATUS_2_DEFAULT + 0x04; // Status 2
                                    diagnose_data[5] = self.master_addr - SAP_OFFSET;
                                    // Adresse Master
                                }

                                if self.watchdog_act {
                                    diagnose_data[3] |= sap_diagnose_byte2::WD_ON;
                                }

                                if self.freeze_configured {
                                    diagnose_data[3] |= sap_diagnose_byte2::FREEZE_MODE;
                                }

                                if self.sync_configured {
                                    diagnose_data[3] |= sap_diagnose_byte2::SYNC_MODE;
                                }

                                diagnose_data[4] = sap_diagnose_byte3::DIAG_SIZE_OK; // Status 3
                                diagnose_data[6] = self.fdl.ident_high; // Ident high
                                diagnose_data[7] = self.fdl.ident_low; // Ident low
                                if self.extern_diag_para.len() > 0 {
                                    self.extern_diag_para[0] = sap_diagnose_ext::EXT_DIAG_GERAET
                                        + self.extern_diag_para.len().to_le_bytes()[0]; // Diagnose (Typ und Anzahl Bytes)
                                    let mut buf: [u8; EXTERN_DIAG_PARA_SIZE] =
                                        [0; EXTERN_DIAG_PARA_SIZE];
                                    buf.copy_from_slice(&self.extern_diag_para[..]);
                                    self.transmit_message_sd2(
                                        source_addr,
                                        fc_response::DATA_LOW,
                                        true,
                                        &diagnose_data[..],
                                        &buf,
                                    ).await;
                                    response = true;
                                } else {
                                    self.transmit_message_sd2(
                                        source_addr,
                                        fc_response::DATA_LOW,
                                        true,
                                        &diagnose_data[..],
                                        &[0; 0],
                                    ).await;
                                    response = true;
                                }
                            }

                            // Status aendern
                            if DpSlaveState::Por == self.slave_state {
                                self.slave_state = DpSlaveState::Wrpm;
                            }
                        }

                        sap::SET_PRM => {
                            // Set Parameters Request (SSAP 62 -> DSAP 61)
                            // Siehe Felser 8/2009 Kap. 4.3.1

                            // Nach dem Erhalt der Parameter wechselt der DP-Slave vom Zustand
                            // "Wait Parameter" (WPRM) in den Zustand "Wait Configuration" (WCFG)
                            if (pdu[6] == self.fdl.ident_high)
                                && (pdu[7] == self.fdl.ident_low)
                            {
                                self.master_addr = source_addr - SAP_OFFSET;

                                if (pdu[2] & sap_set_parameter_request::ACTIVATE_WATCHDOG)
                                    != 0
                                // Watchdog aktivieren
                                {
                                    self.watchdog_act = true;
                                } else {
                                    self.watchdog_act = false;
                                }

                                if (pdu[2] & sap_set_parameter_request::ACTIVATE_FREEZE) != 0
                                {
                                    self.freeze_configured = true;
                                } else {
                                    self.freeze_configured = false;
                                }

                                if (pdu[2] & sap_set_parameter_request::ACTIVATE_SYNC) != 0 {
                                    self.sync_configured = true;
                                } else {
                                    self.sync_configured = false;
                                }

                                // watchdog1 = m_pbUartRxBuffer[10];
                                // watchdog2 = m_pbUartRxBuffer[11];

                                self.watchdog_time =
                                    u32::from(pdu[3]) * u32::from(pdu[4]) * 10;

                                if pdu[5] > 10 {
                                    self.min_tsdr = pdu[5] - 11;
                                } else {
                                    self.min_tsdr = 0;
                                }
                                self.group = pdu[8]; // wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

                                // TODO DPV1 etc.

                                // User Parameter einlesen
                                if self.user_para.len() > 0 {
                                    // User Parameter groesse = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
                                    let user_para_len: usize = usize::from(pdu.len()) - 5;
                                    if user_para_len <= self.user_para.len() {
                                        for i in 0..user_para_len {
                                            self.user_para[i] = pdu[9 + i];
                                        }
                                    }
                                }
                                // Kurzquittung
                                self.transmit_message_sc().await;
                                response = true;
                                // m_printfunc("Quittung");
                                if DpSlaveState::Wrpm == self.slave_state {
                                    self.slave_state = DpSlaveState::Wcfg;
                                }
                            }
                        }

                        sap::GET_CFG => {
                            // Send configuration to master (SSAP 62 -> DSAP 59)
                            // see Felser 8/2009 chapter 4.4.3.1

                            if (function_code == (fc_request::REQUEST + fc_request::SRD_HIGH))
                                || (function_code == (fc_request::REQUEST + fc_request::SRD_LOW))
                            {
                                // Erste Diagnose Abfrage (Aufruf Telegramm)
                                let sap_data = [ssap_data, dsap_data];
                                //TODO
                                let mut buf: [u8; MODULE_CONFIG_SIZE] = [0; MODULE_CONFIG_SIZE];
                                buf.copy_from_slice(&self.module_config[..]);
                                self.transmit_message_sd2(
                                    source_addr,
                                    fc_response::DATA_LOW,
                                    true,
                                    &sap_data[..],
                                    &buf,
                                ).await;
                                response = true;
                            }
                        }

                        sap::CHK_CFG => {
                            // Check Config Request (SSAP 62 -> DSAP 62)
                            // Siehe Felser 8/2009 Kap. 4.4.1

                            // Nach dem Erhalt der Konfiguration wechselt der DP-Slave vom Zustand
                            // "Wait Configuration" (WCFG) in den Zustand "Data Exchange" (DXCHG)

                            // IO Konfiguration:
                            // Kompaktes Format fuer max. 16/32 Byte IO
                            // Spezielles Format fuer max. 64/132 Byte IO

                            // Je nach PDU Datengroesse mehrere Bytes auswerten
                            // LE/LEr - (DA+SA+FC+DSAP+SSAP) = Anzahl Config Bytes
                            //TODO
                            let config_len: usize = pdu.len() - 2;
                            let mut config_is_valid: bool = true;
                            if self.module_config.len() == config_len {
                                if self.module_config.len() > 0 {
                                    for i in 0..config_len {
                                        let config_data: u8 = pdu[2 + i];
                                        if self.module_config[i] != config_data {
                                            config_is_valid = false;
                                        }
                                    }
                                }
                                if !config_is_valid {
                                    self.diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
                                } else {
                                    self.diagnose_status_1 &= !(sap_diagnose_byte1::STATION_NOT_READY
                                        + sap_diagnose_byte1::CFG_FAULT);
                                }
                            } else {
                                self.diagnose_status_1 |= sap_diagnose_byte1::CFG_FAULT;
                            }

                            // Kurzquittung
                            self.transmit_message_sc().await;
                            response = true;
                            if DpSlaveState::Wcfg == self.slave_state {
                                self.slave_state = DpSlaveState::Dxchg;
                            }
                        }

                        _ => (),
                    } // Switch DSAP_data Ende
                }
                // Ziel: Slave Adresse, but no SAP
                else if destination_addr == self.codec.config.t_s {
                    // Status Abfrage

                    if function_code == (fc_request::REQUEST + fc_request::FDL_STATUS) {
                        self.transmit_message_sd1(source_addr, fc_response::FDL_STATUS_OK, false).await;
                        response = true;
                    }
                    // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
                    /*
                    else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) ||
                             function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
                    {
                     */
                    else if function_code == (fc_request::REQUEST + fc_request::SRD_HIGH)
                        || function_code == (fc_request::REQUEST + fc_request::SRD_LOW)
                    {   //TODO
                        let output_data_len = pdu.len();

                        if self.sync_configured && self.sync
                        // write data in output_register when sync
                        {
                            if self.output_data_buffer.len() > 0 {
                                if output_data_len == self.output_data_buffer.len() {
                                    for i in 0..output_data_len {
                                        self.output_data_buffer[i] = pdu[i];
                                    }
                                }
                            }
                        } else
                        // normaler Betrieb
                        {
                            if self.output_data_buffer.len() > 0 {
                                if output_data_len == self.output_data_buffer.len() {
                                    for i in 0..output_data_len {
                                        self.output_data_buffer[i] = pdu[i];
                                    }
                                }
                            }
                            self.output_data = self.output_data_buffer;
                            self.data_handling_interface
                                .data_processing(&mut [0; 0], &self.output_data[..]);
                        }

                        if !(self.freeze_configured && self.freeze)
                        // normaler Betrieb
                        {
                            self.data_handling_interface
                                .data_processing(&mut self.input_data[..], &[0; 0]);
                            self.input_data_buffer = self.input_data;
                            if self.input_data.len() > 0 {
                                // self.input_data[0] = 1;
                            }
                        }

                        if self.input_data_buffer.len() > 0 {
                            if (self.diagnose_status_1 & sap_diagnose_byte1::EXT_DIAG) != 0 {
                                //TODO
                                self.transmit_message_sd2(source_addr, fc_response::DATA_HIGH, false, &[0; 0], &[0; 0]).await;
                                response = true;
                                // Diagnose Abfrage anfordern
                            } else {
                                let mut buf: [u8; INPUT_DATA_SIZE] = [0; INPUT_DATA_SIZE];
                                buf.copy_from_slice(&self.input_data_buffer[..]);
                                self.transmit_message_sd2(source_addr, fc_response::DATA_LOW, false, &buf, &[0; 0]).await;
                                response = true;
                            }
                        } else {
                            // TODO
                            //  if (diagnose_status_1 & EXT_DIAG_ || (get_Address() & 0x80))
                            //    sendCmd(cmd_type::SD1, DATA_HIGH, 0, &self.tx_buffer[7], 0); // Diagnose Abfrage anfordern
                            //  else
                            //    sendCmd(cmd_type::SC, 0, 0, &self.tx_buffer[7], 0); // Kurzquittung
                        }
                    }
                }
                    response
    }

    pub(super) fn fdl_timer_call(&mut self) {
        if self.watchdog_act {
            if (self.data_handling_interface.millis() - self.last_connection_time)
                > self.watchdog_time
            {
                self.output_data.fill(0);
                //TODO:
                // std::vector<uint8_t> unUsed;
                // m_datafunc(m_outputReg, unUsed); // outputs,inputs
                self.data_handling_interface
                    .data_processing(&mut [0; 0], &self.output_data[..]);
            }
        }
    }
}
