use embedded_graphics::{geometry, mono_font, pixelcolor::BinaryColor, text, Drawable};

use crate::sh1106_wrapper::Sh1106Display;

pub trait FeedbackSystem {
    fn write_msg(&mut self, msg: &str);
}

pub struct Sh1106Feedback<D: Sh1106Display> {
    display: D,
    current_line: u8,
}

impl<D: Sh1106Display> Sh1106Feedback<D> {
    const FONT: mono_font::MonoFont<'static> = mono_font::ascii::FONT_5X8;
    const CHAR_STYLE: mono_font::MonoTextStyle<'static, BinaryColor> =
        mono_font::MonoTextStyle::new(&Self::FONT, BinaryColor::On);
    const LINE_SPACING: u8 = 10;
    const MAX_LINES: u8 = 64 / Self::LINE_SPACING;
    const TEXT_STYLE: text::TextStyle = text::TextStyleBuilder::new()
        .baseline(text::Baseline::Top)
        .build();

    pub fn from_display(display: D) -> Self {
        Self {
            display,
            current_line: 0,
        }
    }

    pub fn take_display(mut self) -> D {
        self.display.clear_ignore();
        self.display.flush_ignore();
        self.display
    }

    pub fn write_msg(&mut self, msg: &str) {
        if self.current_line >= Self::MAX_LINES {
            log::error!("Maximum feedback lines");
            return;
        }

        let position = geometry::Point::new(0, (self.current_line * Self::LINE_SPACING) as i32);
        self.current_line += 1;

        let res = text::Text::with_text_style(msg, position, Self::CHAR_STYLE, Self::TEXT_STYLE)
            .draw(&mut self.display);

        if res.is_err() {
            log::error!("Drawing error while writing feedback");
        }

        self.display.flush_ignore();

    }
}

impl<D: Sh1106Display> FeedbackSystem for Sh1106Feedback<D> {
    fn write_msg(&mut self, msg: &str) {
        self.write_msg(msg);
    }
}
