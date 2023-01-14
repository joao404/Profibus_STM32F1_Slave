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

use crate::app::{
    handle_data_receive, save_debug_message, timer2_max, usart3_rx, DEBUG_STRING_SIZE,
};
use crate::profibus::{DataHandlingInterface as PbDataHandling, HwInterface as PbInterface};
use crate::rtc_millis::Rtc;
use heapless::String;
use rtic::mutex_prelude::*;
use stm32f1xx_hal::{
    // dma::{dma1::C2, dma1::C3, RxDma, TxDma},
    gpio::{gpioa, gpiob, Output, PushPull}, //gpioa , Floating, Input, Alternate},
    pac::{TIM2, USART3},
    prelude::*,
    serial::{Config, Rx as serialRx, Serial, Tx as serialTx},
    timer::{CounterUs, Event},
};

pub(crate) fn handle_data_receive(cx: handle_data_receive::Context) {
    let mut profibus_slave = cx.shared.profibus_slave;

    profibus_slave.lock(|profibus_slave| {
        profibus_slave.handle_data_receive();
    });
}

pub(crate) fn usart3_rx(cx: usart3_rx::Context) {
    let mut profibus_slave = cx.shared.profibus_slave;

    profibus_slave.lock(|profibus_slave| {
        profibus_slave.serial_interrupt_handler();
    });
}

pub(crate) fn timer2_max(cx: timer2_max::Context) {
    let mut profibus_slave = cx.shared.profibus_slave;

    profibus_slave.lock(|profibus_slave| {
        profibus_slave.timer_interrupt_handler();
    });
}

pub struct PbDpHwInterface {
    tx: serialTx<USART3>,
    rx: serialRx<USART3>,
    // tx_dma: TxDma<serialTx<USART3>, C2>,
    // rx_dma: RxDma<serialRx<USART3>, C3>,
    tx_en: gpiob::PB1<Output<PushPull>>,
    rx_en: gpiob::PB0<Output<PushPull>>,
    timer_handler: CounterUs<TIM2>,
}

impl PbDpHwInterface {
    pub fn new(
        tx: serialTx<USART3>,
        rx: serialRx<USART3>,
        // tx_dma: TxDma<serialTx<USART3>, C2>,
        // rx_dma: RxDma<serialRx<USART3>, C3>,
        tx_en: gpiob::PB1<Output<PushPull>>,
        rx_en: gpiob::PB0<Output<PushPull>>,
        timer_handler: CounterUs<TIM2>,
    ) -> Self {
        PbDpHwInterface {
            tx,
            rx,
            // tx_dma,
            // rx_dma,
            tx_en,
            rx_en,
            timer_handler,
        }
    }
}

impl PbInterface for PbDpHwInterface {
    fn config_timer(&mut self) {}

    fn run_timer(&mut self, _timeout_in_us: u32) {
        self.timer_handler.start(_timeout_in_us.micros()).unwrap();
    }

    fn stop_timer(&mut self) {
        self.timer_handler.cancel().unwrap_or_default();
    }

    fn clear_overflow_flag(&mut self) {
        self.timer_handler.clear_interrupt(Event::Update);
    }

    fn config_uart(&mut self) {}

    fn activate_tx_interrupt(&mut self) {
        self.tx.listen_transmission_complete();
    }

    fn deactivate_tx_interrupt(&mut self) {
        self.tx.unlisten_transmission_complete();
    }

    fn activate_rx_interrupt(&mut self) {
        self.rx.listen();
    }

    fn deactivate_rx_interrupt(&mut self) {
        self.rx.unlisten();
    }

    fn activate_idle_interrupt(&mut self) {
        self.rx.listen_idle();
    }

    fn deactivate_idle_interrupt(&mut self) {
        self.rx.unlisten_idle();
    }

    fn set_tx_flag(&mut self) {}

    fn clear_tx_flag(&mut self) {}

    fn clear_rx_flag(&mut self) {}

    fn clear_idle_flag(&mut self) {
        self.rx.clear_idle_interrupt();
    }

    fn wait_for_activ_transmission(&mut self) {
        while !self.tx.is_tx_complete() {}
    }

    fn is_rx_received(&mut self) -> bool {
        self.rx.is_rx_not_empty()
    }

    fn is_rx_idle(&mut self) -> bool {
        self.rx.is_idle()
    }

    fn is_tx_done(&mut self) -> bool {
        self.tx.is_tx_empty()
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

    fn config_rs485_pin(&mut self) {
        self.tx_en.set_low();
        self.rx_en.set_high();
    }

    fn get_uart_value(&mut self) -> Option<u8> {
        match self.rx.read() {
            Ok(data) => Some(data),
            Err(_err) => None,
        }
    }

    fn set_uart_value(&mut self, _value: u8) {
        self.tx.write(_value).unwrap_or_default();
    }

    fn schedule_receive_handling(&mut self) {
        handle_data_receive::spawn().ok();
    }

    fn get_baudrate(&self) -> u32 {
        500_000_u32
    }

    fn get_timer_frequency(&self) -> u32 {
        1_000_000_u32
    }

    fn debug_write(&mut self, _debug: &str) {
        // self.serial_tx.write(_data).ok();
        let mut s: String<DEBUG_STRING_SIZE> = String::new();
        if s.push_str(_debug).is_ok() {
            save_debug_message::spawn(s).ok();
        }
    }
}

pub struct PbDpDataHandling {
    rtc: Rtc,
    debug_pin: gpioa::PA7<Output<PushPull>>,
}

impl PbDpDataHandling {
    pub fn new(rtc: Rtc, debug_pin: gpioa::PA7<Output<PushPull>>) -> Self {
        PbDpDataHandling { rtc, debug_pin }
    }
}

impl PbDataHandling for PbDpDataHandling {
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
        self.rtc.current_time()
    }
    fn data_processing(&self, _input: &mut [u8], _output: &[u8]) {
        if (_output.len() > 0) && (_input.len() > 0) {
            _input[0] = 22;
        }
    }

    fn debug_write(&mut self, _debug: &str) {
        // self.serial_tx.write(_data).ok();
        let mut s: String<DEBUG_STRING_SIZE> = String::new();
        if s.push_str(_debug).is_ok() {
            save_debug_message::spawn(s).ok();
        }
    }
}
