#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_time::Timer;
use usb_serial::SerialPort;
use {defmt_rtt as _, panic_probe as _};

mod usb_serial;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut sp = usb_serial::init(&spawner, p.USB);

    // NOTE: info!() only goes to a debug probe, sp.print() goes out USB
    info!("Hello World!");
    sp.print("Hello World!\n").await;

    let mut led = Output::new(p.PIN_25, Level::Low);

    let btn_pen = Input::new(p.PIN_0, Pull::Down);

    let mut row_1 = Output::new(p.PIN_1, Level::Low);
    let mut row_2 = Output::new(p.PIN_2, Level::Low);
    let mut row_3 = Output::new(p.PIN_3, Level::Low);
    let mut row_4 = Output::new(p.PIN_4, Level::Low);
    let mut row_5 = Output::new(p.PIN_5, Level::Low);


    let col_1 = Input::new(p.PIN_6, Pull::Down);
    let col_2 = Input::new(p.PIN_7, Pull::Down);

    let mut pen_state = btn_pen.is_high();
    let mut btn_states = [[false; 2]; 5];
    led.set_high();
    loop {
        let new_state = btn_pen.is_high();

        if new_state != pen_state {
            pen_state = new_state;
            if pen_state {
                sp.print("Pen button is pressed\n").await;
            } else {
                sp.print("Pen button not pressed\n").await;
            }
        }

        async fn check_btns<'a>(row: &mut Output<'a>, col_1: &Input<'a>, col_2: &Input<'a>, states: &mut [bool; 2], name_1: &str, name_2: &str, sp: &mut SerialPort) {
            row.set_high();
            let but_1 = col_1.is_high();
            let but_2 = col_2.is_high();

            if but_1 != states[0] {
                states[0] = but_1;
                sp.print("Button ").await;
                sp.print(name_1).await;
                sp.print(" is ").await;
                if but_1 {
                    sp.print("pressed\n").await;
                } else {
                    sp.print("released\n").await;
                }
            }

            if but_2 != states[1] {
                states[1] = but_2;
                sp.print("Button ").await;
                sp.print(name_2).await;
                sp.print(" is ").await;
                if but_2 {
                    sp.print("pressed\n").await;
                } else {
                    sp.print("released\n").await;
                }
            }
            row.set_low();
        }

        // row 1
        check_btns(&mut row_1, &col_1, &col_2, &mut btn_states[0], "Up", "P1", &mut sp).await;
        Timer::after_millis(1).await;

        check_btns(&mut row_2, &col_1, &col_2, &mut btn_states[1], "Right", "Pause", &mut sp).await;
        Timer::after_millis(1).await;

        check_btns(&mut row_3, &col_1, &col_2, &mut btn_states[2], "Fast", "Enter", &mut sp).await;
        Timer::after_millis(1).await;

        check_btns(&mut row_4, &col_1, &col_2, &mut btn_states[3], "Down", "P2", &mut sp).await;
        Timer::after_millis(1).await;

        check_btns(&mut row_5, &col_1, &col_2, &mut btn_states[4], "Left", "Home", &mut sp).await;
        Timer::after_millis(1).await;
    }
}
