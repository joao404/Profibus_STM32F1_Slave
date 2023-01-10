use super::hwinterface::HwInterface;
use super::types::{
    cmd_type, fc_request, fc_response, sap, sap_diagnose_byte1, sap_diagnose_byte2,
    sap_diagnose_byte3, sap_diagnose_ext, sap_global_control, sap_set_parameter_request,
    DpSlaveState, StreamState,
};

#[derive(PartialEq, Eq)]
pub enum UartAccess {
    SingleByte,
    Dma,
}

#[derive(PartialEq, Eq)]
pub enum ReceiveHandling {
    Interrupt,
    Thread,
}

pub struct Config {
    ident_high: u8,
    ident_low: u8,
    addr: u8,
    counter_frequency: u32,
    baudrate: u32,
    rx_handling: UartAccess,
    tx_handling: UartAccess,
    receive_handling: ReceiveHandling,
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

    pub fn rx_handling(mut self, rx_handling: UartAccess) -> Self {
        self.rx_handling = rx_handling;
        self
    }

    pub fn tx_handling(mut self, tx_handling: UartAccess) -> Self {
        self.tx_handling = tx_handling;
        self
    }

    pub fn receive_handling(mut self, receive_handling: ReceiveHandling) -> Self {
        self.receive_handling = receive_handling;
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
            rx_handling: UartAccess::SingleByte,
            tx_handling: UartAccess::SingleByte,
            receive_handling: ReceiveHandling::Interrupt,
        }
    }
}

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;
const MASTER_ADD_DEFAULT: u8 = 0xFF;

#[allow(dead_code)]
pub struct PbDpSlave<
    T,
    const BUF_SIZE: usize,
    const INPUT_DATA_SIZE: usize,
    const OUTPUT_DATA_SIZE: usize,
    const USER_PARA_SIZE: usize,
    const EXTERN_DIAG_PARA_SIZE: usize,
    const MODULE_CONFIG_SIZE: usize,
> {
    config: Config,
    interface: T,
    tx_buffer: [u8; BUF_SIZE],
    rx_buffer: [u8; BUF_SIZE],

    rx_len: usize,
    tx_len: usize,
    tx_pos: usize,

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
    module_config: [u8; MODULE_CONFIG_SIZE],

    diagnose_status_1: u8,
    master_addr: u8,
    group: u8,

    source_add_last: u8,
    fcv_act: bool,
    fcb_last: bool,

    freeze: bool,
    sync: bool,
    watchdog_act: bool,
    min_tsdr: u8,

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
        const MODULE_CONFIG_SIZE: usize,
    >
    PbDpSlave<
        T,
        BUF_SIZE,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    T: HwInterface,
{
    pub fn new(
        mut interface: T,
        mut config: Config,
        module_config: [u8; MODULE_CONFIG_SIZE],
    ) -> Self {
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

        let input_data = [0; INPUT_DATA_SIZE];
        let output_data = [0; OUTPUT_DATA_SIZE];
        let user_para = [0; USER_PARA_SIZE];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];

        // Timer init
        interface.config_timer();
        // LED Status
        interface.config_error_led();
        // Pin Init
        interface.config_rs485_pin();

        // Uart Init
        interface.config_uart();
        interface.run_timer(timeout_max_syn_time_in_us);
        interface.rx_rs485_enable();
        interface.activate_rx_interrupt();

        let current_time = interface.millis();

        Self {
            config,
            interface,
            tx_buffer: [0; BUF_SIZE],
            rx_buffer: [0; BUF_SIZE],
            rx_len: 0,
            tx_len: 0,
            tx_pos: 0,
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
            module_config,
            diagnose_status_1: sap_diagnose_byte1::STATION_NOT_READY,
            master_addr: 0xFF,
            group: 0,
            source_add_last: 0xFF,
            fcv_act: false,
            fcb_last: false,
            freeze: false,
            sync: false,
            watchdog_act: false,
            min_tsdr: 0,
            freeze_act: false,
            sync_act: false,
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

    pub fn serial_interrupt_handler(&mut self) {
        if self.interface.rx_data_received() {
            self.rx_interrupt_handler();
        } else if self.interface.tx_data_send() {
            self.tx_interrupt_handler();
        }
    }

    pub fn rx_interrupt_handler(&mut self) {
        self.interface.stop_timer();
        loop {
            match self.interface.get_uart_value() {
                Some(data) => {
                    self.rx_buffer[self.rx_len] = data;
                    // if we waited for TSYN, data can be saved
                    if StreamState::WaitData == self.stream_state {
                        self.stream_state = StreamState::GetData;
                    }

                    // Einlesen erlaubt?
                    if StreamState::GetData == self.stream_state {
                        if self.rx_len < self.rx_buffer.len() {
                            self.rx_len += 1;
                        }
                    }
                }
                None => break,
            }
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn tx_interrupt_handler(&mut self) {
        self.interface.stop_timer();
        if self.config.tx_handling == UartAccess::SingleByte {
            if self.tx_pos < self.tx_len {
                // TX Buffer fuellen
                self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                self.tx_pos += 1;
                // m_printfunc(m_txCnt);
            } else {
                self.interface.tx_rs485_disable();
                // Alles gesendet, Interrupt wieder aus
                self.interface.deactivate_tx_interrupt();
                // clear Flag because we are not writing to buffer
                self.interface.clear_tx_flag();
            }
        } else if self.config.tx_handling == UartAccess::Dma {
            self.interface.tx_rs485_disable();
            // Alles gesendet, Interrupt wieder aus
            self.interface.deactivate_tx_interrupt();
            // clear Flag because we are not writing to buffer
            self.interface.clear_tx_flag();
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn timer_interrupt_handler(&mut self) {
        // Timer A Stop
        self.interface.stop_timer();
        // self.interface.serial_write(b'c');

        match self.stream_state {
            StreamState::WaitSyn => {
                self.stream_state = StreamState::WaitData;
                self.rx_len = 0;
                self.interface.rx_rs485_enable(); // Auf Receive umschalten
                self.timer_timeout_in_us = self.timeout_max_sdr_time_in_us;
            }
            StreamState::GetData => {
                if self.config.receive_handling == ReceiveHandling::Interrupt {
                    self.stream_state = StreamState::WaitSyn;
                    self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                    self.interface.deactivate_rx_interrupt();
                    self.handle_data_receive();
                    self.interface.activate_rx_interrupt();
                } else if self.config.receive_handling == ReceiveHandling::Thread {
                    self.stream_state = StreamState::HandleData;
                    self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                    self.interface.deactivate_rx_interrupt();
                    self.interface.schedule_receive_handling();
                }
            }
            StreamState::WaitMinTsdr => {
                self.stream_state = StreamState::SendData;
                self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
                self.interface.wait_for_activ_transmission();
                self.interface.tx_rs485_enable();
                self.interface.activate_tx_interrupt();
                self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                self.tx_pos += 1;
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
                self.interface
                    .data_processing(&mut [0; 0], &self.output_data[..]);
            }
        }

        self.interface.run_timer(self.timer_timeout_in_us);
    }

    fn transmit_message_sd1(&mut self, function_code: u8, sap_offset: u8) {
        self.tx_buffer[0] = cmd_type::SD1;
        self.tx_buffer[1] = self.master_addr;
        self.tx_buffer[2] = self.config.addr + sap_offset;
        self.tx_buffer[3] = function_code;
        let checksum = self.calc_checksum(&self.tx_buffer[1..4]);
        self.tx_buffer[4] = checksum;
        self.tx_buffer[5] = cmd_type::ED;
        self.tx_len = 6;
        self.transmit();
    }

    fn transmit_message_sd2(
        &mut self,
        function_code: u8,
        sap_offset: u8,
        pdu1: &[u8],
        pdu2: &[u8],
    ) {
        self.tx_buffer[0] = cmd_type::SD2;
        self.tx_buffer[1] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
        self.tx_buffer[2] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
        self.tx_buffer[3] = cmd_type::SD2;
        self.tx_buffer[4] = self.master_addr;
        self.tx_buffer[5] = self.config.addr + sap_offset;
        self.tx_buffer[6] = function_code;
        if pdu1.len() > 0 {
            for i in 0..pdu1.len() {
                self.tx_buffer[7 + i] = pdu1[i];
            }
        }
        if pdu2.len() > 0 {
            for i in 0..pdu2.len() {
                self.tx_buffer[7 + i + pdu1.len()] = pdu2[i];
            }
        }
        let checksum = self.calc_checksum(&self.tx_buffer[4..7])
            + self.calc_checksum(pdu1)
            + self.calc_checksum(pdu2);
        self.tx_buffer[7 + pdu1.len() + pdu2.len()] = checksum;
        self.tx_buffer[8 + pdu1.len() + pdu2.len()] = cmd_type::ED;
        self.tx_len = 9 + pdu1.len() + pdu2.len();
        self.transmit();
    }
    #[allow(dead_code)]
    fn transmit_message_sd3(&mut self, function_code: u8, sap_offset: u8, pdu: &[u8; 8]) {
        self.tx_buffer[0] = cmd_type::SD3;
        self.tx_buffer[1] = self.master_addr;
        self.tx_buffer[2] = self.config.addr + sap_offset;
        self.tx_buffer[3] = function_code;
        for i in 0..pdu.len() {
            self.tx_buffer[4 + i] = pdu[i];
        }
        let checksum = self.calc_checksum(&self.tx_buffer[1..12]);
        self.tx_buffer[12] = checksum;
        self.tx_buffer[13] = cmd_type::ED;
        self.tx_len = 14;
        self.transmit();
    }
    #[allow(dead_code)]
    fn transmit_message_sd4(&mut self, sap_offset: u8) {
        self.tx_buffer[0] = cmd_type::SD4;
        self.tx_buffer[1] = self.master_addr;
        self.tx_buffer[2] = self.config.addr + sap_offset;
        self.tx_len = 3;
        self.transmit();
    }

    fn transmit_message_sc(&mut self) {
        self.tx_buffer[0] = cmd_type::SC;
        self.tx_len = 1;
        self.transmit();
    }

    fn transmit(&mut self) {
        self.interface.stop_timer();
        self.tx_pos = 0;
        if 0 != self.min_tsdr {
            self.stream_state = StreamState::WaitMinTsdr;
            self.timer_timeout_in_us = (self.config.counter_frequency * u32::from(self.min_tsdr))
                / self.config.baudrate
                / 2u32;
            self.interface.run_timer(self.timer_timeout_in_us);
        } else {
            self.stream_state = StreamState::SendData;
            self.interface.wait_for_activ_transmission();
            self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
            // activate Send Interrupt
            self.interface.tx_rs485_enable();
            self.interface.clear_tx_flag();
            if self.config.tx_handling == UartAccess::SingleByte {
                self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                self.interface.activate_tx_interrupt();
                self.tx_pos += 1;
                self.interface.run_timer(self.timer_timeout_in_us);
            } else if self.config.tx_handling == UartAccess::Dma {
                self.interface.send_uart_data(&self.tx_buffer);
                self.interface.activate_tx_interrupt();
            }
        }
    }

    fn calc_checksum(&self, data: &[u8]) -> u8 {
        let mut checksum: u8 = 0;
        for x in data {
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

    pub fn handle_data_receive(&mut self) {
        let mut process_data = false;

        // Profibus Datentypen
        let mut destination_add: u8 = 0;
        let mut source_add: u8 = 0;
        let mut function_code: u8 = 0;
        let mut pdu_len: u8 = 0; // PDU Groesse

        match self.rx_buffer[0] {
            cmd_type::SD1 => {
                if 6 == self.rx_len {
                    if cmd_type::ED == self.rx_buffer[5] {
                        destination_add = self.rx_buffer[1];
                        source_add = self.rx_buffer[2];
                        function_code = self.rx_buffer[3];
                        let fcs_data = self.rx_buffer[4]; // Frame Check Sequence

                        if self.check_destination_addr(destination_add) {
                            if fcs_data == self.calc_checksum(&self.rx_buffer[1..4]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                process_data = true;
                            }
                        }
                    }
                }
            }

            cmd_type::SD2 => {
                if self.rx_len > 4 {
                    if self.rx_len == usize::from(self.rx_buffer[1] + 6) {
                        if cmd_type::ED == self.rx_buffer[self.rx_len - 1] {
                            pdu_len = self.rx_buffer[1]; // DA+SA+FC+Nutzdaten
                            destination_add = self.rx_buffer[4];
                            source_add = self.rx_buffer[5];
                            function_code = self.rx_buffer[6];
                            let fcs_data = self.rx_buffer[usize::from(pdu_len + 4)]; // Frame Check Sequence
                            if self.check_destination_addr(destination_add) {
                                if fcs_data
                                    == self.calc_checksum(
                                        &self.rx_buffer[4..usize::from(self.rx_len - 2)],
                                    )
                                {
                                    // FCV und FCB loeschen, da vorher überprüft
                                    function_code &= 0xCF;
                                    process_data = true;
                                }
                            }
                        }
                    }
                }
            }

            cmd_type::SD3 => {
                if 14 == self.rx_len {
                    if cmd_type::ED == self.rx_buffer[13] {
                        pdu_len = 11; // DA+SA+FC+Nutzdaten
                        destination_add = self.rx_buffer[1];
                        source_add = self.rx_buffer[2];
                        function_code = self.rx_buffer[3];
                        let fcs_data = self.rx_buffer[12]; // Frame Check Sequence

                        if self.check_destination_addr(destination_add) {
                            if fcs_data == self.calc_checksum(&self.rx_buffer[1..12]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                process_data = true;
                            }
                        }
                    }
                }
            }

            cmd_type::SD4 => {
                if 3 == self.rx_len {
                    destination_add = self.rx_buffer[1];
                    source_add = self.rx_buffer[2];

                    if self.check_destination_addr(destination_add) {
                        //TODO
                    }
                }
            }

            _ => (),
        } // match self.buffer[0]

        if process_data {
            self.last_connection_time = self.interface.millis(); // letzte Zeit eines Telegramms sichern

            self.master_addr = source_add; // Master Adresse ist Source Adresse

            if (function_code & 0x30) == fc_request::FCB
            // Startbedingung
            {
                self.fcv_act = true;
                self.fcb_last = true;
            } else if self.fcv_act {
                // Adresse wie vorher?
                if source_add != self.source_add_last {
                    // neue Verbindung und damit FCV ungültig
                    self.fcv_act = false;
                } else if ((function_code & fc_request::FCB) != 0) == self.fcb_last
                // FCB ist gleich geblieben
                {
                    // Nachricht wiederholen
                    self.transmit();
                    // die Nachricht liegt noch im Speicher
                } else
                // Speichern des neuen FCB
                {
                    self.fcb_last = !self.fcb_last; // das negierte bit speichern, da sonst die vorherige Bedingung angeschlagen hätte
                }
            } else
            // wenn es keine Startbedingung gibt und wir nicht eingeschaltet sind, können wir fcv ausschalten
            {
                self.fcv_act = false;
            }

            // letzte Adresse sichern
            self.source_add_last = source_add;

            // Service Access Point erkannt?
            if ((destination_add & 0x80) != 0) && ((source_add & 0x80) != 0) {
                let dsap_data = self.rx_buffer[7]; // sap destination
                let ssap_data = self.rx_buffer[8]; // sap source

                // Ablauf Reboot:
                // 1) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
                // 2) SSAP 62 -> DSAP 61 (Set Parameters Request)
                // 3) SSAP 62 -> DSAP 62 (Check Config Request)
                // 4) SSAP 62 -> DSAP 60 (Get Diagnostics Request)
                // 5) Data Exchange Request (normaler Zyklus)

                // Siehe Felser 8/2009 Kap. 4.1
                // m_printfunc((int)DSAP_data);
                match dsap_data {
                    sap::SET_SLAVE_ADR => {
                        // Set Slave Address (SSAP 62 -> DSAP 55)
                        // Siehe Felser 8/2009 Kap. 4.2

                        // Nur im Zustand "Wait Parameter" (WPRM) moeglich

                        if DpSlaveState::Wrpm == self.slave_state {
                            // adresse ändern
                            // new_addr = pb_uart_buffer[9];
                            // IDENT_HIGH_BYTE = m_pbUartRxBuffer[10];
                            // IDENT_LOW_BYTE = m_pbUartRxBuffer[11];
                            // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
                        }

                        self.transmit_message_sc();
                    }

                    sap::GLOBAL_CONTROL => {
                        // Global Control Request (SSAP 62 -> DSAP 58)
                        // Siehe Felser 8/2009 Kap. 4.6.2

                        // Wenn "Clear Data" high, dann SPS CPU auf "Stop"
                        if (self.rx_buffer[9] & sap_global_control::CLEAR_DATA) != 0 {
                            self.interface.error_led_on(); // Status "SPS nicht bereit"
                        } else {
                            self.interface.error_led_off(); // Status "SPS OK"
                        }

                        // Gruppe berechnen
                        // for (cnt = 0;  pb_uart_buffer[10] != 0; cnt++) pb_uart_buffer[10]>>=1;

                        // Wenn Befehl fuer uns ist
                        if (self.rx_buffer[10] & self.group) != 0
                        //(cnt == group)
                        {
                            if (self.rx_buffer[9] & sap_global_control::UNFREEZE) != 0 {
                                // FREEZE Zustand loeschen
                                self.freeze = false;
                                // m_datafunc(NULL,&(self.tx_buffer[7]));//outputs,inputs
                            } else if (self.rx_buffer[9] & sap_global_control::UNSYNC) != 0 {
                                // SYNC Zustand loeschen
                                self.sync = false;
                                self.interface
                                    .data_processing(&mut [0; 0], &self.output_data[..]);
                            } else if (self.rx_buffer[9] & sap_global_control::FREEZE) != 0 {
                                // Eingaenge nicht mehr neu einlesen
                                if self.freeze {
                                    self.interface
                                        .data_processing(&mut self.input_data[..], &[0; 0]);
                                }
                                self.freeze = true;
                            } else if (self.rx_buffer[9] & sap_global_control::SYNC) != 0 {
                                // Ausgaenge nur bei SYNC Befehl setzen

                                if self.sync {
                                    self.interface
                                        .data_processing(&mut [0; 0], &self.output_data[..]);
                                }
                                self.sync = true;
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
                            let mut diagnose_data: [u8; (8)] = [0; 8];
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

                            if self.freeze_act {
                                diagnose_data[3] |= sap_diagnose_byte2::FREEZE_MODE;
                            }

                            if self.sync_act {
                                diagnose_data[3] |= sap_diagnose_byte2::SYNC_MODE;
                            }

                            diagnose_data[4] = sap_diagnose_byte3::DIAG_SIZE_OK; // Status 3
                            diagnose_data[6] = self.config.ident_high; // Ident high
                            diagnose_data[7] = self.config.ident_low; // Ident low
                            if self.extern_diag_para.len() > 0 {
                                self.extern_diag_para[0] = sap_diagnose_ext::EXT_DIAG_GERAET
                                    + self.extern_diag_para.len().to_le_bytes()[0]; // Diagnose (Typ und Anzahl Bytes)
                                let mut buf: [u8; EXTERN_DIAG_PARA_SIZE] =
                                    [0; EXTERN_DIAG_PARA_SIZE];
                                buf.copy_from_slice(&self.extern_diag_para[..]);
                                self.transmit_message_sd2(
                                    fc_response::DATA_LOW,
                                    SAP_OFFSET,
                                    &diagnose_data[..],
                                    &buf,
                                );
                            } else {
                                self.transmit_message_sd2(
                                    fc_response::DATA_LOW,
                                    SAP_OFFSET,
                                    &diagnose_data[..],
                                    &[0; 0],
                                );
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
                        if (self.rx_buffer[13] == self.config.ident_high)
                            && (self.rx_buffer[14] == self.config.ident_low)
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

                            if (self.rx_buffer[9] & sap_set_parameter_request::ACTIVATE_WATCHDOG)
                                != 0
                            // Watchdog aktivieren
                            {
                                self.watchdog_act = true;
                            } else {
                                self.watchdog_act = false;
                            }

                            if (self.rx_buffer[9] & sap_set_parameter_request::ACTIVATE_FREEZE) != 0
                            {
                                self.freeze_act = true;
                            } else {
                                self.freeze_act = false;
                            }

                            if (self.rx_buffer[9] & sap_set_parameter_request::ACTIVATE_SYNC) != 0 {
                                self.sync_act = true;
                            } else {
                                self.sync_act = false;
                            }

                            // watchdog1 = m_pbUartRxBuffer[10];
                            // watchdog2 = m_pbUartRxBuffer[11];

                            self.watchdog_time =
                                u32::from(self.rx_buffer[10]) * u32::from(self.rx_buffer[11]) * 10;

                            if self.rx_buffer[12] > 10 {
                                self.min_tsdr = self.rx_buffer[12] - 11;
                            } else {
                                self.min_tsdr = 0;
                            }

                            self.config.ident_high = self.rx_buffer[13];
                            self.config.ident_low = self.rx_buffer[14];

                            // User Parameter einlesen
                            if self.user_para.len() > 0 {
                                // User Parameter groesse = Laenge - DA, SA, FC, DSAP, SSAP, 7 Parameter Bytes
                                let user_para_len: usize = usize::from(pdu_len) - 12;
                                if user_para_len <= self.user_para.len() {
                                    for i in 0..user_para_len {
                                        self.user_para[i] = self.rx_buffer[16 + i];
                                    }
                                }
                            }

                            self.group = self.rx_buffer[15]; // wir speichern das gesamte Byte und sparen uns damit die Schleife. Ist unsere Gruppe gemeint, ist die Verundung von Gruppe und Empfang ungleich 0

                            // Kurzquittung
                            self.transmit_message_sc();
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
                            let mut buf: [u8; MODULE_CONFIG_SIZE] = [0; MODULE_CONFIG_SIZE];
                            buf.copy_from_slice(&self.module_config[..]);
                            self.transmit_message_sd2(
                                fc_response::DATA_LOW,
                                SAP_OFFSET,
                                &sap_data[..],
                                &buf,
                            );
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
                        let config_len: usize = usize::from(self.rx_buffer[1]) - 5;
                        let mut config_is_valid: bool = true;
                        if self.module_config.len() == config_len {
                            if self.module_config.len() > 0 {
                                for i in 0..config_len {
                                    let config_data: u8 = self.rx_buffer[9 + i];
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
                        self.transmit_message_sc();

                        if DpSlaveState::Wcfg == self.slave_state {
                            self.slave_state = DpSlaveState::Dxchg;
                        }
                    }

                    _ => (),
                } // Switch DSAP_data Ende
            }
            // Ziel: Slave Adresse, but no SAP
            else if destination_add == self.config.addr {
                // Status Abfrage

                if function_code == (fc_request::REQUEST + fc_request::FDL_STATUS) {
                    self.transmit_message_sd1(fc_response::FDL_STATUS_OK, 0);
                }
                // Master sendet Ausgangsdaten und verlangt Eingangsdaten (Send and Request Data)
                /*
                else if (function_code == (REQUEST_ + FCV_ + SRD_HIGH) ||
                         function_code == (REQUEST_ + FCV_ + FCB_ + SRD_HIGH))
                {
                 */
                else if function_code == (fc_request::REQUEST + fc_request::SRD_HIGH)
                    || function_code == (fc_request::REQUEST + fc_request::SRD_LOW)
                {
                    if self.sync_act && self.sync
                    // write data in output_register when sync
                    {
                        //TODO: size check
                        if self.output_data.len() > 0 {
                            for i in 0..usize::from(pdu_len - 3) {
                                self.output_data[i] = self.rx_buffer[7 + i];
                            }
                        }
                    } else
                    // normaler Betrieb
                    {
                        if self.output_data.len() > 0 {
                            for i in 0..usize::from(pdu_len - 3) {
                                self.output_data[i] = self.rx_buffer[7 + i];
                            }
                        }
                        self.interface
                            .data_processing(&mut [0; 0], &self.output_data[..]);
                    }

                    if self.freeze_act && self.freeze
                    // write input_register in telegram when freeze
                    {
                        // if self.input_data.len() > 0
                        // {
                        //   self.tx_buffer[7..(7+self.input_data.len())] = self.input_data;
                        // }
                        //TODO => freeze does not work
                        self.input_data[0] = 1;
                    } else
                    // normaler Betrieb
                    {
                        // self.interface
                        //     .data_processing(&mut self.input_data[..], &[0; 0]);
                        // if self.input_data.len() > 0
                        // {
                        //   self.tx_buffer[7..(7+self.input_data.len())] = self.input_data;
                        // }
                        self.input_data[0] = 1;
                    }

                    if self.input_data.len() > 0 {
                        if (self.diagnose_status_1 & sap_diagnose_byte1::EXT_DIAG) != 0 {
                            self.transmit_message_sd2(fc_response::DATA_HIGH, 0, &[0; 0], &[0; 0]);
                        // Diagnose Abfrage anfordern
                        } else {
                            let mut buf: [u8; INPUT_DATA_SIZE] = [0; INPUT_DATA_SIZE];
                            buf.copy_from_slice(&self.input_data[..]);
                            self.transmit_message_sd2(fc_response::DATA_LOW, 0, &buf, &[0; 0]);
                            // Daten senden
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
        } else
        // data not valid
        {
            self.rx_len = 0;
            self.stream_state = StreamState::WaitSyn;
            self.interface.run_timer(self.timer_timeout_in_us);
        }
        if self.config.receive_handling == ReceiveHandling::Thread {
            self.interface.stop_timer();
            self.stream_state = StreamState::WaitSyn;
            self.interface.activate_rx_interrupt();
            self.interface.run_timer(self.timer_timeout_in_us);
        }
    }
}
