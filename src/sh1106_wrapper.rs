use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

pub trait Sh1106Display : DrawTarget<Color = BinaryColor> {
    fn flush_ignore(&mut self);
    fn clear_ignore(&mut self);
}

impl<DI : sh1106::interface::DisplayInterface> Sh1106Display for sh1106::mode::GraphicsMode<DI> {
    fn flush_ignore(&mut self) {
        if self.flush().is_err() {
            log::error!("Display flushing error")
        }
    }

    fn clear_ignore(&mut self) {
        self.clear()
    }
}