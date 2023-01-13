// use crate::Duration;

pub trait HwInterface {
    fn config_timer(&mut self) {}

    fn run_timer(&mut self, _timeout_in_us: u32) {}

    fn stop_timer(&mut self) {}

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

    fn is_rx_received(&mut self) -> bool {
        false
    }

    fn is_tx_done(&mut self) -> bool {
        false
    }

    fn tx_rs485_enable(&mut self) {}

    fn tx_rs485_disable(&mut self) {}

    fn rx_rs485_enable(&mut self) {}

    fn config_rs485_pin(&mut self) {}

    fn get_uart_value(&mut self) -> Option<u8> {
        None
    }

    fn set_uart_value(&mut self, _value: u8) {}

    fn send_uart_data(&mut self, _value: &[u8]) {}

    fn get_uart_data(&mut self, _value: &mut [u8]) {}

    fn schedule_receive_handling(&mut self) {}

    fn config_error_led(&mut self) {}

    fn error_led_on(&mut self) {}

    fn error_led_off(&mut self) {}

    fn millis(&mut self) -> u32 {
        0
    }

    fn data_processing(&self, _input: &mut [u8], _output: &[u8]) {}

    fn debug_write(&mut self, _debug: &str) {}
}
