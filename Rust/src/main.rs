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

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [I2C1_EV], peripherals = true,)]
mod app {
    use super::profibus::{Config as PbDpConfig, HwInterface, PbDpSlave};
    use nb::block;
    use stm32f1xx_hal::{
        gpio::{gpiob, gpioc, Output, PushPull}, //gpioa , Floating, Input, Alternate},
        pac::{USART1, USART3},
        prelude::*,
        serial::{Config, Rx as serialRx, Serial, Tx as serialTx /*TxDma1, RxDma1,*/},
    };
    //use rtic::{app};

    use heapless::Vec;


    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    const PERIOD: u32 = 56_000_000;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<PERIOD>; // 56 MHz

    #[local]
    struct Local {
        serial1_rx: serialRx<USART1>,
        serial1_tx: serialTx<USART1>,
        led: gpioc::PC13<Output<PushPull>>,
    }
    #[shared]
    struct Shared {
        profibus_slave: PbDpSlave<PbDpHwInterface>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // init::LateResources
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

        // Initialize the monotonic timer (CYCCNT)
        //cp.DCB.enable_trace();

        //cx.schedule.blinky(cx.start + PERIOD.cycles()).unwrap();

        // Initialize the monotonic
        let mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, PERIOD);

        //cp.DWT.enable_cycle_counter();

        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();

        //LED
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        //led.set_low().ok();

        // USART1
        let serial1_tx_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
        let serial1_rx_pin = gpioa.pa10;

        let mut serial1 = Serial::usart1(
            cx.device.USART1,
            (serial1_tx_pin, serial1_rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(115200.bps()),
            clocks,
        );

        // USART3
        let serial3_tx_pin = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let serial3_rx_pin = gpiob.pb11;

        let serial3 = Serial::usart3(
            cx.device.USART3,
            (serial3_tx_pin, serial3_rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(115200.bps()),
            clocks,
        );

        block!(serial1.write(b'S')).ok();

        let _dma1 = cx.device.DMA1.split();

        // DMA channel selection depends on the peripheral:
        // - USART1: TX = 4, RX = 5
        // - USART2: TX = 6, RX = 7
        // - USART3: TX = 3, RX = 2

        let (mut serial1_tx, serial1_rx) = serial1.split();

        let (serial3_tx, serial3_rx) = serial3.split();
        //let serial_dma_rx = serial_rx.with_dma(dma1.5);
        //let serial_dma_tx = serial_tx.with_dma(dma1.4);

        let profibus_config = PbDpConfig::default()
            .baudrate(500_000_u32)
            .counter_frequency(56_000_000_u32)
            .ident_high(0x00)
            .ident_low(0x2B)
            .addr(0x0B)
            .transmit_buffer(&mut Vec::<u8, 255>::new())
            .buf_size(45)
            .input_data_size(2)
            .output_data_size(5)
            .module_count(5)
            .user_para_size(0)
            .extern_diag_para_size(0)
            .vendor_data_size(0);

        let tx_en = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
        let rx_en = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
        let interface = PbDpHwInterface::new(serial3_tx, serial3_rx, tx_en, rx_en);

        let profibus_slave = PbDpSlave::new(profibus_config, interface);

        block!(serial1_tx.write(b't')).ok();

        blinky::spawn().unwrap();

        (
            Shared { profibus_slave },
            Local {
                serial1_rx,
                serial1_tx,
                led,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [serial1_rx, serial1_tx])]
    fn idle(cx: idle::Context) -> ! {
        //let serial = cx.resources.serial;
        let _serial1_rx = cx.local.serial1_rx;
        let _serial1_tx = cx.local.serial1_tx;

        //let buf = singleton!(: [u8; 8] = [0; 8]).unwrap();
        //let (_buf, _rx) = rx_channel.read(buf).wait();

        loop {
            // Read the byte that was just sent. Blocks until the read is complete

            //tx_channel.write(b"The quick brown fox");
            //rx_channel.ReadDma();
        }
    }

    #[task(priority = 1, local = [led])]
    fn blinky(cx: blinky::Context) {
        // Periodic
        //blinky::spawn_after(Seconds(1_u32)).unwrap();
        cx.local.led.toggle();
        blinky::spawn_after(1.secs()).unwrap();
    }

    #[task(binds = USART3, priority = 2, shared = [profibus_slave])]
    fn usart3_rx(cx: usart3_rx::Context) {
        let mut profibus_slave = cx.shared.profibus_slave;
        // // Echo back received packages with correct priority ordering.

        profibus_slave.lock(|profibus_slave| {
            let _t = profibus_slave.get_interface();
            // loop {
            //     match profibus_slave.get_interface().get_rx().is_rx_not_empty() {
            //         Ok(b) => {
            //             rx_queue.lock(|rx_queue| {
            //                 rx_queue.push(b).unwrap();
            //             });

            //             profibusSlave.lock(|profibusSlave| {
            //                 profibusSlave.handle_data(b);
            //             });

            //             // cx.local.serial3_tx.write(b).unwrap();
            //         }
            //         Err(_err) => break,
            //     }
            // }
        });
    }

    pub struct PbDpHwInterface {
        rx: serialRx<USART3>,
        tx: serialTx<USART3>,
        rx_en: gpiob::PB0<Output<PushPull>>,
        tx_en: gpiob::PB1<Output<PushPull>>,
    }

    impl PbDpHwInterface {
        pub fn new(
            tx: serialTx<USART3>,
            rx: serialRx<USART3>,
            tx_en: gpiob::PB1<Output<PushPull>>,
            rx_en: gpiob::PB0<Output<PushPull>>,
        ) -> Self {
            PbDpHwInterface {
                rx,
                tx,
                tx_en,
                rx_en,
            }
        }
        // pub fn get_rx(&self) -> &serialRx<USART3> {
        //     &self.rx
        // }

        // pub fn get_tx(&self) -> &serialTx<USART3> {
        //     &self.tx
        // }
    }

    impl HwInterface for PbDpHwInterface {
        fn config_timer(&mut self) {}

        fn run_timer(&mut self) {}

        fn stop_timer(&mut self) {}

        fn set_timer_counter(&mut self, value: u16) {}

        fn set_timer_max(&mut self, value: u16) {}

        fn clear_overflow_flag(&mut self) {}

        fn config_uart(&mut self) {}

        fn activate_tx_interrupt(&mut self) {
            self.tx.listen();
        }

        fn deactivate_tx_interrupt(&mut self) {
            self.tx.unlisten();
        }

        fn activate_rx_interrupt(&mut self) {
            self.rx.listen();
        }

        fn deactivate_rx_interrupt(&mut self) {
            self.rx.unlisten();
        }

        fn set_tx_flag(&mut self) {}

        fn clear_tx_flag(&mut self) {}

        fn clear_rx_flag(&mut self) {}

        fn wait_for_activ_transmission(&mut self) {}

        fn rx_data_received(&mut self) -> bool {self.rx.is_rx_not_empty()}

        fn tx_data_send(&mut self) -> bool {self.tx.is_tx_empty()}

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

        fn get_uart_value(&mut self) -> Result<u8> {
            self.rx.read()
        }

        fn set_uart_value(&mut self, value: u8) {self.tx.write(value)}

        fn config_error_led(&mut self) {}

        fn error_led_on(&mut self) {}

        fn error_led_off(&mut self) {}

        fn millis(&mut self) -> u32 {
            0
        }
    }
}