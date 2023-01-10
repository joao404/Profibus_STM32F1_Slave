/*********************************************************************
 * Profibus Stm32f1 slave
 *
 * Copyright (C) 2022 Marcel Maage
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

//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

//cargo build --release
//cargo flash --chip stm32f103C8 --release

//requires UART Rx on and Tx on

#![no_main]
#![no_std]

use panic_halt as _;

//use cortex_m::singleton;
//use cortex_m_semihosting::{hprintln};
mod profibus;
mod rtc_millis;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [I2C1_EV], peripherals = true,)]
mod app {
    use crate::profibus::{Config as PbDpConfig, ReceiveHandling, HwInterface, PbDpSlave};
    use crate::rtc_millis::Rtc;
    use nb::block;
    use stm32f1xx_hal::{
        dma::{dma1::C2, dma1::C3, RxDma, TxDma},
        gpio::{gpioa, gpiob, gpioc, Output, PushPull}, //gpioa , Floating, Input, Alternate},
        pac::{TIM2, USART1, USART3},
        prelude::*,
        serial::{Config, Rx as serialRx, Serial, Tx as serialTx},
        timer::{CounterUs, Event},
    };
    //use rtic::{app};

    // use heapless::Vec;

    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    const PERIOD: u32 = 56_000_000;

    const PROFIBUS_BUF_SIZE: usize = 50;
    const INPUT_DATA_SIZE: usize = 2;
    const OUTPUT_DATA_SIZE: usize = 5;
    const USER_PARA_SIZE: usize = 0;
    const EXTERN_DIAG_PARA_SIZE: usize = 0;
    const VENDOR_DATA_SIZE: usize = 5;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<PERIOD>; // 56 MHz

    #[local]
    struct Local {
        // serial1_rx: serialRx<USART1>,
        // serial1_tx: serialTx<USART1>,
        led: gpioc::PC13<Output<PushPull>>,
    }
    #[shared]
    struct Shared {
        profibus_slave: PbDpSlave<
            PbDpHwInterface,
            PROFIBUS_BUF_SIZE,
            INPUT_DATA_SIZE,
            OUTPUT_DATA_SIZE,
            USER_PARA_SIZE,
            EXTERN_DIAG_PARA_SIZE,
            VENDOR_DATA_SIZE,
        >,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .hclk(56.MHz())
            .sysclk(56.MHz())
            .pclk1(28.MHz())
            .pclk2(56.MHz())
            .freeze(&mut flash.acr);

        let mut afio = cx.device.AFIO.constrain();

        let mut cp = cx.core;

        // Initialize the monotonic
        let mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, PERIOD);
        // rtc for millis
        let mut pwr = cx.device.PWR;
        let mut backup_domain = rcc.bkp.constrain(cx.device.BKP, &mut pwr);
        let rtc = Rtc::new(cx.device.RTC, &mut backup_domain);

        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();

        //LED
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        // USART1
        let serial1_tx_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
        let serial1_rx_pin = gpioa.pa10;

        let mut serial1 = Serial::new(
            cx.device.USART1,
            (serial1_tx_pin, serial1_rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(500_000.bps()),
            &clocks,
        );
        let (mut serial1_tx, serial1_rx) = serial1.split();

        // USART3
        let serial3_tx_pin = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let serial3_rx_pin = gpiob.pb11;

        let serial3 = Serial::new(
            cx.device.USART3,
            (serial3_tx_pin, serial3_rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(500_000.bps()).wordlength_9bits().parity_even(),
            &clocks,
        );

        block!(serial1_tx.write(b'S')).ok();
        // let (_, serial1_tx) = serial1_tx.write(b"St").wait();

        // DMA channel selection depends on the peripheral:
        // - USART1: TX = 4, RX = 5
        // - USART2: TX = 6, RX = 7
        // - USART3: TX = 2, RX = 3

        // let (mut serial1_tx, serial1_rx) = serial1.split();
        let _dma1 = cx.device.DMA1.split();
        let (serial3_tx, serial3_rx) = serial3.split();
        // let serial3_tx_dma = serial3_tx.with_dma(dma1.2);
        // let serial3_rx_dma = serial3_rx.with_dma(dma1.3);

        let mut timer = cx.device.TIM2.counter_us(&clocks);
        timer.listen(Event::Update);

        let profibus_config = PbDpConfig::default()
            .baudrate(500_000_u32)
            .counter_frequency(1_000_000_u32)
            .ident_high(0x00)
            .ident_low(0x2B)
            .addr(0x0B)
            .receive_handling(ReceiveHandling::Thread);

        let tx_en = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
        let rx_en = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

        // let (_, serial1_tx) = serial1_tx.write(b"art").wait();
        block!(serial1_tx.write(b't')).ok();
        block!(serial1_tx.write(b'a')).ok();
        block!(serial1_tx.write(b'r')).ok();
        block!(serial1_tx.write(b't')).ok();

        let debug_pin = gpioa.pa7.into_push_pull_output(&mut gpioa.crl);

        let interface = PbDpHwInterface::new(
            serial3_tx, serial3_rx, tx_en, rx_en, timer, rtc, serial1_tx, debug_pin,
        );

        let profibus_slave =
            PbDpSlave::new(interface, profibus_config, [0x22, 0x20, 0x20, 0x10, 0x10]);

        // block!(serial1_tx.write(b't')).ok();

        blinky::spawn().unwrap();

        (
            Shared { profibus_slave },
            Local {
                // serial1_rx,
                // serial1_tx,
                led,
            },
            init::Monotonics(mono),
        )
    }

    // #[idle(local = [serial1_rx, serial1_tx])]
    // fn idle(cx: idle::Context) -> ! {
    //     //let serial = cx.resources.serial;
    //     // let _serial1_rx = cx.local.serial1_rx;
    //     // let _serial1_tx = cx.local.serial1_tx;

    //     //let buf = singleton!(: [u8; 8] = [0; 8]).unwrap();
    //     //let (_buf, _rx) = rx_channel.read(buf).wait();

    //     loop {
    //         // Read the byte that was just sent. Blocks until the read is complete

    //         // _serial1_tx.write(b'a').ok();
    //         //rx_channel.ReadDma();
    //     }
    // }

    #[task(priority = 1, local = [led])]
    fn blinky(cx: blinky::Context) {
        cx.local.led.toggle();
        blinky::spawn_after(1.secs()).unwrap();
    }

    #[task(priority = 1, shared = [profibus_slave])]
    fn handle_data_receive(cx: handle_data_receive::Context) {
        let mut profibus_slave = cx.shared.profibus_slave;

        profibus_slave.lock(|profibus_slave| {
            profibus_slave.handle_data_receive();
        });
    }

    #[task(binds = USART3, priority = 2, shared = [profibus_slave])]
    fn usart3_rx(cx: usart3_rx::Context) {
        let mut profibus_slave = cx.shared.profibus_slave;

        profibus_slave.lock(|profibus_slave| {
            profibus_slave.serial_interrupt_handler();
        });
    }

    #[task(binds = TIM2, priority = 2, shared = [profibus_slave])]
    fn timer2_max(cx: timer2_max::Context) {
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
        rtc: Rtc,
        serial_tx: serialTx<USART1>,
        debug_pin: gpioa::PA7<Output<PushPull>>,
        // serial_tx:TxDma<serialTx<USART1>, C4>
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
            rtc: Rtc,
            serial_tx: serialTx<USART1>,
            debug_pin: gpioa::PA7<Output<PushPull>>,
            // serial_tx: TxDma<serialTx<USART1>, C4>,
        ) -> Self {
            PbDpHwInterface {
                tx,
                rx,
                // tx_dma,
                // rx_dma,
                tx_en,
                rx_en,
                timer_handler,
                rtc,
                serial_tx,
                debug_pin,
            }
        }
    }

    impl HwInterface for PbDpHwInterface {
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

        fn set_tx_flag(&mut self) {}

        fn clear_tx_flag(&mut self) {
            self.tx.clear_transmission_complete_interrupt()
        }

        fn clear_rx_flag(&mut self) {}

        fn wait_for_activ_transmission(&mut self) {
            while !self.tx.is_tx_complete() {}
        }

        fn rx_data_received(&mut self) -> bool {
            self.rx.is_rx_not_empty()
        }

        fn tx_data_send(&mut self) -> bool {
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

        fn serial_write(&mut self, _data: u8) {
            self.serial_tx.write(_data).ok();
        }
    }
}
