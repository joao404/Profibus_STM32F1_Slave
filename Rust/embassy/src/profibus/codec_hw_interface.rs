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
#![allow(incomplete_features)]

//use async_trait::async_trait;

//#[async_trait]
pub trait HwInterface {
    fn config_timer(&mut self) {}

    async fn wait_for(&mut self, _time_in_us: u32) {}

    fn config_uart(&mut self) {}

    async fn wait_for_activ_transmission(&mut self) {}

    fn tx_rs485_enable(&mut self) {}

    fn tx_rs485_disable(&mut self) {}

    fn rx_rs485_enable(&mut self) {}

    fn rx_rs485_disable(&mut self) {}

    fn config_rs485_pin(&mut self) {}

    async fn send_uart_data(&mut self, _value: &[u8]) {}

    async fn receive_uart_data(&mut self, _value: &mut [u8], _len: &mut usize) {}

    fn get_baudrate(&self) -> u32 {
        0
    }
}
