//! OpenGL ES 2.0 backend built on `glow`.
//!
//! The host (Android EGL, or the desktop harness) creates the GL context and hands
//! over a [`glow::Context`]. Dropping the context's resources is explicit
//! ([`Gles2Backend::destroy`]) so the app can release all GPU memory when it sleeps.

use super::batch::{
    Frame, Program, RectVertex, TexVertex, Texture as ImageTexture, MAX_QUADS_PER_BATCH,
    QUAD_INDICES,
};
use super::geometry::Rect;
use super::shaders::{
    BAYER_4X4, GLASS_FRAGMENT, IMAGE_FRAGMENT, RECT_FRAGMENT, RECT_VERTEX, REDUCE_FRAGMENT,
    REDUCE_VERTEX, TEXT_FRAGMENT, TEX_VERTEX, WALLPAPER_FRAGMENT, WALLPAPER_VERTEX,
};
use super::{Backend, Viewport};
use crate::text::{TextEngine, Update};
use crate::theme::Color;
use crate::wallpaper::WallpaperId;
use glow::HasContext;
use std::mem::size_of;

/// A wallpaper going up to the GPU: the decoded image and how many rows are already there.
struct WallpaperUpload {
    id: WallpaperId,
    cover: (u32, u32),
    rgb: Vec<u8>,
    rows_done: u32,
    source: glow::Texture,
    started: std::time::Instant,
    loaded: std::time::Duration,
}

/// Rows of the wallpaper sent to the GPU per frame while it replaces another one (about 2 ms on the TV).
const WALLPAPER_UPLOAD_ROWS: u32 = 90;

struct ShaderProgram {
    program: glow::Program,
    u_view: Option<glow::UniformLocation>,
    u_surface: Option<glow::UniformLocation>,
    u_texture: Option<glow::UniformLocation>,
    u_bayer: Option<glow::UniformLocation>,
    u_step: Option<glow::UniformLocation>,
    u_dest: Option<glow::UniformLocation>,
}

#[derive(Clone, Copy)]
struct Texture {
    texture: glow::Texture,
    width: u32,
    height: u32,
}

pub struct Gles2Backend {
    gl: glow::Context,
    rect_program: ShaderProgram,
    text_program: ShaderProgram,
    image_program: ShaderProgram,
    glass_program: ShaderProgram,
    wallpaper_program: ShaderProgram,
    reduce_program: ShaderProgram,
    rect_vbo: glow::Buffer,
    tex_vbo: glow::Buffer,
    reduce_vbo: glow::Buffer,
    index_buffer: glow::Buffer,
    index_quads: u32,
    tex_staging: Vec<TexVertex>,
    atlas: Option<Texture>,
    image: Option<Texture>,
    /// The wallpaper, created the first time a frame draws it and freed with everything else.
    wallpaper: Option<Texture>,
    wallpaper_id: Option<WallpaperId>,
    /// The size the wallpaper texture covers, in render-buffer pixels (the 1920x1080 image is
    /// reduced to it when smaller).
    wallpaper_cover: (u32, u32),
    /// The render buffer's size, set by the host and by every frame.
    render_size: (u32, u32),
    /// Whether a wallpaper that is not on the GPU yet is prepared on a worker thread (the frame loop
    /// keeps the current one until the new one is ready) instead of inside the frame.
    async_wallpaper: bool,
    /// The wallpaper being prepared on the worker thread.
    pending_wallpaper: Option<(WallpaperId, std::sync::mpsc::Receiver<Option<Vec<u8>>>)>,
    uploading: Option<WallpaperUpload>,
    /// The wallpaper whose asset could not be read or decoded; not tried again.
    wallpaper_failed: Option<WallpaperId>,
    wallpaper_thumbs: Option<Texture>,
    /// The 4x4 dither pattern the wallpaper is drawn with, made with the wallpaper.
    bayer: Option<Texture>,
    /// Whether the wallpaper is dithered (tests compare it with and without).
    dither: bool,
    /// Whether a texture or the buffer changed since [`Gles2Backend::take_content_changed`]: what
    /// a comparison of frames cannot see, so the next frame must be drawn whole.
    content_changed: bool,
}

impl Gles2Backend {
    /// Compiles the shaders and creates the buffers. Needs a current GL context.
    pub fn new(gl: glow::Context) -> Result<Gles2Backend, String> {
        // SAFETY: the host guarantees a current GLES2-capable context for `gl`.
        unsafe {
            let rect_program = link(
                &gl,
                RECT_VERTEX,
                RECT_FRAGMENT,
                &[
                    "a_pos_local",
                    "a_shape",
                    "a_misc",
                    "a_stops",
                    "a_c0",
                    "a_c1",
                    "a_c2",
                ],
            )?;
            let text_program = link(&gl, TEX_VERTEX, TEXT_FRAGMENT, &["a_pos_uv", "a_color"])?;
            let image_program = link(
                &gl,
                TEX_VERTEX,
                IMAGE_FRAGMENT,
                &["a_pos_uv", "a_color", "a_local", "a_params", "a_bg"],
            )?;
            let glass_program = link(
                &gl,
                TEX_VERTEX,
                GLASS_FRAGMENT,
                &["a_pos_uv", "a_color", "a_local", "a_params", "a_bg"],
            )?;
            let wallpaper_program = link(
                &gl,
                WALLPAPER_VERTEX,
                WALLPAPER_FRAGMENT,
                &["a_pos_uv", "a_color"],
            )?;
            let reduce_program = link(&gl, REDUCE_VERTEX, REDUCE_FRAGMENT, &["a_pos"])?;
            let rect_vbo = gl.create_buffer()?;
            let tex_vbo = gl.create_buffer()?;
            let reduce_vbo = gl.create_buffer()?;
            let quad: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(reduce_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad),
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            let index_buffer = gl.create_buffer()?;
            Ok(Gles2Backend {
                gl,
                rect_program,
                text_program,
                image_program,
                glass_program,
                wallpaper_program,
                reduce_program,
                rect_vbo,
                tex_vbo,
                reduce_vbo,
                index_buffer,
                index_quads: 0,
                tex_staging: Vec::new(),
                atlas: None,
                image: None,
                wallpaper: None,
                wallpaper_id: None,
                wallpaper_cover: (0, 0),
                render_size: (1920, 1080),
                async_wallpaper: false,
                pending_wallpaper: None,
                uploading: None,
                wallpaper_failed: None,
                wallpaper_thumbs: None,
                bayer: None,
                dither: true,
                content_changed: true,
            })
        }
    }

    /// Creates (or replaces) the 8-bit glyph atlas.
    pub fn set_atlas(&mut self, width: u32, height: u32, pixels: &[u8]) {
        self.content_changed = true;
        assert_eq!(
            pixels.len(),
            (width * height) as usize,
            "the atlas is 8 bits per pixel"
        );
        // SAFETY: see `new`.
        unsafe {
            let texture = match self.atlas.take() {
                Some(old) => old.texture,
                None => self
                    .gl
                    .create_texture()
                    .expect("cannot create the atlas texture"),
            };
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::ALPHA as i32,
                width as i32,
                height as i32,
                0,
                glow::ALPHA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            set_filtering(&self.gl);
            self.atlas = Some(Texture {
                texture,
                width,
                height,
            });
        }
    }

    /// Updates a region of the atlas in place.
    pub fn update_atlas(&mut self, region: [u32; 4], pixels: &[u8]) {
        // Not a change of content: a slot filled now is only read by glyph quads that are new in
        // this frame, which the comparison of frames already redraws.
        let [x, y, w, h] = region;
        assert_eq!(pixels.len(), (w * h) as usize);
        let atlas = self.atlas.as_ref().expect("set_atlas must be called first");
        assert!(
            x + w <= atlas.width && y + h <= atlas.height,
            "the region is outside the atlas"
        );
        // SAFETY: see `new`.
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(atlas.texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                glow::ALPHA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
        }
    }

    /// Creates (or replaces) the RGBA image texture used by [`Program::Image`].
    pub fn set_image(&mut self, width: u32, height: u32, rgba: &[u8]) {
        self.content_changed = true;
        assert_eq!(rgba.len(), (width * height * 4) as usize);
        // SAFETY: see `new`.
        unsafe {
            let texture = match self.image.take() {
                Some(old) => old.texture,
                None => self
                    .gl
                    .create_texture()
                    .expect("cannot create the image texture"),
            };
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(rgba)),
            );
            set_filtering(&self.gl);
            self.image = Some(Texture {
                texture,
                width,
                height,
            });
        }
    }

    /// Creates the RGBA image texture empty, if it does not exist yet at this size (a texture of
    /// another size, from another scale, is freed first). Regions are filled in by
    /// [`Gles2Backend::update_image`].
    pub fn ensure_image(&mut self, width: u32, height: u32) {
        match self.image {
            Some(image) if (image.width, image.height) == (width, height) => return,
            // SAFETY: see `new`.
            Some(image) => unsafe { self.gl.delete_texture(image.texture) },
            None => {}
        }
        self.image = None;
        // SAFETY: see `new`.
        unsafe {
            let texture = self
                .gl
                .create_texture()
                .expect("cannot create the image texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            set_filtering(&self.gl);
            self.image = Some(Texture {
                texture,
                width,
                height,
            });
        }
    }

    /// Replaces a region (`[x, y, width, height]`) of the image texture with premultiplied RGBA
    /// pixels. [`Gles2Backend::ensure_image`] must have been called first.
    pub fn update_image(&mut self, region: [u32; 4], rgba: &[u8]) {
        self.content_changed = true;
        let [x, y, w, h] = region;
        assert_eq!(rgba.len(), (w * h * 4) as usize);
        let image = self
            .image
            .as_ref()
            .expect("ensure_image must be called first");
        assert!(
            x + w <= image.width && y + h <= image.height,
            "the region is outside the image"
        );
        // SAFETY: see `new`.
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(image.texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(rgba)),
            );
        }
    }

    /// Creates or updates the wallpaper texture (opaque RGB, linear filter, clamped to the edge) from
    /// [`WallpaperId`], if not already loaded. If a different wallpaper was previously loaded, its
    /// texture data is updated. Until then a frame has only the clear color behind its content.
    pub fn ensure_wallpaper(&mut self, id: WallpaperId) {
        // Waiting is only acceptable when there is nothing to show yet: the first frame needs a wallpaper.
        let block = !self.async_wallpaper || self.wallpaper.is_none();
        self.prepare_wallpaper(id, block);
    }

    /// Makes the host prepare wallpapers on a worker thread: [`ensure_wallpaper`](Self::ensure_wallpaper)
    /// then returns at once while a new one is being prepared, and the frame keeps the current one.
    pub fn set_async_wallpaper(&mut self, enabled: bool) {
        self.async_wallpaper = enabled;
    }

    /// Like [`ensure_wallpaper`](Self::ensure_wallpaper) but never waits: starts the preparation on
    /// the worker thread and uploads the wallpaper once it is ready (called again by the host's loop).
    pub fn poll_wallpaper(&mut self, id: WallpaperId) {
        self.prepare_wallpaper(id, !self.async_wallpaper);
    }

    fn prepare_wallpaper(&mut self, id: WallpaperId, block: bool) {
        let cover = cover_size(self.render_size);
        if let Some(upload) = &self.uploading {
            if (upload.id, upload.cover) == (id, cover) {
                self.step_wallpaper_upload(block);
                return;
            }
            // The choice changed while it was going up: that one is dropped.
            if let Some(upload) = self.uploading.take() {
                // SAFETY: see `new`.
                unsafe { self.gl.delete_texture(upload.source) };
            }
        }
        if self.wallpaper.is_some()
            && self.wallpaper_id == Some(id)
            && self.wallpaper_cover == cover
        {
            return;
        }
        self.content_changed = true;
        let wallpaper = id.wallpaper();
        if self.wallpaper_failed == Some(id) {
            return;
        }
        let started = std::time::Instant::now();
        let loaded_rgb = if self.async_wallpaper {
            // Only the last choice is prepared: a different pending one is dropped (its thread ends alone).
            if self.pending_wallpaper.as_ref().map(|(pending, _)| *pending) != Some(id) {
                self.pending_wallpaper = Some((id, crate::wallpaper::load_in_background(id)));
            }
            let Some((_, receiver)) = self.pending_wallpaper.as_ref() else {
                return;
            };
            let result = if block {
                receiver.recv().map_err(|_| ())
            } else {
                match receiver.try_recv() {
                    Ok(rgb) => Ok(rgb),
                    Err(std::sync::mpsc::TryRecvError::Empty) => return,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(()),
                }
            };
            self.pending_wallpaper = None;
            result.unwrap_or(None)
        } else {
            wallpaper.load_rgb()
        };
        let Some(rgb) = loaded_rgb else {
            // Tried once: the frames keep the clear color instead of retrying (and logging) every frame.
            log::error!("cannot load the wallpaper {}", id.as_str());
            self.wallpaper_failed = Some(id);
            return;
        };
        let loaded = started.elapsed();
        self.ensure_bayer();
        // SAFETY: see `new`.
        let source = unsafe {
            let texture = self
                .gl
                .create_texture()
                .expect("cannot create the wallpaper source texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                wallpaper.width as i32,
                wallpaper.height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            set_filtering(&self.gl);
            texture
        };
        self.uploading = Some(WallpaperUpload {
            id,
            cover,
            rgb,
            rows_done: 0,
            source,
            started,
            loaded,
        });
        // A wallpaper that replaces another goes up a slice per frame, so no frame is late for it.
        self.step_wallpaper_upload(!(self.async_wallpaper && self.wallpaper.is_some()));
    }

    /// Whether a wallpaper is still going up to the GPU, slice by slice: the host keeps drawing frames.
    pub fn wallpaper_uploading(&self) -> bool {
        self.uploading.is_some()
    }

    /// Sends the next slice of the wallpaper being uploaded (all of it when `all`) and, with the last
    /// one, reduces it if the render buffer is smaller and swaps it in.
    fn step_wallpaper_upload(&mut self, all: bool) {
        let Some(mut upload) = self.uploading.take() else {
            return;
        };
        self.content_changed = true;
        let wallpaper = upload.id.wallpaper();
        let rows = if all {
            wallpaper.height
        } else {
            WALLPAPER_UPLOAD_ROWS
        }
        .min(wallpaper.height - upload.rows_done);
        let row_bytes = (wallpaper.width * 3) as usize;
        let from = upload.rows_done as usize * row_bytes;
        // SAFETY: see `new`.
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(upload.source));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                upload.rows_done as i32,
                wallpaper.width as i32,
                rows as i32,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(
                    &upload.rgb[from..from + rows as usize * row_bytes],
                )),
            );
        }
        upload.rows_done += rows;
        if upload.rows_done < wallpaper.height {
            self.uploading = Some(upload);
            return;
        }
        let WallpaperUpload {
            id,
            cover,
            source,
            started,
            loaded,
            ..
        } = upload;
        let reduce = cover.0 < wallpaper.width && cover.1 < wallpaper.height;
        // SAFETY: see `new`.
        unsafe {
            let mut texture = Texture {
                texture: source,
                width: wallpaper.width,
                height: wallpaper.height,
            };
            if reduce {
                let destination = match self.wallpaper {
                    Some(existing) => existing.texture,
                    None => self
                        .gl
                        .create_texture()
                        .expect("cannot create the wallpaper texture"),
                };
                if self.reduce_wallpaper(source, destination, cover) {
                    self.gl.delete_texture(source);
                    self.wallpaper = None;
                    texture = Texture {
                        texture: destination,
                        width: cover.0,
                        height: cover.1,
                    };
                } else if self.wallpaper.is_none() {
                    // The reduction is not available (no renderable texture): the image as it is,
                    // which the GPU reduces with its linear filter at every frame.
                    self.gl.delete_texture(destination);
                }
            }
            if let Some(previous) = self.wallpaper.take() {
                self.gl.delete_texture(previous.texture);
            }
            self.wallpaper = Some(texture);
            self.wallpaper_id = Some(id);
            self.wallpaper_cover = cover;
        }
        log::info!(
            "wallpaper {} ready: load {} ms, total {} ms ({}x{})",
            id.as_str(),
            loaded.as_millis(),
            started.elapsed().as_millis(),
            self.wallpaper.map_or(0, |t| t.width),
            self.wallpaper.map_or(0, |t| t.height)
        );
    }

    /// Draws `source` (the full image) into `destination`, a texture of `cover` pixels, with the
    /// reduction shader. Returns false, with the GL state restored, when the framebuffer is not complete.
    ///
    /// # Safety
    /// Needs the current GL context; the textures must belong to it.
    unsafe fn reduce_wallpaper(
        &mut self,
        source: glow::Texture,
        destination: glow::Texture,
        cover: (u32, u32),
    ) -> bool {
        let gl = &self.gl;
        let previous_framebuffer = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING);
        let mut previous_viewport = [0i32; 4];
        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut previous_viewport);

        gl.bind_texture(glow::TEXTURE_2D, Some(destination));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            cover.0 as i32,
            cover.1 as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        set_filtering(gl);
        let framebuffer = gl
            .create_framebuffer()
            .expect("cannot create the framebuffer");
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(destination),
            0,
        );
        let complete = gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
        if complete {
            gl.viewport(0, 0, cover.0 as i32, cover.1 as i32);
            gl.disable(glow::BLEND);
            gl.disable(glow::SCISSOR_TEST);
            let program = &self.reduce_program;
            gl.use_program(Some(program.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(source));
            gl.uniform_1_i32(program.u_texture.as_ref(), 0);
            if let Some(bayer) = self.bayer {
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(bayer.texture));
                gl.uniform_1_i32(program.u_bayer.as_ref(), 1);
                gl.active_texture(glow::TEXTURE0);
            }
            // A quarter of the footprint of a destination pixel, in source texture coordinates.
            let step = [0.25 / cover.0 as f32, 0.25 / cover.1 as f32];
            gl.uniform_2_f32(self.reduce_program.u_step.as_ref(), step[0], step[1]);
            gl.uniform_2_f32(
                self.reduce_program.u_dest.as_ref(),
                cover.0 as f32,
                cover.1 as f32,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.reduce_vbo));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.disable_vertex_attrib_array(0);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        gl.bind_framebuffer(
            glow::FRAMEBUFFER,
            std::num::NonZeroU32::new(previous_framebuffer as u32).map(glow::NativeFramebuffer),
        );
        gl.delete_framebuffer(framebuffer);
        gl.viewport(
            previous_viewport[0],
            previous_viewport[1],
            previous_viewport[2],
            previous_viewport[3],
        );
        complete
    }

    /// Tells the backend the size of the render buffer, so the wallpaper is reduced to it when the
    /// buffer is smaller than the image. Every frame also sets it from its viewport.
    pub fn set_render_size(&mut self, width: u32, height: u32) {
        self.content_changed |= self.render_size != (width.max(1), height.max(1));
        self.render_size = (width.max(1), height.max(1));
    }

    /// Creates the 4x4 dither texture of the wallpaper, repeated over the screen and read texel by
    /// texel; without dither, a flat one at the middle level (which adds nothing).
    fn ensure_bayer(&mut self) {
        let pattern = if self.dither { BAYER_4X4 } else { [128; 16] };
        // SAFETY: see `new`.
        unsafe {
            let texture = match self.bayer {
                Some(existing) => existing.texture,
                None => self
                    .gl
                    .create_texture()
                    .expect("cannot create the dither texture"),
            };
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::LUMINANCE as i32,
                4,
                4,
                0,
                glow::LUMINANCE,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&pattern)),
            );
            for (parameter, value) in [
                (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
                (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
                (glow::TEXTURE_WRAP_S, glow::REPEAT),
                (glow::TEXTURE_WRAP_T, glow::REPEAT),
            ] {
                self.gl
                    .tex_parameter_i32(glow::TEXTURE_2D, parameter, value as i32);
            }
            self.bayer = Some(Texture {
                texture,
                width: 4,
                height: 4,
            });
        }
    }

    /// Turns the dither of the wallpaper on or off (for tests that compare the two).
    pub fn set_wallpaper_dither(&mut self, enabled: bool) {
        self.content_changed = true;
        self.dither = enabled;
        if self.bayer.is_some() {
            self.ensure_bayer();
        }
    }

    /// The ID of the currently loaded wallpaper, if any.
    pub fn current_wallpaper(&self) -> Option<WallpaperId> {
        self.wallpaper_id
    }

    /// Uploads the 640x144 RGBA wallpaper thumbnails atlas texture if not already present.
    pub fn ensure_wallpaper_thumbs(&mut self) {
        if self.wallpaper_thumbs.is_some() {
            return;
        }
        self.content_changed = true;
        let rgba = crate::wallpaper::decompress_thumbnails();
        assert_eq!(
            rgba.len(),
            (crate::wallpaper::THUMBNAILS_WIDTH * crate::wallpaper::THUMBNAILS_HEIGHT * 4) as usize
        );
        // SAFETY: see `new`.
        unsafe {
            let texture = self
                .gl
                .create_texture()
                .expect("cannot create the wallpaper thumbnails texture");
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                crate::wallpaper::THUMBNAILS_WIDTH as i32,
                crate::wallpaper::THUMBNAILS_HEIGHT as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&rgba)),
            );
            set_filtering(&self.gl);
            self.wallpaper_thumbs = Some(Texture {
                texture,
                width: crate::wallpaper::THUMBNAILS_WIDTH,
                height: crate::wallpaper::THUMBNAILS_HEIGHT,
            });
        }
    }

    /// Whether the wallpaper texture exists.
    /// The GPU's name and the OpenGL ES version the driver reports. The context must be current.
    pub fn gpu_strings(&self) -> (String, String) {
        use glow::HasContext;
        // SAFETY: plain queries of the current context.
        unsafe {
            (
                self.gl.get_parameter_string(glow::RENDERER),
                self.gl.get_parameter_string(glow::VERSION),
            )
        }
    }

    pub fn has_wallpaper(&self) -> bool {
        self.wallpaper.is_some()
    }

    /// Uploads whatever changed in the text engine's atlas since the last call.
    pub fn sync_text_atlas(&mut self, engine: &mut TextEngine) {
        let updates = engine.take_updates();
        if updates.iter().any(|u| matches!(u, Update::Full { .. })) {
            // The bitmap changed shape or content as a whole; regions are already included.
            let atlas = engine.atlas();
            self.set_atlas(atlas.width(), atlas.height(), atlas.pixels());
            return;
        }
        for update in updates {
            if let Update::Region(slot) = update {
                let pixels = engine.atlas().region_pixels(slot);
                self.update_atlas([slot.x, slot.y, slot.width, slot.height], &pixels);
            }
        }
    }

    pub fn atlas_size(&self) -> Option<(u32, u32)> {
        self.atlas.as_ref().map(|t| (t.width, t.height))
    }

    /// Reads the framebuffer back as tightly packed RGBA rows, top row first.
    pub fn read_pixels(&self, width: u32, height: u32) -> Vec<u8> {
        let mut bottom_up = vec![0u8; (width * height * 4) as usize];
        // SAFETY: see `new`; the buffer is exactly width * height * 4 bytes.
        unsafe {
            self.gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
            self.gl.read_pixels(
                0,
                0,
                width as i32,
                height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut bottom_up)),
            );
        }
        let stride = (width * 4) as usize;
        flip_rows_in_place(&mut bottom_up, stride);
        bottom_up
    }

    /// Releases every GL object. The context itself belongs to the host.
    pub fn destroy(mut self) {
        // SAFETY: see `new`.
        unsafe {
            for program in [
                &self.rect_program,
                &self.text_program,
                &self.image_program,
                &self.glass_program,
                &self.wallpaper_program,
                &self.reduce_program,
            ] {
                self.gl.delete_program(program.program);
            }
            for buffer in [
                self.rect_vbo,
                self.tex_vbo,
                self.reduce_vbo,
                self.index_buffer,
            ] {
                self.gl.delete_buffer(buffer);
            }
            for texture in [
                self.atlas,
                self.image,
                self.wallpaper,
                self.wallpaper_thumbs,
                self.bayer,
            ]
            .into_iter()
            .flatten()
            {
                self.gl.delete_texture(texture.texture);
            }
            if let Some(upload) = self.uploading.take() {
                self.gl.delete_texture(upload.source);
            }
        }
    }

    unsafe fn ensure_indices(&mut self, quads: u32) {
        if quads <= self.index_quads {
            return;
        }
        let quads = quads.clamp(64, MAX_QUADS_PER_BATCH);
        let mut indices = Vec::with_capacity(quads as usize * 6);
        for quad in 0..quads {
            let base = (quad * 4) as u16;
            indices.extend(QUAD_INDICES.iter().map(|i| base + i));
        }
        self.gl
            .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
        self.gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STATIC_DRAW,
        );
        self.index_quads = quads;
    }

    unsafe fn bind_rect_attributes(&self, first_quad: u32) {
        let stride = size_of::<RectVertex>() as i32;
        let base = first_quad as i32 * 4 * stride;
        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.rect_vbo));
        // Seven vec4 attributes, one after another.
        for index in 0..7u32 {
            self.gl.enable_vertex_attrib_array(index);
            self.gl.vertex_attrib_pointer_f32(
                index,
                4,
                glow::FLOAT,
                false,
                stride,
                base + index as i32 * 16,
            );
        }
    }

    unsafe fn bind_tex_attributes(&self, first_quad: u32) {
        let stride = size_of::<TexVertex>() as i32;
        let base = first_quad as i32 * 4 * stride;
        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.tex_vbo));
        for index in 0..5u32 {
            self.gl.enable_vertex_attrib_array(index);
            self.gl.vertex_attrib_pointer_f32(
                index,
                4,
                glow::FLOAT,
                false,
                stride,
                base + index as i32 * 16,
            );
        }
        for index in 5..7u32 {
            self.gl.disable_vertex_attrib_array(index);
        }
    }
}

impl Backend for Gles2Backend {
    fn render(&mut self, viewport: &Viewport, clear: Color, frame: &Frame) {
        self.prepare(frame);
        self.render_damaged(viewport, clear, frame, None);
    }
}

impl Gles2Backend {
    /// Puts on the GPU the textures `frame` samples. Call it before
    /// [`Gles2Backend::take_content_changed`] and [`Gles2Backend::render_damaged`].
    pub fn prepare(&mut self, frame: &Frame) {
        if frame.batches.is_empty() {
            return;
        }
        if frame.uses_wallpaper() {
            self.ensure_wallpaper(frame.wallpaper_id.unwrap_or_default());
        }
        if self.wallpaper_thumbs.is_none()
            && frame
                .batches
                .iter()
                .any(|b| b.texture == ImageTexture::WallpaperThumbs)
        {
            self.ensure_wallpaper_thumbs();
        }
    }

    /// Whether a texture or the buffer changed since the last call, so the frame cannot be
    /// limited to what [`Frame::damage`] found.
    pub fn take_content_changed(&mut self) -> bool {
        std::mem::take(&mut self.content_changed)
    }

    /// The GL scissor box (device pixels, origin bottom-left) of a logical rect.
    pub fn device_rect(rect: &Rect, viewport: &Viewport) -> [i32; 4] {
        device_scissor(rect, viewport)
    }

    /// Draws `frame` (already [`Gles2Backend::prepare`]d) only inside `damage`, a scissor box;
    /// the rest of the buffer is kept. `None` draws the whole surface.
    pub fn render_damaged(
        &mut self,
        viewport: &Viewport,
        clear: Color,
        frame: &Frame,
        damage: Option<[i32; 4]>,
    ) {
        let max_quads = frame
            .batches
            .iter()
            .map(|b| b.quad_count)
            .max()
            .unwrap_or(0);
        // SAFETY: see `new`.
        unsafe {
            let gl = &self.gl;
            self.render_size = (viewport.width.max(1), viewport.height.max(1));
            gl.viewport(0, 0, viewport.width as i32, viewport.height as i32);
            match damage {
                Some([x, y, w, h]) => {
                    gl.enable(glow::SCISSOR_TEST);
                    gl.scissor(x, y, w, h);
                }
                None => gl.disable(glow::SCISSOR_TEST),
            }
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            let [r, g, b, a] = clear.to_f32();
            gl.clear_color(r * a, g * a, b * a, a);
            gl.clear(glow::COLOR_BUFFER_BIT);
            if frame.batches.is_empty() {
                gl.disable(glow::SCISSOR_TEST);
                return;
            }
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.ensure_indices(max_quads);

            // Upload this frame's vertices.
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.rect_vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&frame.rect_vertices),
                glow::STREAM_DRAW,
            );
            // Text and image quads share one buffer: text first, then images.
            self.tex_staging.clear();
            self.tex_staging
                .reserve(frame.text_vertices.len() + frame.image_vertices.len());
            self.tex_staging.extend_from_slice(&frame.text_vertices);
            self.tex_staging.extend_from_slice(&frame.image_vertices);
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.tex_vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&self.tex_staging),
                glow::STREAM_DRAW,
            );
            let image_quad_base = (frame.text_vertices.len() / 4) as u32;

            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
            let mut clipped = false;
            for batch in &frame.batches {
                match (batch.clip, damage) {
                    (Some(clip), _) => {
                        let mut s = device_scissor(&clip, viewport);
                        if let Some(d) = damage {
                            s = intersect_scissor(s, d);
                        }
                        self.gl.enable(glow::SCISSOR_TEST);
                        self.gl.scissor(s[0], s[1], s[2], s[3]);
                        clipped = true;
                    }
                    (None, Some([x, y, w, h])) if clipped => {
                        self.gl.scissor(x, y, w, h);
                        clipped = false;
                    }
                    (None, None) if clipped => {
                        self.gl.disable(glow::SCISSOR_TEST);
                        clipped = false;
                    }
                    _ => {}
                }
                let program = match (batch.program, batch.texture) {
                    (Program::Rect, _) => &self.rect_program,
                    (Program::Text, _) => &self.text_program,
                    (Program::Image, ImageTexture::Wallpaper) => &self.wallpaper_program,
                    (Program::Image, _) => &self.image_program,
                    (Program::Glass, _) => &self.glass_program,
                };
                self.gl.use_program(Some(program.program));
                self.gl.uniform_4_f32(
                    program.u_view.as_ref(),
                    viewport.scale,
                    viewport.offset_x,
                    viewport.offset_y,
                    viewport.edge_softness,
                );
                self.gl.uniform_2_f32(
                    program.u_surface.as_ref(),
                    viewport.width as f32,
                    viewport.height as f32,
                );
                match batch.program {
                    Program::Rect => self.bind_rect_attributes(batch.first_quad),
                    Program::Text => {
                        self.bind_texture(self.atlas.as_ref(), program);
                        self.bind_tex_attributes(batch.first_quad);
                    }
                    Program::Image | Program::Glass => {
                        let texture = match batch.texture {
                            ImageTexture::Icons => self.image.as_ref(),
                            ImageTexture::Wallpaper => self.wallpaper.as_ref(),
                            ImageTexture::WallpaperThumbs => self.wallpaper_thumbs.as_ref(),
                        };
                        if batch.texture == ImageTexture::Wallpaper
                            && batch.program == Program::Image
                        {
                            // The dither pattern on unit 1, back to unit 0 for the rest.
                            self.gl.active_texture(glow::TEXTURE1);
                            self.gl
                                .bind_texture(glow::TEXTURE_2D, self.bayer.map(|t| t.texture));
                            self.gl.uniform_1_i32(program.u_bayer.as_ref(), 1);
                        }
                        self.bind_texture(texture, program);
                        self.bind_tex_attributes(image_quad_base + batch.first_quad);
                    }
                }
                self.gl.draw_elements(
                    glow::TRIANGLES,
                    batch.quad_count as i32 * 6,
                    glow::UNSIGNED_SHORT,
                    0,
                );
            }
            self.gl.disable(glow::SCISSOR_TEST);
        }
    }
}

impl Gles2Backend {
    unsafe fn bind_texture(&self, texture: Option<&Texture>, program: &ShaderProgram) {
        self.gl.active_texture(glow::TEXTURE0);
        self.gl
            .bind_texture(glow::TEXTURE_2D, texture.map(|t| t.texture));
        self.gl.uniform_1_i32(program.u_texture.as_ref(), 0);
    }
}

/// The overlap of two scissor boxes (empty when they do not meet).
fn intersect_scissor(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    let (x, y) = (a[0].max(b[0]), a[1].max(b[1]));
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let top = (a[1] + a[3]).min(b[1] + b[3]);
    [x, y, (right - x).max(0), (top - y).max(0)]
}

/// Converts a logical clip rectangle to a GL scissor box (device pixels, origin bottom-left).
fn device_scissor(clip: &Rect, viewport: &Viewport) -> [i32; 4] {
    let left = (clip.x * viewport.scale + viewport.offset_x)
        .floor()
        .max(0.0);
    let top = (clip.y * viewport.scale + viewport.offset_y)
        .floor()
        .max(0.0);
    let right = ((clip.right()) * viewport.scale + viewport.offset_x)
        .ceil()
        .min(viewport.width as f32);
    let bottom = ((clip.bottom()) * viewport.scale + viewport.offset_y)
        .ceil()
        .min(viewport.height as f32);
    let (w, h) = ((right - left).max(0.0), (bottom - top).max(0.0));
    [
        left as i32,
        (viewport.height as f32 - bottom).max(0.0) as i32,
        w as i32,
        h as i32,
    ]
}

/// The size the wallpaper (16:9) covers in a render buffer: the buffer's width or height, whichever
/// makes the image reach the whole buffer.
fn cover_size((width, height): (u32, u32)) -> (u32, u32) {
    if u64::from(width) * 9 >= u64::from(height) * 16 {
        (width, (width * 9).div_ceil(16))
    } else {
        ((height * 16).div_ceil(9), height)
    }
}

unsafe fn set_filtering(gl: &glow::Context) {
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_S,
        glow::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_WRAP_T,
        glow::CLAMP_TO_EDGE as i32,
    );
}

unsafe fn compile(gl: &glow::Context, kind: u32, source: &str) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(kind)?;
    gl.shader_source(shader, source);
    gl.compile_shader(shader);
    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        let stage = if kind == glow::VERTEX_SHADER {
            "vertex"
        } else {
            "fragment"
        };
        Err(format!("{stage} shader failed to compile: {log}"))
    }
}

unsafe fn link(
    gl: &glow::Context,
    vertex: &str,
    fragment: &str,
    attributes: &[&str],
) -> Result<ShaderProgram, String> {
    let vertex = compile(gl, glow::VERTEX_SHADER, vertex)?;
    let fragment = compile(gl, glow::FRAGMENT_SHADER, fragment)?;
    let program = gl.create_program()?;
    gl.attach_shader(program, vertex);
    gl.attach_shader(program, fragment);
    for (index, name) in attributes.iter().enumerate() {
        gl.bind_attrib_location(program, index as u32, name);
    }
    gl.link_program(program);
    let linked = gl.get_program_link_status(program);
    // The shaders are no longer needed once linked.
    gl.detach_shader(program, vertex);
    gl.detach_shader(program, fragment);
    gl.delete_shader(vertex);
    gl.delete_shader(fragment);
    if !linked {
        let log = gl.get_program_info_log(program);
        gl.delete_program(program);
        return Err(format!("program failed to link: {log}"));
    }
    Ok(ShaderProgram {
        program,
        u_view: gl.get_uniform_location(program, "u_view"),
        u_surface: gl.get_uniform_location(program, "u_surface"),
        u_texture: gl.get_uniform_location(program, "u_texture"),
        u_bayer: gl.get_uniform_location(program, "u_bayer"),
        u_step: gl.get_uniform_location(program, "u_step"),
        u_dest: gl.get_uniform_location(program, "u_dest"),
    })
}

/// Reverses image rows top-to-bottom in-place, eliminating second full-screen buffer allocations.
pub(crate) fn flip_rows_in_place(buf: &mut [u8], stride: usize) {
    if stride == 0 || buf.len() <= stride {
        return;
    }
    let num_rows = buf.len() / stride;
    let (mut top, mut bottom) = (0, num_rows.saturating_sub(1));
    while top < bottom {
        let (head, tail) = buf.split_at_mut(bottom * stride);
        let top_row = &mut head[top * stride..(top + 1) * stride];
        let bottom_row = &mut tail[..stride];
        top_row.swap_with_slice(bottom_row);
        top += 1;
        bottom -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_rows_in_place_reverses_top_to_bottom_without_allocating() {
        let stride = 8;
        let mut buf = vec![
            1, 1, 1, 1, 1, 1, 1, 1, // row 0
            2, 2, 2, 2, 2, 2, 2, 2, // row 1
            3, 3, 3, 3, 3, 3, 3, 3, // row 2
        ];
        flip_rows_in_place(&mut buf, stride);
        assert_eq!(&buf[0..8], &[3, 3, 3, 3, 3, 3, 3, 3]);
        assert_eq!(&buf[8..16], &[2, 2, 2, 2, 2, 2, 2, 2]);
        assert_eq!(&buf[16..24], &[1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn the_wallpaper_cover_reaches_the_whole_buffer_in_any_proportion() {
        for (w, h) in [
            (1280, 720),
            (1920, 1080),
            (1366, 768),
            (1000, 720),
            (1280, 900),
            (720, 1280),
            (1, 1),
        ] {
            let (cw, ch) = cover_size((w, h));
            assert!(cw >= w && ch >= h, "{w}x{h}: cover {cw}x{ch}");
            // One of the two dimensions is exact; the other one is cropped.
            assert!(cw == w || ch == h);
            // The proportion stays 16:9, so the image is only cropped, never stretched.
            assert!(
                (cw as f32 / ch as f32 - 16.0 / 9.0).abs() < 0.01 * (16.0 / 9.0) + 1.0 / ch as f32
            );
        }
        assert_eq!(cover_size((1280, 720)), (1280, 720));
    }

    #[test]
    fn scissor_is_flipped_and_rounded_outwards() {
        let viewport = Viewport {
            width: 1920,
            height: 1080,
            scale: 1.5,
            offset_x: 0.0,
            offset_y: 0.0,
            edge_softness: 1.0,
        };
        // Logical (10.2, 20.2)-(110.2, 70.2) -> device (15.3, 30.3)-(165.3, 105.3).
        let [x, y, w, h] = device_scissor(&Rect::new(10.2, 20.2, 100.0, 50.0), &viewport);
        assert_eq!([x, w], [15, 151]);
        assert_eq!([y, h], [1080 - 106, 76]);
    }

    #[test]
    fn scissor_accounts_for_the_canvas_offset_and_stays_inside_the_surface() {
        let viewport = Viewport {
            width: 1000,
            height: 800,
            scale: 1.0,
            offset_x: 100.0,
            offset_y: 50.0,
            edge_softness: 1.0,
        };
        let [x, y, w, h] = device_scissor(&Rect::new(-500.0, -500.0, 5000.0, 5000.0), &viewport);
        assert_eq!([x, w, h], [0, 1000, 800]);
        assert_eq!(y, 0);
    }
}
