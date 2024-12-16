use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_hal::digital::{OutputPin, PinState};
use embassy_stm32::gpio::Output;

const DISPLAY_TICK_DELAY_MS: u64 = 2;
static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, DisplayValues> = Signal::new();
struct Encoder {
    a: Output<'static>,
    b: Output<'static>,
    c: Output<'static>,
    d: Output<'static>,
}

impl Encoder {
    fn encode(&mut self, val: u8) {
        let states = match val {
            1 => [PinState::High, PinState::Low, PinState::Low, PinState::Low],
            2 => [PinState::Low, PinState::High, PinState::Low, PinState::Low],
            3 => [PinState::High, PinState::High, PinState::Low, PinState::Low],
            4 => [PinState::Low, PinState::Low, PinState::High, PinState::Low],
            5 => [PinState::High, PinState::Low, PinState::High, PinState::Low],
            6 => [PinState::Low, PinState::High, PinState::High, PinState::Low],
            7 => [
                PinState::High,
                PinState::High,
                PinState::High,
                PinState::Low,
            ],
            8 => [PinState::Low, PinState::Low, PinState::Low, PinState::High],
            9 => [PinState::High, PinState::Low, PinState::Low, PinState::High],
            _ => [PinState::Low, PinState::Low, PinState::Low, PinState::Low],
        };

        let _ = self.a.set_state(states[0]);
        let _ = self.b.set_state(states[1]);
        let _ = self.c.set_state(states[2]);
        let _ = self.d.set_state(states[3]);
    }
}

struct DisplayValues {
    left: u16,
    right: u16,
}

impl DisplayValues {
    fn encode_into(&self, output: &mut EncodedDisplayValues) {
        fn decompose(mut val: u16, output: &mut [u8; 4]) {
            if val == 0 {
                *output = [0, 0, 0, 0];
            } else if val >= 9999 {
                *output = [9, 9, 9, 9];
            } else {
                let thousands = val / 1000;
                val -= thousands * 1000;
                let hundreds = val / 100;
                val -= hundreds * 100;
                let tens = val / 10;
                let ones = val - tens * 10;

                *output = [thousands as u8, hundreds as u8, tens as u8, ones as u8];
            }
        }

        decompose(self.left, &mut output.left);
        decompose(self.right, &mut output.right);
    }
}

pub struct EncodedDisplayValues {
    left: [u8; 4],
    right: [u8; 4]
}

#[non_exhaustive]
pub struct DisplayController {}

impl DisplayController {
    pub fn set_values(&mut self, left: u16, right: u16) {
        DISPLAY_SIGNAL.signal(DisplayValues { left, right });
    }

    // pub fn clear(&mut self) {
    //     DISPLAY_SIGNAL.signal(DisplayValues { left: 0, right: 0 });
    // }
}

pub struct Display {
    encoder: Encoder,
    _en_1: Output<'static>,
    en_2: Output<'static>,
    en_3: Output<'static>,
    en_4: Output<'static>,
    en_5: Output<'static>,
    _en_6: Output<'static>,
    en_7: Output<'static>,
    en_8: Output<'static>,
    en_9: Output<'static>,
    en_10: Output<'static>,
}

impl Display
{
    pub fn new(
        a: Output<'static>,
        b: Output<'static>,
        c: Output<'static>,
        d: Output<'static>,
        en_1: Output<'static>,
        en_2: Output<'static>,
        en_3: Output<'static>,
        en_4: Output<'static>,
        en_5: Output<'static>,
        en_6: Output<'static>,
        en_7: Output<'static>,
        en_8: Output<'static>,
        en_9: Output<'static>,
        en_10: Output<'static>,
    ) -> (Self, DisplayController) {
        (
            Self {
                encoder: Encoder { a, b, c, d },
                _en_1: en_1,
                en_2,
                en_3,
                en_4,
                en_5,
                _en_6: en_6,
                en_7,
                en_8,
                en_9,
                en_10,
            },
            DisplayController {},
        )
    }
}

impl Display
{
    async fn show_digit<P: OutputPin>(encoder: &mut Encoder, val: u8, pin: &mut P) {
        encoder.encode(val);
        let _ = pin.set_high();
        Timer::after_millis(DISPLAY_TICK_DELAY_MS).await;
        let _ = pin.set_low();
    }

    pub async fn run(mut self) -> ! {
        let mut display_values = EncodedDisplayValues { left: [0, 0, 0, 0], right: [0, 0, 0, 0] };
        let mut skipping;

        loop {
            // Start of the loop, update the values
            if let Some(values) = DISPLAY_SIGNAL.try_take() {
                values.encode_into(&mut display_values);
            }

            {
                // Left display
                skipping = display_values.left[0] == 0;
                if !skipping {
                    Self::show_digit(&mut self.encoder, display_values.left[0], &mut self.en_2).await;
                }

                skipping &= display_values.left[1] == 0;
                if !skipping {
                    Self::show_digit(&mut self.encoder, display_values.left[1], &mut self.en_3).await;
                }

                // We never skip the last 2 characters
                Self::show_digit(&mut self.encoder, display_values.left[2], &mut self.en_4).await;
                Self::show_digit(&mut self.encoder, display_values.left[3], &mut self.en_5).await;
            }

            {
                // Right display
                skipping = display_values.right[0] == 0;
                if !skipping {
                    Self::show_digit(&mut self.encoder, display_values.right[0], &mut self.en_7).await;
                }

                skipping &= display_values.right[1] == 0;
                if !skipping {
                    Self::show_digit(&mut self.encoder, display_values.right[1], &mut self.en_8).await;
                }

                // We never skip the last 2 characters
                Self::show_digit(&mut self.encoder, display_values.right[2], &mut self.en_9).await;
                Self::show_digit(&mut self.encoder, display_values.right[3], &mut self.en_10).await;
            }
        }
    }
}
