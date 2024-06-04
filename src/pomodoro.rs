use std::f32::consts::PI;

use embedded_graphics::{
    geometry::{self, Angle, Point},
    mono_font,
    pixelcolor::BinaryColor,
    primitives::{Arc, Circle, Primitive, PrimitiveStyle},
    text, Drawable,
};
use esp_idf_svc::sys::EspError;

use crate::{
    led::{Led, Leds},
    sh1106_wrapper::Sh1106Display,
};

const UTC_OFFSET_H: i8 = parse_i64(env!("UTC_OFFSET_H")) as i8;
const UTC_OFFSET: time::UtcOffset = match time::UtcOffset::from_hms(UTC_OFFSET_H, 0, 0) {
    Ok(offset) => offset,
    Err(_) => panic!(),
};

const WORK_DUR: time::Duration = time::Duration::minutes(parse_u64(env!("WORK_MIN")) as i64);
const BREAK_DUR: time::Duration = time::Duration::minutes(parse_u64(env!("BREAK_MIN")) as i64);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Phase {
    Work,
    Break,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pause {
    Paused,
    Running,
}

pub struct Pomodoro {
    phase: Phase,
    pause: Pause,
    cycles_counter: u8,
    update_time: time::OffsetDateTime,
    remaining_phase_time: time::Duration,
}

impl Pomodoro {
    const FONT: mono_font::MonoFont<'static> = mono_font::ascii::FONT_9X15;
    const CHAR_STYLE: mono_font::MonoTextStyle<'static, BinaryColor> =
        mono_font::MonoTextStyle::new(&Self::FONT, BinaryColor::On);
    const TEXT_STYLE: text::TextStyle = text::TextStyleBuilder::new()
        .baseline(text::Baseline::Top)
        .build();
    const STROKE_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 2);

    const LINE_WIDTH: u8 = 128 / 9;
    const LINE_HEIGHT: u8 = 16;

    pub fn init(start_phase: Phase, start_pause: Pause) -> Self {
        let update_time = time::OffsetDateTime::now_utc().to_offset(UTC_OFFSET);
        let remaining_phase_time = Self::get_phase_time(start_phase);

        Self {
            phase: start_phase,
            pause: start_pause,
            cycles_counter: 0,
            update_time,
            remaining_phase_time,
        }
    }

    pub fn set_pause(&mut self, pause: Pause) {
        self.pause = pause;
    }

    pub fn update(&mut self) {
        let current_time = time::OffsetDateTime::now_utc().to_offset(UTC_OFFSET);
        let diff = current_time - self.update_time;
        self.update_time = current_time;

        if self.pause == Pause::Paused {
            return;
        }

        self.remaining_phase_time -= diff;

        if self.remaining_phase_time.is_negative() {
            self.phase = match self.phase {
                Phase::Work => Phase::Break,
                Phase::Break => Phase::Work,
            };
            self.cycles_counter += 1;
            self.pause = Pause::Paused;
            self.remaining_phase_time = Self::get_phase_time(self.phase);
        }
    }

    pub fn update_leds(&self, leds: &mut impl Leds) -> Result<(), EspError> {
        match self.pause {
            Pause::Paused => leds.yellow().set_high()?,
            Pause::Running => leds.yellow().set_low()?,
        }

        match self.phase {
            Phase::Work => {
                leds.green().set_high()?;
                leds.red().set_low()?
            }
            Phase::Break => {
                leds.green().set_low()?;
                leds.red().set_high()?;
            }
        }

        Ok(())
    }

    pub fn display(&self, display: &mut impl Sh1106Display) {
        use std::io::Write;

        display.clear_ignore();

        let mut inner = || -> Option<()> {
            let mut line_buf = [b' '; Self::LINE_WIDTH as usize];
            write!(
                &mut line_buf[..],
                ">> {:>2}:{:02}:{:02} <<",
                self.update_time.hour(),
                self.update_time.minute(),
                self.update_time.second()
            )
            .ok()?;
            let line_str = std::str::from_utf8(&line_buf).ok()?;
            self.draw_text(display, line_str, geometry::Point::new(0, 0));

            let mut line_buf = [b' '; Self::LINE_WIDTH as usize];
            write!(
                &mut line_buf[..],
                "Left: {:>3}:{:02}",
                self.remaining_phase_time.whole_seconds() / 60,
                self.remaining_phase_time.whole_seconds() % 60,
            )
            .ok()?;
            let line_str = std::str::from_utf8(&line_buf).ok()?;
            self.draw_text(
                display,
                line_str,
                geometry::Point::new(0, Self::LINE_HEIGHT as i32),
            );

            let line_str = match self.pause {
                Pause::Paused => "Paused",
                Pause::Running => "Running",
            };
            self.draw_text(
                display,
                line_str,
                geometry::Point::new(0, Self::LINE_HEIGHT as i32 * 2),
            );

            let phase = match self.phase {
                Phase::Work => "Work",
                Phase::Break => "Break",
            };
            let mut line_buf = [b' '; Self::LINE_WIDTH as usize];
            write!(
                &mut line_buf[..],
                "{} {:02}",
                phase,
                self.cycles_counter / 2,
            )
            .ok()?;
            let line_str = std::str::from_utf8(&line_buf).ok()?;
            self.draw_text(
                display,
                line_str,
                geometry::Point::new(0, Self::LINE_HEIGHT as i32 * 3),
            );

            let time_percent = self.remaining_phase_time.as_seconds_f32()
                / Self::get_phase_time(self.phase).as_seconds_f32();
            let time_angle = time_percent * 2. * PI;

            let circle = Circle::new(Point::new(96, 32), 28);

            Arc::from_circle(
                circle,
                Angle::from_radians(- PI / 2.),
                Angle::from_radians(time_angle),
            )
            .into_styled(Self::STROKE_STYLE)
            .draw(display)
            .ok()?;

            Some(())
        };

        if inner().is_none() {
            log::error!("Couldn't draw pomodoro");
        }

        display.flush_ignore()
    }

    fn draw_text(&self, display: &mut impl Sh1106Display, text: &str, position: geometry::Point) {
        _ = text::Text::with_text_style(text, position, Self::CHAR_STYLE, Self::TEXT_STYLE)
            .draw(display);
    }

    fn get_phase_time(phase: Phase) -> time::Duration {
        match phase {
            Phase::Work => WORK_DUR,
            Phase::Break => BREAK_DUR,
        }
    }
}

/// This function is needed because i8::from_str_radix() isn't const yet
///
/// https://github.com/rust-lang/rust/pull/99322
const fn parse_i64(string: &str) -> i64 {
    let mut bytes = string.as_bytes();
    let mut value = 0;

    let is_neg = bytes[0] == b'-';

    while let [byte, rest @ ..] = bytes {
        if *byte == b'-' {
            continue;
        }
        assert!(b'0' <= *byte && *byte <= b'9', "Invalid digit");
        value = value * 10 + (*byte - b'0') as i64;
        bytes = rest;
    }

    if is_neg {
        -value
    } else {
        value
    }
}

/// This function is needed because i8::from_str_radix() isn't const yet
///
/// https://github.com/rust-lang/rust/pull/99322
const fn parse_u64(string: &str) -> u64 {
    let mut bytes = string.as_bytes();
    let mut value = 0;

    while let [byte, rest @ ..] = bytes {
        assert!(b'0' <= *byte && *byte <= b'9', "Invalid digit");
        value = value * 10 + (*byte - b'0') as u64;
        bytes = rest;
    }

    value
}
