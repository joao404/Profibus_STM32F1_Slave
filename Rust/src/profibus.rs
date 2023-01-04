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

    fn tx_rs485_enable(&mut self) {}

    fn tx_rs485_disable(&mut self) {}

    fn rx_rs485_enable(&mut self) {}

    fn config_rs485_pin(&mut self) {}

    fn get_uart_value(&mut self) -> u8 {
        0
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
pub const  EXT_DIAG_TYPE:u8= 0xC0;     // Bit 6-7 ist Diagnose Typ
pub const  EXT_DIAG_BYTE_CNT:u8=  0x3F; // Bit 0-5 sind Anzahl der Diagnose Bytes

pub const  EXT_DIAG_GERAET:u8=  0x00 ; // Wenn Bit 7 und 6 = 00, dann Geraetebezogen
pub const  EXT_DIAG_KENNUNG:u8=  0x40; // Wenn Bit 7 und 6 = 01, dann Kennungsbezogen
pub const  EXT_DIAG_KANAL:u8=  0x80 ;  // Wenn Bit 7 und 6 = 10, dann Kanalbezogen
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Set Parameters Request (Daten Master)
///////////////////////////////////////////////////////////////////////////////////////////////////
/* Befehl */
pub const  LOCK_SLAVE_ 0x80   // Slave fuer andere Master gesperrt
pub const  UNLOCK_SLAVE_ 0x40 // Slave fuer andere Master freigegeben
pub const  ACTIVATE_SYNC_ 0x20
pub const  ACTIVATE_FREEZE_ 0x10
pub const  ACTIVATE_WATCHDOG_ 0x08

pub mod Dpv1StatusByte1{
pub const  DPV1_MODE:u8 = 0x80;
pub const  FAIL_SAVE_MODE:u8= 0x40;
pub const  PUBLISHER_MODE:u8= 0x20;
pub const  WATCHDOG_TB_1MS:u8= 0x04;
}

pub mod Dpv1StatusByte2{
pub const  PULL_PLUG_ALARM:u8=  0x80;
pub const  PROZESS_ALARM:u8= 0x40;
pub const  DIAGNOSE_ALARM:u8=  0x20;
pub const  VENDOR_ALARM:u8=  0x10;
pub const  STATUS_ALARM:u8=  0x08;
pub const  UPDATE_ALARM:u8=  0x04;
pub const  CHECK_CONFIG_MODE:u8= 0x01;
}

pub mod Dpv1StatusByte3{
pub const  PARAMETER_CMD_ON:u8= 0x80;
pub const  ISOCHRON_MODE_ON:u8= 0x10;
pub const  PARAMETER_BLOCK:u8 =  0x08;
}
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Check Config Request (Daten Master)
///////////////////////////////////////////////////////////////////////////////////////////////////
pub const  CFG_DIRECTION_ 0x30   // Bit 4-5 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
pub const  CFG_INPUT 0x10        // Eingang
pub const  CFG_OUTPUT 0x20       // Ausgang
pub const  CFG_INPUT_OUTPUT 0x30 // Eingang/Ausgang
pub const  CFG_SPECIAL 0x00      // Spezielles Format wenn mehr als 16/32Byte uebertragen werden sollen

pub const  CFG_KONSISTENZ_ 0x80    // Bit 7 ist Konsistenz. 0 = Byte oder Wort, 1 = Ueber gesamtes Modul
pub const  CFG_KONS_BYTE_WORT 0x00 // Byte oder Wort
pub const  CFG_KONS_MODUL 0x80     // Modul

pub const  CFG_WIDTH_ 0x40 // Bit 6 ist IO Breite. 0 = Byte (8bit), 1 = Wort (16bit)
pub const  CFG_BYTE 0x00   // Byte
pub const  CFG_WORD 0x40   // Wort

/* Kompaktes Format */
pub const  CFG_BYTE_CNT_ 0x0F // Bit 0-3 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.

/* Spezielles Format */
pub const  CFG_SP_DIRECTION_ 0xC0   // Bit 6-7 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
pub const  CFG_SP_VOID 0x00         // Leerplatz
pub const  CFG_SP_INPUT 0x40        // Eingang
pub const  CFG_SP_OUTPUT 0x80       // Ausgang
pub const  CFG_SP_INPUT_OUTPUT 0xC0 // Eingang/Ausgang

pub const  CFG_SP_VENDOR_CNT_ 0x0F // Bit 0-3 sind Anzahl der herstellerspezifischen Bytes. 0 = keine

/* Spezielles Format / Laengenbyte */
pub const  CFG_SP_BYTE_CNT_ 0x3F // Bit 0-5 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.

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
            diagnose_status_1:SapDiagnoseByte1::STATUS_1_DEFAULT,
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

    pub fn handle_message_timeout(&mut self) {
        let _test = self.buffer.len();

        self.buffer.clear();
    }
}
