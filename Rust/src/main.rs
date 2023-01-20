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
mod pb_dp_interface;
mod profibus;
mod rtc_millis;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [I2C1_EV], peripherals = true,)]
mod app {
    use crate::pb_dp_interface::{PbDpDataHandling, PbDpHwInterface};
    use crate::profibus::{ConfigOld as PbDpConfig, PbDpSlave, ReceiveHandling, CodecConfig, Fdl};
    use crate::rtc_millis::Rtc;
    use heapless::{
        spsc::{Consumer, Producer, Queue},
        String,
    };
    use nb::block;
    use stm32f1xx_hal::{
        // dma::{dma1::C2, dma1::C3, RxDma, TxDma},
        gpio::{gpioa, gpiob, gpioc, Output, PushPull}, //gpioa , Floating, Input, Alternate},
        pac::USART1,
        prelude::*,
        serial::{Config, Rx as serialRx, Serial, Tx as serialTx},
        timer::Event,
    };

    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    const PERIOD: u32 = 56_000_000;

    const PROFIBUS_BUF_SIZE: usize = 50;
    const INPUT_DATA_SIZE: usize = 2;
    const OUTPUT_DATA_SIZE: usize = 5;
    const USER_PARA_SIZE: usize = 0;
    const EXTERN_DIAG_PARA_SIZE: usize = 0;
    const VENDOR_DATA_SIZE: usize = 5;

    const DEBUG_QUEUE_SIZE: usize = 255;
    pub const DEBUG_STRING_SIZE: usize = 10;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<PERIOD>; // 56 MHz

    #[local]
    struct Local {
        serial1_rx: serialRx<USART1>,
        serial1_tx: serialTx<USART1>,
        debug_consumer: Consumer<'static, u8, DEBUG_QUEUE_SIZE>,
        led: gpioc::PC13<Output<PushPull>>,
    }

    // type SerialHwInterface = PbDpHwInterface<PROFIBUS_BUF_SIZE>;
    // type ProfibusCodec = Codec<'static, SerialHwInterface, dyn FdlTrait>;

    #[shared]
    struct Shared {
        debug_producer: Producer<'static, u8, DEBUG_QUEUE_SIZE>,
        // profibus_slave: PbDpSlave<
        //     // PbDpHwInterface<PROFIBUS_BUF_SIZE>,
        //     PbDpDataHandling,
        //     INPUT_DATA_SIZE,
        //     OUTPUT_DATA_SIZE,
        //     USER_PARA_SIZE,
        //     EXTERN_DIAG_PARA_SIZE,
        //     VENDOR_DATA_SIZE,
        // >,
        // profibus_codec : Codec<'static, PbDpHwInterface<PROFIBUS_BUF_SIZE>, PbDpSlave<PbDpDataHandling,
        // INPUT_DATA_SIZE,
        // OUTPUT_DATA_SIZE,
        // USER_PARA_SIZE,
        // EXTERN_DIAG_PARA_SIZE,
        // VENDOR_DATA_SIZE,>>,
        profibus_fdl: Fdl<'static,  PbDpHwInterface<PROFIBUS_BUF_SIZE>>,
    }

    #[init(local = [debug_queue: Queue<u8, DEBUG_QUEUE_SIZE> = Queue::new()])]
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

        let serial1 = Serial::new(
            cx.device.USART1,
            (serial1_tx_pin, serial1_rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(500_000.bps()),
            &clocks,
        );
        let (mut serial1_tx, serial1_rx) = serial1.split();
        let (debug_producer, debug_consumer) = cx.local.debug_queue.split();

        // USART3
        let serial3_tx_pin = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let serial3_rx_pin = gpiob.pb11;

        let serial3 = Serial::new(
            cx.device.USART3,
            (serial3_tx_pin, serial3_rx_pin),
            &mut afio.mapr,
            Config::default()
                .baudrate(500_000.bps())
                .wordlength_9bits()
                .parity_even(),
            &clocks,
        );

        block!(serial1_tx.write(b'S')).ok();

        // DMA channel selection depends on the peripheral:
        // - USART1: TX = 4, RX = 5
        // - USART2: TX = 6, RX = 7
        // - USART3: TX = 2, RX = 3

        let dma1 = cx.device.DMA1.split();
        let (serial3_tx, serial3_rx) = serial3.split();
        // let serial3_tx_dma = serial3_tx.with_dma(dma1.2);
        // let rx_buffer: [u8; 2];
        // serial3_tx_dma.write(&rx_buffer);
        // let &mut serial3_tx = &mut serial3_tx_dma.payload();
        // let serial3_rx_dma = serial3_rx.with_dma(dma1.3);

        let mut timer = cx.device.TIM2.counter_us(&clocks);
        timer.listen(Event::Update);

        let profibus_config = PbDpConfig::default()
            .ident_high(0x00)
            .ident_low(0x2B)
            .addr(0x0B)
            .receive_handling(ReceiveHandling::Thread);

        let tx_en = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
        let rx_en = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

        block!(serial1_tx.write(b't')).ok();
        block!(serial1_tx.write(b'a')).ok();
        block!(serial1_tx.write(b'r')).ok();
        block!(serial1_tx.write(b't')).ok();

        let debug_pin = gpioa.pa7.into_push_pull_output(&mut gpioa.crl);

        let mut serial_interface :PbDpHwInterface<PROFIBUS_BUF_SIZE> = PbDpHwInterface::new(serial3_tx, serial3_rx, tx_en, rx_en, timer);

        let data_interface = PbDpDataHandling::new(rtc, debug_pin);

        // let profibus_slave = PbDpSlave::new(
        //     // serial_interface,
        //     data_interface,
        //     profibus_config,
        //     [0x22, 0x20, 0x20, 0x10, 0x10],
        // );

        let profibus_slave = PbDpSlave::new();

        

        let serial_config = CodecConfig::default()
        .t_s(0x0B)
        .receive_handling(ReceiveHandling::Thread);


        let profibus_fdl: Fdl<PbDpHwInterface<PROFIBUS_BUF_SIZE>> = Fdl::new(serial_config, serial_interface);//, data_interface);

        // let profibus_codec = Codec::new(serial_interface, serial_config);

        blinky::spawn().unwrap();

        (
            Shared {
                debug_producer,
                profibus_fdl,
            },
            Local {
                serial1_rx,
                serial1_tx,
                debug_consumer,
                led,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [serial1_rx, serial1_tx, debug_consumer])]
    fn idle(cx: idle::Context) -> ! {
        let _serial1_rx = cx.local.serial1_rx;
        let serial1_tx = cx.local.serial1_tx;
        let debug_consumer = cx.local.debug_consumer;
        //let buf = singleton!(: [u8; 8] = [0; 8]).unwrap();
        //let (_buf, _rx) = rx_channel.read(buf).wait();

        loop {
            // debug_consumer.lock(|debug_consumer| {
            if let Some(data) = debug_consumer.dequeue() {
                block!(serial1_tx.write(data)).ok();
            }
            // match debug_consumer.dequeue() {
            //     Some(data) => {block!(serial1_tx.write(data)).ok();},
            //     None => { /* sleep */ },
            // }
            // });
        }
    }

    #[task(priority = 1, shared = [profibus_fdl], local = [led])]
    fn blinky(cx: blinky::Context) {
        cx.local.led.toggle();
        // let mut profibus_slave = cx.shared.profibus_slave;
        // profibus_slave.lock(|profibus_slave| {
        //     let input = profibus_slave.access_input();
        //     if input.len() > 0 {
        //         input[0] += 1;
        //     }
        // });
        blinky::spawn_after(1.secs()).unwrap();
    }

    #[task(capacity = 10, shared = [debug_producer])]
    fn save_debug_message(cx: save_debug_message::Context, data: String<DEBUG_STRING_SIZE>) {
        let mut debug_producer = cx.shared.debug_producer;
        debug_producer.lock(|debug_producer| {
            for c in data.chars() {
                if debug_producer.enqueue(c as u8).is_err() {
                    break;
                }
            }
        });
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    /// Profibus
    ///////////////////////////////////////////////////////////////////////////////////////////////
    use crate::pb_dp_interface::{handle_data_receive, timer2_max, usart3_rx};

    extern "Rust" {
        #[task(priority = 1, shared = [profibus_fdl])]
        fn handle_data_receive(cx: handle_data_receive::Context);

        #[task(binds = USART3, priority = 2, shared = [profibus_fdl])]
        fn usart3_rx(cx: usart3_rx::Context);

        #[task(binds = TIM2, priority = 2, shared = [profibus_fdl])]
        fn timer2_max(cx: timer2_max::Context);
    }
}
