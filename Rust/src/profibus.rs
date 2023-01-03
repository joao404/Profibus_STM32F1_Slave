use heapless::Vec;
use stm32f1xx_hal::{
    gpio::{gpiob, Output, PushPull}, //gpioa , Floating, Input, Alternate},
    pac::USART3,
    serial::{Rx, Tx},
};

pub struct PbDpHwInterface {
    // Associated function signature; `Self` refers to the implementor type.
    rx: Rx<USART3>,
    tx: Tx<USART3>,
    rx_en: gpiob::PB0<Output<PushPull>>,
    tx_en: gpiob::PB1<Output<PushPull>>,
}

impl PbDpHwInterface {
    pub fn new(tx: Tx<USART3>, rx: Rx<USART3>, tx_en: gpiob::PB1<Output<PushPull>>, rx_en: gpiob::PB0<Output<PushPull>>) -> Self {
        PbDpHwInterface { rx, tx, tx_en, rx_en }
    }

    pub fn get_rx(&self) -> &Rx<USART3> {
        &self.rx
    }

    pub fn get_tx(&self) -> &Tx<USART3> {
        &self.tx
    }

    pub fn config_timer(&self) {}

    pub fn run_timer(&self) {}

    pub fn stop_timer(&self) {}

    pub fn set_timer_counter(&self, value: u16) {}

    pub fn set_timer_max(&self, value: u16) {}

    pub fn clear_overflow_flag(&self) {}

    pub fn config_uart(&self) {}

    pub fn activate_tx_interrupt(&mut self) {self.tx.listen();
        }

    pub fn deactivate_tx_interrupt(&mut self) {self.tx.unlisten();}

    pub fn activate_rx_interrupt(&mut self) {self.rx.listen();}

    pub fn deactivate_rx_interrupt(&mut self) {self.rx.unlisten();}

    pub fn set_tx_flag(&self) {}

    pub fn clear_tx_flag(&self) {}

    pub fn clear_rx_flag(&self) {}

    pub fn wait_for_activ_transmission(&self) {}

    pub fn tx_rs485_enable(&self) {
        self.rx_en.set_high();
        self.tx_en.set_high();
    }

    pub fn tx_rs485_disable(&self) {
        self.tx_en.set_low();
        self.rx_en.set_low();
    }

    pub fn rx_rs485_enable(&self) {
        self.tx_en.set_low();
        self.rx_en.set_low();
    }

    pub fn config_rs485_pin(&self) {
        self.tx_en.set_low();
        self.rx_en.set_high();
    }

    pub fn get_uart_value(&self) -> u8 {
        0
    }

    pub fn set_uart_value(&self, value: u8) {}

    pub fn config_error_led(&self) {}

    pub fn error_led_on(&self) {}

    pub fn error_led_off(&self) {}

    pub fn millis(&self) -> u32 {
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

pub struct Config {
    ident_high: u8,
    ident_low: u8,
    counter_frequency: u32,
    baudrate: u32,
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
    pub fn counter_frequency(mut self, counter_frequency: u32) -> Self {
        self.counter_frequency = counter_frequency;
        self
    }
    pub fn baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = baudrate;
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
            counter_frequency: 0,
            baudrate: 500_000_u32,
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

pub struct PbDpSlave {
    config: Config,
    buffer: Vec<u8, 255>,
    slave_state: DpSlaveState,
    stream_state: StreamState,
    interface: PbDpHwInterface,
}

impl PbDpSlave {
    pub fn new(config: Config, interface: PbDpHwInterface) -> Self {
        Self {
            config,
            buffer: Vec::<u8, 255>::new(),
            slave_state: DpSlaveState::Por,
            stream_state: StreamState::WaitSyn,
            interface,
        }
    }

    pub fn get_interface(&self) -> &PbDpHwInterface {
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