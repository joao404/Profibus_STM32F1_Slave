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
use embassy_stm32::usart::{Config, DataBits, Parity, Uart};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
// use embassy_stm32::Peripherals;

mod pb_dp_interface;
mod profibus;

use crate::pb_dp_interface::{PbDpDataHandling, PbDpHwInterface};
use crate::profibus::{ProfibusConfig as PbDpConfig, PbDpSlave};

#[embassy_executor::task()]
async fn profibus_slave() {
    // unwrap!(Spawner::for_current_executor().await.spawn(profibus_client()));
    loop{
        
    }
    // Timer::after(Duration::from_secs(1)).await;
    //unwrap!(Spawner::for_current_executor().await.spawn(my_task(n + 1)));
}


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

    let led = Output::new(p.PC13, Level::High, Speed::Low);
    let mut test = Test::new(led);
    let mut test2 = Test2::new();

    let mut _tx_en = Output::new(p.PB1, Level::High, Speed::VeryHigh);
    let mut _rx_en = Output::new(p.PB0, Level::High, Speed::VeryHigh);

    let mut uart_config = Config::default();
    uart_config.baudrate = 500000u32;
    uart_config.data_bits = DataBits::DataBits9;
    uart_config.parity = Parity::ParityEven;
    let irq = interrupt::take!(USART3);
    let mut usart = Uart::new(
        p.USART3,
        p.PB11,
        p.PB10,
        irq,
        p.DMA1_CH2,
        p.DMA1_CH3,
        uart_config,
    );

    usart.write(b"Starting Echo\r\n").await.unwrap();

    // let mut msg: [u8; 8] = [0; 8];

    unwrap!(_spawner.spawn(profibus_slave()));

    loop {
        // usart.read_until_idle(&mut msg).await.unwrap();
        // usart.write(&msg).await.unwrap();

        // match embassy_futures::select::select(test.blink(),usart.read_until_idle(&mut msg)).await {
        //     _ => {},
        // };

        let blink1 = test.blink();
        let blink2 = test2.blink();

        // if let select::Either::First(first) = select::select(blink1, blink2).await
        // {
        //     if Test::blink == first
        //     {

        //     }
        // }

        match select::select(blink1, blink2).await {
            select::Either::First(_) => {
                info!("blink1");
            }
            select::Either::Second(_) => {
                info!("blink2");
            }
        }

        test.preblink();

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

    pub fn preblink(&mut self)
    {
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
