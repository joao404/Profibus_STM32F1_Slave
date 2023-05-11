#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeviceState {
    Por = 1,   // Power on reset
    Wrpm = 2,  // Wait for parameter
    Wcfg = 3,  // Wait for config
    Dxchg = 4, // Data exchange
}

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum StreamState {
    WaitSyn,
    WaitData,
    GetData,
    HandleData,
    WaitMinTsdr,
    SendData,
}

#[allow(dead_code)]
pub mod cmd_type {
    pub const SD1: u8 = 0x10; // Telegramm ohne Datenfeld
    pub const SD2: u8 = 0x68; // Daten Telegramm variabel
    pub const SD3: u8 = 0xA2; // Daten Telegramm fest
    pub const SD4: u8 = 0xDC; // Token
    pub const SC: u8 = 0xE5; // Kurzquittung
    pub const ED: u8 = 0x16; // Ende
}

#[allow(dead_code)]
pub mod FcRequestLowNibble{
    pub const TIME_EVENT :u8 = 0x0;
    pub const SDA_LOW :u8 = 0x3;
    pub const SDN_LOW :u8 = 0x4;
    pub const SDA_HIGH :u8 = 0x5;
    pub const SDN_HIGH :u8 = 0x6;
    pub const MSRD :u8 = 0x7;
    pub const REQUEST_FDL_STATUS :u8 = 0x9;
    pub const TIME_EVENT_SYNCHRONISATION :u8 = 0xA;
    pub const COUNTER_VALUE :u8 = 0xB;
    pub const SRD_LOW :u8 = 0xC;
    pub const SRD_HIGH :u8 = 0xD;
    pub const REQUEST_IDENT_WITH_REPLY :u8 = 0xE;
    pub const REQUEST_LSAP_STATUS_WITH_REPLY :u8 =0xF;
}

#[allow(dead_code)]
pub mod FcRequestHighNibble{
    pub const FCV :u8 = 0x10;
    pub const FCB :u8 = 0x20;
    pub const REQUEST_TELEGRAM :u8 = 0x40;
    pub const RESERVED :u8 = 0x80;// must be zero
}

#[allow(dead_code)]
pub mod FcResponseLowNibble{
    pub const OK :u8 = 0x0;
    pub const USER_ERROR :u8 = 0x1; // UE
    pub const NO_RESOURCE :u8 = 0x2; // RR
    pub const SAP_NOT_ACTIVE :u8 = 0x3; // RS
    pub const DATA_LOW :u8 = 0x8; // DL
    pub const NO_RESPONSE :u8 = 0x9; // NR
    pub const DATA_HIGH :u8 = 0xA; // DH
    pub const DATA_NOT_RECEIVED_LOW :u8 = 0xC; // RDL
    pub const DATA_NOT_RECEIVED_HIGH :u8 = 0xD; // RDH
}

#[allow(dead_code)]
pub mod FcResponseHighNibble{
    pub const DEVICE_MASK : u8 = 0x30;
    pub const SLAVE :u8 = 0x00;
    pub const MASTER_NOT_READY :u8 = 0x10;
    pub const MASTER_READY_WITHOUT_TOKEN :u8 = 0x20;
    pub const MASTER_READY_TOKEN :u8 = 0x30;
    pub const REQUEST_TELEGRAM :u8 = 0x40; // must be zero
    pub const RESERVED :u8 = 0x80; // must be zero
}

#[allow(dead_code)]
pub mod sap_codes {
    pub const SET_SLAVE_ADR:u8 = 55;     // Master setzt Slave Adresse; Slave Anwortet mit SC
    pub const RD_INP :u8 = 56;           // Master fordert Input Daten; Slave sendet Input Daten
    pub const RD_OUTP :u8 = 57;          // Master fordert Output Daten; Slave sendet Output Daten
    pub const GLOBAL_CONTROL :u8 = 58;   // Master Control; Slave Antwortet nicht
    pub const GET_CFG :u8 = 59;          // Master fordert Konfig.; Slave sendet Konfiguration
    pub const SLAVE_DIAGNOSTIC :u8 = 60; // Master fordert Diagnose; Slave sendet Diagnose Daten
    pub const SET_PRM :u8 = 61;          // Master sendet Parameter; Slave sendet SC
    pub const CHK_CFG :u8 = 62;          // Master sendet Konfuguration; Slave sendet SC
}

#[allow(dead_code)]
pub mod sap_global_control {
    pub const CLEAR_DATA: u8 = 0x02;
    pub const UNFREEZE: u8 = 0x04;
    pub const FREEZE: u8 = 0x08;
    pub const UNSYNC: u8 = 0x10;
    pub const SYNC: u8 = 0x20;
}

#[allow(dead_code)]
pub mod sap_diagnose_byte1 {
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

#[allow(dead_code)]
pub mod sap_diagnose_byte2 {
    pub const STATUS_2_DEFAULT: u8 = 0x04;
    pub const PRM_REQ: u8 = 0x01;
    pub const STAT_DIAG: u8 = 0x02;
    pub const WD_ON: u8 = 0x08;
    pub const FREEZE_MODE: u8 = 0x10;
    pub const SYNC_MODE: u8 = 0x20;
    //pub const  free                  0x40
    pub const DEACTIVATED: u8 = 0x80;
}

#[allow(dead_code)]
pub mod sap_diagnose_byte3 {
    pub const DIAG_SIZE_OK: u8 = 0x00;
    pub const DIAG_SIZE_ERROR: u8 = 0x80;
}

#[allow(dead_code)]
pub mod sap_diagnose_ext {
    pub const EXT_DIAG_TYPE: u8 = 0xC0; // Bit 6-7 ist Diagnose Typ
    pub const EXT_DIAG_BYTE_CNT: u8 = 0x3F; // Bit 0-5 sind Anzahl der Diagnose Bytes

    pub const EXT_DIAG_GERAET: u8 = 0x00; // Wenn Bit 7 und 6 = 00; dann Geraetebezogen
    pub const EXT_DIAG_KENNUNG: u8 = 0x40; // Wenn Bit 7 und 6 = 01; dann Kennungsbezogen
    pub const EXT_DIAG_KANAL: u8 = 0x80; // Wenn Bit 7 und 6 = 10; dann Kanalbezogen
}

#[allow(dead_code)]
pub mod sap_set_parameter_request {
    pub const LOCK_SLAVE: u8 = 0x80; // Slave fuer andere Master gesperrt
    pub const UNLOCK_SLAVE: u8 = 0x40; // Slave fuer andere Master freigegeben
    pub const ACTIVATE_SYNC: u8 = 0x20;
    pub const ACTIVATE_FREEZE: u8 = 0x10;
    pub const ACTIVATE_WATCHDOG: u8 = 0x08;
}

#[allow(dead_code)]
pub mod dpv1_status_byte1 {
    pub const DPV1_MODE: u8 = 0x80;
    pub const FAIL_SAVE_MODE: u8 = 0x40;
    pub const PUBLISHER_MODE: u8 = 0x20;
    pub const WATCHDOG_TB_1MS: u8 = 0x04;
}

#[allow(dead_code)]
pub mod dpv1_status_byte2 {
    pub const PULL_PLUG_ALARM: u8 = 0x80;
    pub const PROZESS_ALARM: u8 = 0x40;
    pub const DIAGNOSE_ALARM: u8 = 0x20;
    pub const VENDOR_ALARM: u8 = 0x10;
    pub const STATUS_ALARM: u8 = 0x08;
    pub const UPDATE_ALARM: u8 = 0x04;
    pub const CHECK_CONFIG_MODE: u8 = 0x01;
}

#[allow(dead_code)]
pub mod dpv1_status_byte3 {
    pub const PARAMETER_CMD_ON: u8 = 0x80;
    pub const ISOCHRON_MODE_ON: u8 = 0x10;
    pub const PARAMETER_BLOCK: u8 = 0x08;
}

#[allow(dead_code)]
pub mod sap_check_config_request {
    pub const CFG_DIRECTION: u8 = 0x30; // Bit 4-5 ist Richtung. 01 =  Eingang; 10 = Ausgang; 11 = Eingang/Ausgang
    pub const CFG_INPUT: u8 = 0x10; // Eingang
    pub const CFG_OUTPUT: u8 = 0x20; // Ausgang
    pub const CFG_INPUT_OUTPUT: u8 = 0x30; // Eingang/Ausgang
    pub const CFG_SPECIAL: u8 = 0x00; // Spezielles Format wenn mehr als 16/32Byte uebertragen werden sollen

    pub const CFG_KONSISTENZ: u8 = 0x80; // Bit 7 ist Konsistenz. 0 = Byte oder Wort; 1 = Ueber gesamtes Modul
    pub const CFG_KONS_BYTE_WORT: u8 = 0x00; // Byte oder Wort
    pub const CFG_KONS_MODUL: u8 = 0x80; // Modul

    pub const CFG_WIDTH: u8 = 0x40; // Bit 6 ist IO Breite. 0 = Byte (8bit); 1 = Wort (16bit)
    pub const CFG_BYTE: u8 = 0x00; // Byte
    pub const CFG_WORD: u8 = 0x40; // Wort

    /* Kompaktes Format */
    pub const CFG_BYTE_CNT: u8 = 0x0F; // Bit 0-3 sind Anzahl der Bytes oder Worte. 0 = 1 Byte; 1 = 2 Byte usw.

    /* Spezielles Format */
    pub const CFG_SP_DIRECTION: u8 = 0xC0; // Bit 6-7 ist Richtung. 01 =  Eingang; 10 = Ausgang; 11 = Eingang/Ausgang
    pub const CFG_SP_VOID: u8 = 0x00; // Leerplatz
    pub const CFG_SP_INPUT: u8 = 0x40; // Eingang
    pub const CFG_SP_OUTPUT: u8 = 0x80; // Ausgang
    pub const CFG_SP_INPUT_OUTPUT: u8 = 0xC0; // Eingang/Ausgang

    pub const CFG_SP_VENDOR_CNT: u8 = 0x0F; // Bit 0-3 sind Anzahl der herstellerspezifischen Bytes. 0 = keine

    /* Spezielles Format / Laengenbyte */
    pub const CFG_SP_BYTE_CNT: u8 = 0x3F; // Bit 0-5 sind Anzahl der Bytes oder Worte. 0 = 1 Byte; 1 = 2 Byte usw.
}
