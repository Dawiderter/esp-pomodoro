//! LEDs utilty

use esp_idf_svc::{
    hal::gpio::{Level, Output, OutputPin, PinDriver},
    sys::EspError,
};

/// Helper trait for handling pins connected to an LED
pub trait Led {
    fn set_high(&mut self) -> Result<(), EspError>;
    fn set_low(&mut self) -> Result<(), EspError>;
    fn toggle(&mut self) -> Result<(), EspError>;
    fn set(&mut self, value: bool) -> Result<(), EspError>;
}

impl<T: OutputPin> Led for PinDriver<'_, T, Output> {
    fn set_high(&mut self) -> Result<(), EspError> {
        self.set_high()
    }

    fn set_low(&mut self) -> Result<(), EspError> {
        self.set_low()
    }

    fn toggle(&mut self) -> Result<(), EspError> {
        self.toggle()
    }

    fn set(&mut self, value: bool) -> Result<(), EspError> {
        self.set_level(match value {
            true => Level::High,
            false => Level::Low,
        })
    }
}

/// LEDs put into a trait for ease of passing to function
/// For concrete type see [`LedGroup`]
pub trait Leds { 
    fn red(&mut self) -> &mut impl Led;
    fn yellow(&mut self) -> &mut impl Led;
    fn green(&mut self) -> &mut impl Led;
}

/// Concrete group of LEDs.
/// Can be a chore to pass to a function, so use [`Leds`] then.
pub struct LedGroup<B, P, W> {
    pub break_led: B,
    pub pause_led: P,
    pub work_led: W,
}

type LedPin<'d, T> = PinDriver<'d, T, Output>;
impl<B : OutputPin, P : OutputPin, W : OutputPin> LedGroup<LedPin<'_,B>, LedPin<'_,P>, LedPin<'_,W>> {
    pub fn from_pins(red: B, yellow: P, green: W) -> Result<Self, EspError> {
        Ok(Self { 
            break_led : PinDriver::output(red)?, 
            pause_led : PinDriver::output(yellow)?, 
            work_led : PinDriver::output(green)? 
        })
    }
}

impl<B : OutputPin, P : OutputPin, W : OutputPin> Leds for LedGroup<LedPin<'_,B>, LedPin<'_,P>, LedPin<'_,W>> {
    fn red(&mut self) -> &mut impl Led {
        &mut self.break_led
    }

    fn yellow(&mut self) -> &mut impl Led {
        &mut self.pause_led
    }

    fn green(&mut self) -> &mut impl Led {
        &mut self.work_led
    }
}
