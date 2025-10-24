use imgui::DrawData;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::sys::{SDL_Color, SDL_FPoint, SDL_RenderGeometry, SDL_Texture, SDL_Vertex};
use std::ffi::c_int;

pub struct CanvasRenderer {}

impl CanvasRenderer {
    pub fn new(imgui_context: &mut imgui::Context) -> Result<Self, String> {
        imgui_context.fonts().build_rgba32_texture();
        Ok(Self {})
    }

    pub fn render<T: RenderTarget>(
        &mut self,
        data: &DrawData,
        canvas: &mut Canvas<T>,
    ) -> Result<(), String> {
        let (framebuffer_w, framebuffer_h) = (
            data.display_size[0] * data.framebuffer_scale[0],
            data.display_size[1] * data.framebuffer_scale[1],
        );
        if framebuffer_w <= 0.0 && framebuffer_h <= 0.0 {
            return Ok(());
        }

        // Draw Lists
        for draw_list in data.draw_lists() {
            let vtx: Vec<SDL_Vertex> = draw_list
                .vtx_buffer()
                .iter()
                .map(|v| SDL_Vertex {
                    position: SDL_FPoint {
                        x: v.pos[0] * data.framebuffer_scale[0],
                        y: v.pos[1] * data.framebuffer_scale[1],
                    },
                    color: SDL_Color {
                        r: v.col[0],
                        g: v.col[1],
                        b: v.col[2],
                        a: v.col[3],
                    },
                    tex_coord: SDL_FPoint {
                        x: v.uv[0],
                        y: v.uv[1],
                    },
                })
                .collect();
            let vtx_len = vtx.len() as c_int;
            let vtx_ptr = if vtx.is_empty() {
                std::ptr::null()
            } else {
                vtx.as_ptr()
            };

            let idxs: Vec<c_int> = draw_list.idx_buffer().iter().map(|i| *i as c_int).collect();
            let idxs_len = idxs.len() as c_int;
            let idxs_ptr = if idxs_len == 0 {
                std::ptr::null()
            } else {
                idxs.as_ptr() as *const c_int
            };

            let rv = unsafe {
                SDL_RenderGeometry(
                    canvas.raw(),
                    std::ptr::null::<SDL_Texture>() as *mut SDL_Texture,
                    vtx_ptr,
                    vtx_len,
                    idxs_ptr,
                    idxs_len,
                )
            };
            if rv != 0 {
                return Err(format!(
                    "Could not render geometry with SDL_RenderGeometry: {}",
                    rv
                ));
            }
        }

        Ok(())
    }
}
