use imgui::internal::RawWrapper;
use imgui::{DrawCmd, DrawCmdParams, DrawData, TextureId};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use sdl2::surface::Surface;
use sdl2::sys::{SDL_Color, SDL_FPoint, SDL_RenderGeometry, SDL_Texture, SDL_Vertex};
use std::ffi::c_int;

pub struct CanvasRenderer {
    textures: Vec<Texture>,
}

impl CanvasRenderer {
    pub fn new<T: RenderTarget>(
        imgui_context: &mut imgui::Context,
        canvas: &mut Canvas<T>,
    ) -> Result<Self, String> {
        let imgui_font = imgui_context.fonts().build_rgba32_texture();
        let mut font_data = imgui_font.data.to_vec();
        let font_surface = Surface::from_data(
            font_data.as_mut_slice(),
            imgui_font.width,
            imgui_font.height,
            imgui_font.width * 4,
            PixelFormatEnum::ABGR8888,
        )?;
        let font_atlas_texture = canvas
            .create_texture_from_surface(font_surface)
            .map_err(|e| format!("Could not create texture from font surface: {}", e))?;
        let textures = vec![font_atlas_texture];
        imgui_context.fonts().tex_id = TextureId::new(0);
        Ok(Self { textures })
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

            for command in draw_list.commands() {
                match command {
                    DrawCmd::Elements { count, cmd_params } => self.render_elements(
                        count,
                        cmd_params,
                        data,
                        framebuffer_w,
                        framebuffer_h,
                        canvas,
                    )?,
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                    DrawCmd::ResetRenderState => {
                        //TODO implement me
                    },
                }
            }
        }

        Ok(())
    }

    fn render_elements<T: RenderTarget>(
        &self,
        _count: usize,
        draw_cmd_params: DrawCmdParams,
        data: &DrawData,
        fb_w: f32,
        fb_h: f32,
        canvas: &mut Canvas<T>,
    ) -> Result<(), String> {
        let DrawCmdParams {
            clip_rect,
            texture_id,
            vtx_offset: _,
            idx_offset: _,
        } = draw_cmd_params;
        let clip_off = data.display_pos;
        let scale = data.framebuffer_scale;

        let clip_x1 = (clip_rect[0] - clip_off[0]) * scale[0];
        let clip_y1 = (clip_rect[1] - clip_off[1]) * scale[1];
        let clip_x2 = (clip_rect[2] - clip_off[0]) * scale[0];
        let clip_y2 = (clip_rect[3] - clip_off[1]) * scale[1];

        if clip_x1 >= fb_w || clip_y1 >= fb_h || clip_x2 < 0.0 || clip_y2 < 0.0 {
            return Ok(());
        }
        if texture_id.id() >= self.textures.len() {
            return Err(format!("Texture ID out of range: {}", texture_id.id()));
        }
        let _texture = &self.textures[texture_id.id()];
        canvas.set_clip_rect(Rect::new(
            clip_x1 as i32,
            clip_y1 as i32,
            (clip_x2 - clip_x1) as u32,
            (clip_y2 - clip_y1) as u32,
        ));
        // canvas.copy(texture, None, None)?;
        //TODO ???
        canvas.set_clip_rect(None);
        Ok(())
    }
}
