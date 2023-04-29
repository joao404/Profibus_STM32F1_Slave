/*********************************************************************
 * PB DP Interface
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

use crate::profibus::{DataHandlingInterface as PbDataHandling, CodecHwInterface as PbInterface};

use embassy_stm32::peripherals::{PB0, PB1, PA7, USART3, DMA1_CH2, DMA1_CH3};
use embassy_stm32::gpio::Output;
use embassy_stm32::usart::Uart;
use embassy_time::{Duration, Timer};
//use async_trait::async_trait;

pub struct PbDpHwInterface<'a> {
    uart : Uart<'a, USART3, DMA1_CH2, DMA1_CH3>,
    tx_en: Output<'a, PB1>,
    rx_en: Output<'a, PB0>,
}

impl<'a> PbDpHwInterface<'a> {
    pub fn new(
        uart : Uart<'a, USART3, DMA1_CH2, DMA1_CH3>,
        tx_en: Output<'a, PB1>,
        rx_en: Output<'a, PB0>,
    ) -> Self {
        PbDpHwInterface {
            uart,
            tx_en,
            rx_en,
        }
    }
}

//#[async_trait(?Send)]
impl<'a> PbInterface for PbDpHwInterface<'a> {
    fn config_timer(&mut self) {}

    async fn wait_for(&mut self, _time_in_us : u32) {
        Timer::after(Duration::from_micros(_time_in_us.into())).await;
    }

    fn config_uart(&mut self) {}

    async fn wait_for_activ_transmission(&mut self) {
        // while !self.tx.is_tx_complete() {}
    }

    fn tx_rs485_enable(&mut self) {
        self.rx_en.set_high();
        self.tx_en.set_high();
    }

    fn tx_rs485_disable(&mut self) {
        self.tx_en.set_low();
        self.rx_en.set_low();
    }

    fn rx_rs485_enable(&mut self) {
        self.tx_en.set_low();
        self.rx_en.set_low();
    }

    fn rx_rs485_disable(&mut self) {
        self.rx_en.set_high();
    }

    fn config_rs485_pin(&mut self) {
        self.tx_en.set_low();
        self.rx_en.set_high();
    }

    async fn send_uart_data(&mut self, _value: &[u8]) 
    {
        self.uart.write(&_value).await.unwrap();
    }

    async fn receive_uart_data<'b>(&mut self, _value: &'b mut [u8], len: &mut usize)
    {
        match self.uart.read_until_idle(_value).await
        {
            Ok(size) => *len = size,
            _=> *len = 0,
        }
    }

    fn get_baudrate(&self) -> u32 {
        500_000_u32
    }
}

pub struct PbDpDataHandling<'a> {
    debug_pin: Output<'a, PA7>,
}

impl<'a> PbDpDataHandling<'a> {
    pub fn new(debug_pin: Output<'a, PA7>) -> Self {
        PbDpDataHandling {debug_pin }
    }
}

impl<'a> PbDataHandling for PbDpDataHandling<'a> {
    fn config_error_led(&mut self) {
        self.debug_pin.set_high();
    }

    fn error_led_on(&mut self) {
        self.debug_pin.set_low();
    }

    fn error_led_off(&mut self) {
        self.debug_pin.set_high();
    }

    fn millis(&mut self) -> u32 {
        0
    }
    fn data_processing(&self, _input: &mut [u8], _output: &[u8]) {
        if (_output.len() > 0) && (_input.len() > 0) {
            _input[0] = 22;
        }
    }
}