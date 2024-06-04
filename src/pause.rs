use std::{sync::{atomic::{AtomicBool, Ordering}, Mutex}, thread::sleep, time::Duration};

use esp_idf_svc::{hal::{gpio::{Input, InputPin, OutputPin, PinDriver, Pull}, task::block_on}, sys::EspError};

use crate::pomodoro::Pause;

#[derive(Debug)]
pub struct PauseButton<B> {
    is_paused: AtomicBool,
    button: Mutex<B>,
}

impl<B : InputPin + OutputPin> PauseButton<PinDriver<'_,B,Input>> {
    pub fn new(button: B) -> Result<Self, EspError> {
        let mut button = PinDriver::input(button)?;
        button.set_pull(Pull::Up)?;
        Ok(Self { is_paused: AtomicBool::new(true), button: Mutex::new(button) })
    }

    pub fn run(&self) -> Result<(), EspError> {
        loop {
            if self.wait_for_press()? {
                self.is_paused.fetch_xor(true, Ordering::SeqCst);
            }
        }
    }

    pub fn pause_state(&self) -> Pause {
        let pause_state = self.is_paused.load(Ordering::SeqCst);

        match pause_state {
            true => Pause::Paused,
            false => Pause::Running,
        }
    }

    fn wait_for_press(&self) -> Result<bool, EspError> {
        let mut lock = self.button.lock().unwrap();
        block_on(async { 
            lock.wait_for_falling_edge().await?;
            Ok(())
        })?;
        sleep(Duration::from_millis(50));
        Ok(lock.is_low())
    }
}

// let mut button = PinDriver::input(peripherals.pins.gpio4)?;
// button.set_pull(Pull::Up)?;

// thread::spawn({
//     let is_paused = is_paused.clone();
//     move || -> Result<(), EspError> {
//         let is_paused = is_paused.clone();
//         loop {
//             block_on(async { button.wait_for_falling_edge().await })?;
//             sleep(Duration::from_millis(50));
//             if button.is_low() {
//                 let paused = is_paused.fetch_xor(true, Ordering::SeqCst);
//             }
//         }
//     }
// });