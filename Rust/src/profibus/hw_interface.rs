/*********************************************************************
 * HwInterface
 *
 * Copyright (C) 2023 Marcel Maage
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * LICENSE file for more details.
 */
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

    fn activate_idle_interrupt(&mut self) {}

    fn deactivate_idle_interrupt(&mut self) {}

    fn set_tx_flag(&mut self) {}

    fn clear_tx_flag(&mut self) {}

    fn clear_rx_flag(&mut self) {}

    fn clear_idle_flag(&mut self) {}

    fn wait_for_activ_transmission(&mut self) {}

    fn is_rx_received(&mut self) -> bool {
        false
    }

    fn is_rx_idle(&mut self) -> bool {
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

    fn send_uart_data(&mut self, _len : usize) {}

    fn get_uart_data(&mut self) -> usize {0}

    fn schedule_receive_handling(&mut self) {}

    fn get_rx_buffer(&mut self) -> Option<&mut [u8]> {
        None
    }

    fn get_tx_buffer(&mut self) -> Option<&mut [u8]> {
        None
    }

    fn get_baudrate(&self) -> u32 {
        0
    }

    fn get_timer_frequency(&self) -> u32 {
        0
    }

    fn debug_write(&mut self, _debug: &str) {}
}
