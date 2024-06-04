use std::{
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        i2c::{I2cConfig, I2cDriver},
        peripherals::Peripherals,
        prelude::*,
    },
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    timer::EspTaskTimerService,
};

use esp_pomodoro::{
    feedback::Sh1106Feedback,
    led::LedGroup,
    pause::PauseButton,
    pomodoro::{Phase, Pomodoro},
    wifi::{sntp_connect, wifi_connect},
};
use sh1106::mode::GraphicsMode;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let peripherals = Peripherals::take()?;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &config,
    )?;

    let mut display: GraphicsMode<_> = sh1106::Builder::new().connect_i2c(i2c).into();

    display.init().unwrap();
    display.flush().unwrap();

    let mut feedback = Sh1106Feedback::from_display(display);
    let _wifi = wifi_connect(
        peripherals.modem,
        sys_loop.clone(),
        nvs.clone(),
        &mut feedback,
    )?;
    let _sntp = sntp_connect(&mut feedback)?;

    let mut display = feedback.take_display();

    let button = Arc::new(PauseButton::new(peripherals.pins.gpio4)?);

    let mut leds = LedGroup::from_pins(
        peripherals.pins.gpio23,
        peripherals.pins.gpio19,
        peripherals.pins.gpio18,
    )?;

    let mut pomodoro = Pomodoro::init(Phase::Work, button.pause_state());

    {
        let button = button.clone();
        thread::spawn(move || -> Result<(), EspError> {
            let _result = button.run();
            Ok(())
        });
    }

    let timer_service = EspTaskTimerService::new()?;

    let timer = timer_service.timer(move || {
        pomodoro.set_pause(button.pause_state());
        pomodoro.update();
        pomodoro.update_leds(&mut leds).unwrap();
        pomodoro.display(&mut display);
    })?;

    timer.every(Duration::from_secs(1))?;

    loop {
        sleep(Duration::from_secs(420));
    }
}
