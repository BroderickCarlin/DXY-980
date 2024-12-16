#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use embedded_hal::digital::{OutputPin, PinState};
use {defmt_rtt as _, panic_probe as _};

mod display;

#[embassy_executor::task]
async fn run_display(display: display::Display) {
    display.run().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let a = Output::new(p.PA4, Level::Low, Speed::Low);
    let b = Output::new(p.PC0, Level::Low, Speed::Low);
    let c = Output::new(p.PC1, Level::Low, Speed::Low);
    let d = Output::new(p.PB0, Level::Low, Speed::Low);

    let en_1 = Output::new(p.PA9, Level::Low, Speed::Low);
    let en_2 = Output::new(p.PB6, Level::Low, Speed::Low);
    let en_3 = Output::new(p.PA5, Level::Low, Speed::Low);
    let en_4 = Output::new(p.PA7, Level::Low, Speed::Low);
    let en_5 = Output::new(p.PA6, Level::Low, Speed::Low);
    let en_6 = Output::new(p.PC7, Level::Low, Speed::Low);
    let en_7 = Output::new(p.PA8, Level::Low, Speed::Low);
    let en_8 = Output::new(p.PB10, Level::Low, Speed::Low);
    let en_9 = Output::new(p.PB4, Level::Low, Speed::Low);
    let en_10 = Output::new(p.PB5, Level::Low, Speed::Low);

    let mut mystery = Output::new(p.PB3, Level::High, Speed::Low);
    mystery.set_low();

    let (display, mut display_controller) = display::Display::new(a, b, c, d, en_1, en_2, en_3, en_4, en_5, en_6, en_7, en_8, en_9, en_10);

    let _ = spawner.spawn(run_display(display));

    let mut left = 0;
    let mut right = 9999;

    loop {
        left += 1;
        right -= 1;

        if left == 9999 {
            left = 0;
        }

        if right == 0 {
            right = 9999
        }

        display_controller.set_values(left, right);
        Timer::after_millis(10).await;
    }
}
