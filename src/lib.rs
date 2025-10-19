use imgui::DrawData;
use sdl2::render::{Canvas, RenderTarget};

pub struct CanvasRenderer {}

impl CanvasRenderer {
    pub fn render<T: RenderTarget>(
        &mut self,
        data: &DrawData,
        canvas: &mut Canvas<T>,
    ) -> Result<(), String> {
        Ok(())
    }
}
