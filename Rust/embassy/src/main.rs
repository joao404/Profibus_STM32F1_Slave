#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::peripherals::PC13;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart;
use embassy_time::{Duration, TimeoutError, Timer};
use {defmt_rtt as _, panic_probe as _};
// use embassy_stm32::Peripherals;

mod pb_dp_interface;
mod profibus;

//cargo build --release
//cargo flash --chip stm32f103C8 --release

use crate::pb_dp_interface::{PbDpHwInterface};
use crate::profibus::{Codec, CodecConfig, Device, DeviceConfig}; //ProfibusConfig as PbDpConfig, /*PbDpSlave*/};

// #[embassy_executor::task()]
// async fn profibus_slave(mut codec: Codec<PbDpHwInterface<'static>>) {
//     // unwrap!(Spawner::for_current_executor().await.spawn(profibus_client()));
//     loop {
//         let mut buffer: [u8; 128] = [0; 128];
//         match codec.receive(&mut buffer[..]).await {
//             Some(conn) => {
//                 info!("dest:{} source:{}", conn.destination_addr, conn.source_addr)
//             }
//             None => (),
//         }
//     }
//     // Timer::after(Duration::from_secs(1)).await;
//     //unwrap!(Spawner::for_current_executor().await.spawn(my_task(n + 1)));
// }

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p_config = embassy_stm32::Config::default();
    p_config.rcc.hse = Some(Hertz::mhz(8));
    p_config.rcc.hclk = Some(Hertz::mhz(56));
    p_config.rcc.sys_ck = Some(Hertz::mhz(56));
    p_config.rcc.pclk1 = Some(Hertz::mhz(28));
    p_config.rcc.pclk2 = Some(Hertz::mhz(56));
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    // let mut test = Test::new(led);
    // let mut test2 = Test2::new();

    let tx_en = Output::new(p.PB1, Level::High, Speed::VeryHigh);
    let rx_en = Output::new(p.PB0, Level::High, Speed::VeryHigh);

    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 500000u32;
    uart_config.data_bits = usart::DataBits::DataBits9;
    uart_config.parity = usart::Parity::ParityEven;
    let irq = interrupt::take!(USART3);
    let uart = usart::Uart::new(
        p.USART3,
        p.PB11,
        p.PB10,
        irq,
        p.DMA1_CH2,
        p.DMA1_CH3,
        uart_config,
    );

    let mut device_config = DeviceConfig::default();
    device_config.fdl_config.codec_config.t_s = 0x0B;

    // .ident_high(0x00)
    // .ident_low(0x2B)

    const PROFIBUS_BUF_SIZE: usize = 50;
    const INPUT_DATA_SIZE: usize = 2;
    const OUTPUT_DATA_SIZE: usize = 5;
    const USER_PARA_SIZE: usize = 0;
    const EXTERN_DIAG_PARA_SIZE: usize = 0;
    const VENDOR_DATA_SIZE: usize = 5;

    let mut device = Device::<PbDpHwInterface, 
    PROFIBUS_BUF_SIZE,
    INPUT_DATA_SIZE,
    OUTPUT_DATA_SIZE,
    USER_PARA_SIZE,
    EXTERN_DIAG_PARA_SIZE,
    VENDOR_DATA_SIZE, >::new(
        PbDpHwInterface::new(uart, tx_en, rx_en),
        device_config,
        [0x22, 0x20, 0x20, 0x10, 0x10],
    );
    //uart.write(b"Starting Echo\r\n").await.unwrap();

    // unwrap!(_spawner.spawn(profibus_slave(codec)));

    led.set_low();

    // let mut codec = Codec::<PbDpHwInterface>::new(
    //     PbDpHwInterface::new(uart, tx_en, rx_en),
    //     CodecConfig::default().t_s(0x0B),
    // );

    loop {
        if device.run().await
        {
            
        }

        // let mut buffer: [u8; 128] = [0; 128];
        // match embassy_time::with_timeout(Duration::from_millis(500), codec.receive(&mut buffer[..]))
        //     .await
        // {
        //     Ok(connection) => {
        //         match connection {
        //             Some(conn) => {
        //                 info!("dest:{} source:{}", conn.destination_addr, conn.source_addr);
        //                 if (conn.destination_addr == 0x0B)
        //                     && (conn.source_addr == 0x02)
        //                     && (conn.function_code == 76)
        //                 {
        //                     led.toggle();
        //                     // let tx_buffer: [u8;128] = [0;128];
        //                     // codec.transmit(&tx_buffer).await;
        //                 }
        //             }
        //             None => (),
        //         }
        //     }
        //     Err(_) => {
        //         // Go to save state
                
        //         // Wait for first message
        //         match codec.receive(&mut buffer[..]).await {
        //             Some(conn) => {
        //                 info!("dest:{} source:{}", conn.destination_addr, conn.source_addr);
        //                 if (conn.destination_addr == 0x0B)
        //                     && (conn.source_addr == 0x02)
        //                     && (conn.function_code == 76)
        //                 {
        //                     led.toggle();
        //                     // let tx_buffer: [u8;128] = [0;128];
        //                     // codec.transmit(&tx_buffer).await;
        //                 }
        //             }
        //             None => (),
        //         }
        //     }
        // }


        // match codec.receive(&mut buffer[..]).await {
        // // match codec.receive().await {
        //     Some(conn) => {
        //         info!("dest:{} source:{}", conn.destination_addr, conn.source_addr);
        //         if (conn.destination_addr == 0x0B) && (conn.source_addr == 0x02) && (conn.function_code == 76)
        //         {
        //             led.toggle();
        //             // let tx_buffer: [u8;128] = [0;128];
        //             // codec.transmit(&tx_buffer).await;
        //         }
        //     }
        //     None => (),
        // }

        // let blink1 = test.blink();
        // let blink2 = test2.blink();

        // match select::select(blink1, blink2).await {
        //     select::Either::First(_) => {
        //         info!("blink1");
        //     }
        //     select::Either::Second(_) => {
        //         info!("blink2");
        //     }
        // }

        // test.preblink();

        // info!("high");
        // led.set_high();
        // Timer::after(Duration::from_millis(300)).await;

        // info!("low");
        // led.set_low();
        // Timer::after(Duration::from_millis(300)).await;
    }
}

struct Test<'a> {
    led: Output<'a, PC13>,
}

impl<'a> Test<'a> {
    // pub fn new(p:&Peripherals) -> Self{
    //     Test { led: Output::new(p.PC13, Level::High, Speed::Low) }
    // }

    pub fn new(led: Output<'a, PC13>) -> Self {
        Test { led }
    }

    pub fn preblink(&mut self) {
        // self.blink();
        //unwrap!(_spawner.spawn(self.blink()));
    }

    pub async fn blink(&mut self) {
        loop {
            info!("high");
            self.led.set_high();
            Timer::after(Duration::from_millis(300)).await;

            info!("low");
            self.led.set_low();
            Timer::after(Duration::from_millis(300)).await;
        }
    }
}

struct Test2 {}

impl Test2 {
    // pub fn new(p:&Peripherals) -> Self{
    //     Test { led: Output::new(p.PC13, Level::High, Speed::Low) }
    // }

    pub fn new() -> Self {
        Test2 {}
    }

    pub async fn blink(&mut self) {
        loop {
            info!("high2");
            Timer::after(Duration::from_millis(300)).await;

            info!("low2");
            Timer::after(Duration::from_millis(100)).await;
        }
    }
}
