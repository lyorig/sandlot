//! Hardware-accelerated 2D rendering using textures, among other things.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryRender)):
//! - [ ] SDL_AddVulkanRenderSemaphores
//! - [ ] SDL_ConvertEventToRenderCoordinates
//! - [x] SDL_CreateGPURenderer
//! - [x] SDL_CreateRenderer
//! - [x] SDL_CreateRendererWithProperties
//! - [ ] SDL_CreateSoftwareRenderer
//! - [x] SDL_DestroyRenderer
//! - [x] SDL_FlushRenderer
//! - [x] SDL_GetCurrentRenderOutputSize
//! - [x] SDL_GetNumRenderDrivers
//! - [ ] SDL_GetRenderClipRect
//! - [ ] SDL_GetRenderColorScale
//! - [x] SDL_GetRenderDrawBlendMode
//! - [x] SDL_GetRenderDrawColor
//! - [x] SDL_GetRenderDrawColorFloat
//! - [ ] SDL_GetRenderDriver
//! - [x] SDL_GetRendererName
//! - [x] SDL_GetRendererProperties
//! - [ ] SDL_GetRenderLogicalPresentation
//! - [ ] SDL_GetRenderLogicalPresentationRect
//! - [ ] SDL_GetRenderMetalCommandEncoder
//! - [ ] SDL_GetRenderMetalLayer
//! - [x] SDL_GetRenderOutputSize
//! - [ ] SDL_GetRenderSafeArea
//! - [ ] SDL_GetRenderScale
//! - [x] SDL_GetRenderTarget
//! - [ ] SDL_GetRenderTextureAddressMode
//! - [ ] SDL_GetRenderViewport
//! - [x] SDL_GetRenderVSync
//! - [x] SDL_GetRenderWindow
//! - [x] SDL_RenderClear
//! - [ ] SDL_RenderClipEnabled
//! - [ ] SDL_RenderCoordinatesFromWindow
//! - [ ] SDL_RenderCoordinatesToWindow
//! - [ ] SDL_RenderDebugText
//! - [ ] SDL_RenderDebugTextFormat
//! - [x] SDL_RenderFillRect
//! - [x] SDL_RenderFillRects
//! - [ ] SDL_RenderGeometry
//! - [ ] SDL_RenderGeometryRaw
//! - [x] SDL_RenderLine
//! - [x] SDL_RenderLines
//! - [x] SDL_RenderPoint
//! - [x] SDL_RenderPoints
//! - [x] SDL_RenderPresent
//! - [x] SDL_RenderReadPixels
//! - [x] SDL_RenderRect
//! - [x] SDL_RenderRects
//! - [x] SDL_RenderTexture
//! - [x] SDL_RenderTexture9Grid
//! - [ ] SDL_RenderTexture9GridTiled
//! - [x] SDL_RenderTextureAffine
//! - [ ] SDL_RenderTextureRotated
//! - [x] SDL_RenderTextureTiled
//! - [ ] SDL_RenderViewportSet
//! - [ ] SDL_SetRenderClipRect
//! - [ ] SDL_SetRenderColorScale
//! - [x] SDL_SetRenderDrawBlendMode
//! - [x] SDL_SetRenderDrawColor
//! - [x] SDL_SetRenderDrawColorFloat
//! - [x] SDL_SetRenderGPUState
//! - [ ] SDL_SetRenderLogicalPresentation
//! - [ ] SDL_SetRenderScale
//! - [x] SDL_SetRenderTarget
//! - [ ] SDL_SetRenderTextureAddressMode
//! - [ ] SDL_SetRenderViewport
//! - [x] SDL_SetRenderVSync

use std::{ffi::CStr, mem::MaybeUninit};

use sdl3_sys::render::*;

use crate::{
    Result,
    color::{RgbaF32, RgbaU8},
    gpu::{Device, RenderState},
    pixels::BlendMode,
    properties::{Properties, PropertiesHandle},
    rect::{PointF32, PointI32, RectF32, RectI32},
    resource::Ref,
    resv2::resource_new,
    surface::Surface,
    texture::{Texture, TextureHandle},
    traits,
    util::mod_reexport,
    util::{opt2ptr, to_result},
    window::{Window, WindowHandle},
};

mod_reexport!(builder);
mod_reexport!(properties);

resource_new! {
    /// Represents rendering state.
    pub struct Renderer<> : SDL_Renderer, ~SDL_DestroyRenderer {
        marker: PhantomData<()>,
    }
}

impl RendererHandle {
    /// Get the name of a renderer.
    #[doc(alias = "SDL_GetRendererName")]
    pub fn name(&self) -> &str {
        unsafe {
            // SAFETY: Renderer name strings are all UTF-8.
            str::from_utf8_unchecked(
                CStr::from_ptr(SDL_GetRendererName(self.handle.as_ptr())).to_bytes(),
            )
        }
    }

    /// Get the properties associated with a renderer.
    ///
    /// Read-only properties of this renderer, as documented by
    /// [`SDL_GetRendererProperties`](https://wiki.libsdl.org/SDL3/SDL_GetRendererProperties).
    ///
    /// Covers the generic properties plus the D3D9, D3D11, D3D12, Vulkan and
    /// GPU-renderer backends. Not covered: the Metal backend
    /// (`SDL_PROP_RENDERER_METAL_*`), which sdl3-sys does not expose.
    #[doc(alias = "SDL_GetRendererProperties")]
    pub fn properties(&self) -> RendererProperties<'_> {
        unsafe {
            let id = SDL_GetRendererProperties(self.handle.as_ptr());
            let handle = PropertiesHandle::from_id(id).unwrap_unchecked();
            let r = Ref::from_handle(handle);

            RendererProperties::new(r)
        }
    }

    /// Get the window associated with a renderer.
    #[doc(alias = "SDL_GetRenderWindow")]
    pub fn window(&self) -> Ref<'_, Window> {
        unsafe {
            let ptr = SDL_GetRenderWindow(self.handle.as_ptr());
            let handle = WindowHandle::from_ptr(ptr).unwrap_unchecked();

            Ref::from_handle(handle)
        }
    }

    /// Get the current render target.
    ///
    /// Returns [`None`] for the default render target, which is the window
    /// for which the renderer was created.
    #[doc(alias = "SDL_GetRenderTarget")]
    pub fn target(&self) -> Option<Ref<'_, Texture>> {
        TextureHandle::from_ptr(unsafe { SDL_GetRenderTarget(self.handle.as_ptr()) })
            .map(|h| unsafe { Ref::from_handle(h) })
    }

    /// Get VSync of the given renderer.
    ///
    /// Returns the current vertical refresh sync interval. See
    /// [`RendererHandle::set_vsync`] for the meaning of the value.
    #[doc(alias = "SDL_GetRenderVSync")]
    pub fn vsync(&self) -> i32 {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            SDL_GetRenderVSync(self.handle.as_ptr(), ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    /// Get the output size in pixels of a rendering context.
    ///
    /// # Remarks
    ///
    /// This returns the true output size in pixels, ignoring any render
    /// targets or logical size and presentation.
    ///
    /// For the output size of the current rendering target, with logical size
    /// adjustments, use [`RendererHandle::target_output_size`] instead.
    #[doc(alias = "SDL_GetRenderOutputSize")]
    pub fn output_size(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetRenderOutputSize(self.handle.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the current output size in pixels of a rendering context.
    ///
    /// # Remarks
    ///
    /// If a rendering target is active, this will return the size of the
    /// rendering target in pixels, otherwise return the value of
    /// [`RendererHandle::output_size`].
    ///
    /// Rendering target or not, the output will be adjusted by the current
    /// logical presentation state, dictated by
    /// `SDL_SetRenderLogicalPresentation`.
    #[doc(alias = "SDL_GetCurrentRenderOutputSize")]
    pub fn target_output_size(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetCurrentRenderOutputSize(
                self.handle.as_ptr(),
                &raw mut (*ptr).x,
                &raw mut (*ptr).y,
            );
            ret.assume_init()
        }
    }

    /// Get the color used for drawing operations (Rect, Line and Clear),
    /// in 8-bit integer components.
    ///
    /// The alpha value is usually `255` (`SDL_ALPHA_OPAQUE`).
    #[doc(alias = "SDL_GetRenderDrawColor")]
    pub fn draw_color_u8(&self) -> RgbaU8 {
        let mut ret = MaybeUninit::<RgbaU8>::uninit();
        let ptr = ret.as_mut_ptr();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetRenderDrawColor(
                self.handle.as_ptr(),
                &raw mut (*ptr).rgb.r,
                &raw mut (*ptr).rgb.g,
                &raw mut (*ptr).rgb.b,
                &raw mut (*ptr).a,
            );

            ret.assume_init()
        }
    }

    /// Get the color used for drawing operations (Rect, Line and Clear),
    /// in floating-point components.
    #[doc(alias = "SDL_GetRenderDrawColorFloat")]
    pub fn draw_color_f32(&self) -> RgbaF32 {
        let mut ret = MaybeUninit::<RgbaF32>::uninit();
        let ptr = ret.as_mut_ptr();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetRenderDrawColorFloat(
                self.handle.as_ptr(),
                &raw mut (*ptr).rgb.r,
                &raw mut (*ptr).rgb.g,
                &raw mut (*ptr).rgb.b,
                &raw mut (*ptr).a,
            );

            ret.assume_init()
        }
    }

    /// Read pixels from the entire current rendering target.
    ///
    /// Returns a new surface containing pixels inside the desired area
    /// clipped to the current viewport.
    ///
    /// Note that this returns the actual pixels on the screen, so if you are
    /// using logical presentation you should use
    /// `SDL_GetRenderLogicalPresentationRect` to get the area containing your
    /// content.
    ///
    /// # Warning
    ///
    /// This is a very slow operation, and should not be used frequently. If
    /// you're using this on the main rendering target, it should be called
    /// after rendering and before [`RendererHandle::present`].
    #[doc(alias = "SDL_RenderReadPixels")]
    pub fn read_target(&self) -> Result<Surface> {
        Surface::from_ptr(unsafe { SDL_RenderReadPixels(self.handle.as_ptr(), std::ptr::null()) })
    }

    /// Read pixels from the current rendering target.
    ///
    /// `area` represents the area to read, which will be clipped to the
    /// current viewport. Returns a new surface containing pixels inside the
    /// desired area clipped to the current viewport.
    ///
    /// Note that this returns the actual pixels on the screen, so if you are
    /// using logical presentation you should use
    /// `SDL_GetRenderLogicalPresentationRect` to get the area containing your
    /// content.
    ///
    /// # Warning
    ///
    /// This is a very slow operation, and should not be used frequently. If
    /// you're using this on the main rendering target, it should be called
    /// after rendering and before [`RendererHandle::present`].
    #[doc(alias = "SDL_RenderReadPixels")]
    pub fn read_target_area(&self, area: RectI32) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            SDL_RenderReadPixels(self.handle.as_ptr(), (&raw const area).cast())
        })
    }

    /// Clear the current rendering target with the drawing color.
    ///
    /// # Remarks
    ///
    /// This function clears the entire rendering target, ignoring the
    /// viewport and the clip rectangle. Note, that clearing will also
    /// set/fill all pixels of the rendering target to current renderer draw
    /// color, so make sure to invoke [`RendererHandle::set_draw_color_u8`]
    /// (or its float variant) when needed.
    #[doc(alias = "SDL_RenderClear")]
    pub fn clear(&self) -> Result<()> {
        to_result(unsafe { SDL_RenderClear(self.handle.as_ptr()) })
    }

    /// Update the screen with any rendering performed since the previous call.
    ///
    /// # Remarks
    ///
    /// SDL's rendering functions operate on a backbuffer; that is, calling a
    /// rendering function such as [`RendererHandle::draw_line`] does not
    /// directly put a line on the screen, but rather updates the backbuffer.
    /// As such, you compose your entire scene and *present* the composed
    /// backbuffer to the screen as a complete picture.
    ///
    /// Therefore, when using SDL's rendering API, one does all drawing
    /// intended for the frame, and then calls this function once per frame to
    /// present the final drawing to the user.
    ///
    /// The backbuffer should be considered invalidated after each present; do
    /// not assume that previous contents will exist between frames. You are
    /// strongly encouraged to call [`RendererHandle::clear`] to initialize
    /// the backbuffer before starting each new frame's drawing, even if you
    /// plan to overwrite every pixel.
    ///
    /// Please note, that in case of rendering to a texture - there is **no
    /// need** to call this function after drawing needed objects to a
    /// texture, and should not be done; you are only required to change back
    /// the rendering target to default via [`RendererHandle::reset_target`]
    /// afterwards, as textures by themselves do not have a concept of
    /// backbuffers. Calling this function while rendering to a texture will
    /// fail.
    #[doc(alias = "SDL_RenderPresent")]
    pub fn present(&self) -> Result<()> {
        to_result(unsafe { SDL_RenderPresent(self.handle.as_ptr()) })
    }

    /// Force the rendering context to flush any pending commands and state.
    ///
    /// # Remarks
    ///
    /// You do not need to (and in fact, shouldn't) call this function unless
    /// you are planning to call into OpenGL/Direct3D/Metal/whatever directly,
    /// in addition to using an SDL_Renderer.
    ///
    /// This is for a very-specific case: if you are using SDL's render API,
    /// and you plan to make OpenGL/D3D/whatever calls in addition to SDL
    /// render API calls. If this applies, you should call this function
    /// between calls to SDL's render API and the low-level API you're using
    /// in cooperation. In all other cases, you can ignore this function.
    ///
    /// This call makes SDL flush any pending rendering work it was queueing
    /// up to do later in a single batch, and marks any internal cached state
    /// as invalid, so it'll prepare all its state again later, from scratch.
    ///
    /// This means you do not need to save state in your rendering code to
    /// protect the SDL renderer. However, there are lots of arbitrary pieces
    /// of Direct3D and OpenGL state that can confuse things; you should use
    /// your best judgment and be prepared to make changes if specific state
    /// needs to be protected.
    #[doc(alias = "SDL_FlushRenderer")]
    pub fn flush(&self) -> Result<()> {
        to_result(unsafe { SDL_FlushRenderer(self.handle.as_ptr()) })
    }

    /// Copy a portion of the texture to the current rendering target at
    /// subpixel precision.
    ///
    /// `src` selects the source rectangle, or the entire texture if [`None`].
    /// `dst` selects the destination rectangle, or the entire rendering
    /// target if [`None`].
    ///
    /// This function is a direct wrapper of SDL's `SDL_RenderTexture`;
    /// see [`DrawBuilder`] for a neater way to draw to a renderer.
    #[doc(alias = "SDL_RenderTexture")]
    pub fn draw(
        &self,
        tex: Ref<Texture>,
        src: Option<&RectF32>,
        dst: Option<&RectF32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_RenderTexture(
                self.handle.as_ptr(),
                tex.handle.as_ptr(),
                opt2ptr(src).cast(),
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Copy a portion of the source texture to the current rendering target,
    /// with affine transform, at subpixel precision.
    ///
    /// `src` selects the source rectangle, or the entire texture if [`None`].
    ///
    /// `origin` indicates where the top-left corner of `src` should be mapped
    /// to, or the rendering target's origin if [`None`].
    ///
    /// `right` indicates where the top-right corner of `src` should be mapped
    /// to, or the rendering target's top-right corner if [`None`].
    ///
    /// `down` indicates where the bottom-left corner of `src` should be
    /// mapped to, or the rendering target's bottom-left corner if [`None`].
    #[doc(alias = "SDL_RenderTextureAffine")]
    pub fn draw_affine(
        &self,
        tex: Ref<Texture>,
        src: Option<&RectF32>,
        origin: Option<&PointF32>,
        right: Option<&PointF32>,
        down: Option<&PointF32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_RenderTextureAffine(
                self.handle.as_ptr(),
                tex.handle.as_ptr(),
                opt2ptr(src).cast(),
                opt2ptr(origin).cast(),
                opt2ptr(right).cast(),
                opt2ptr(down).cast(),
            )
        })
    }

    /// Tile a portion of the texture to the current rendering target at
    /// subpixel precision.
    ///
    /// `src` selects the source rectangle, or the entire texture if [`None`].
    /// `dst` selects the destination rectangle, or the entire rendering
    /// target if [`None`].
    ///
    /// # Remarks
    ///
    /// The pixels in `src` will be repeated as many times as needed to
    /// completely fill `dst`.
    ///
    /// `scale` is the scale used to transform `src` into the destination
    /// rectangle, e.g. a 32x32 texture with a scale of 2 would fill 64x64
    /// tiles.
    #[doc(alias = "SDL_RenderTextureTiled")]
    pub fn draw_tiled(
        &self,
        tex: Ref<Texture>,
        src: Option<&RectF32>,
        scale: f32,
        dst: Option<&RectF32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_RenderTextureTiled(
                self.handle.as_ptr(),
                tex.handle.as_ptr(),
                opt2ptr(src).cast(),
                scale,
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Perform a scaled copy using the 9-grid algorithm to the current
    /// rendering target at subpixel precision.
    ///
    /// `src` selects the rectangle to be used for the 9-grid, or the entire
    /// texture if [`None`]. `dst` selects the destination rectangle, or the
    /// entire rendering target if [`None`].
    ///
    /// The tuple elements are, in order: the width, in pixels, of the left
    /// corners in `src`; the width of the right corners; the height of the
    /// top corners; the height of the bottom corners.
    ///
    /// `scale` is the scale used to transform the corners of `src` into the
    /// corners of `dst`, or `0.0` for an unscaled copy.
    ///
    /// # Remarks
    ///
    /// The pixels in the texture are split into a 3x3 grid, using the
    /// different corner sizes for each corner, and the sides and center
    /// making up the remaining pixels. The corners are then scaled using
    /// `scale` and fit into the corners of the destination rectangle. The
    /// sides and center are then stretched into place to cover the remaining
    /// destination rectangle.
    #[doc(alias = "SDL_RenderTexture9Grid")]
    pub fn draw_9grid(
        &self,
        tex: Ref<Texture>,
        src: Option<&RectF32>,
        (width_left, width_right, width_top, width_bottom): (f32, f32, f32, f32),
        scale: f32,
        dst: Option<&RectF32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_RenderTexture9Grid(
                self.handle.as_ptr(),
                tex.handle.as_ptr(),
                opt2ptr(src).cast(),
                width_left,
                width_right,
                width_top,
                width_bottom,
                scale,
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Draw a line on the current rendering target at subpixel precision.
    ///
    /// The arguments are the coordinates of the start and end points.
    #[doc(alias = "SDL_RenderLine")]
    pub fn draw_line(&self, start: PointF32, end: PointF32) -> Result<()> {
        to_result(unsafe { SDL_RenderLine(self.handle.as_ptr(), start.x, start.y, end.x, end.y) })
    }

    /// Draw a line on the current rendering target at subpixel precision,
    /// temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderLine")]
    pub fn draw_line_with(&self, start: PointF32, end: PointF32, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_line(start, end);
        self.set_draw_color_f32(old);

        ret
    }

    /// Draw a series of connected lines on the current rendering target at
    /// subpixel precision.
    ///
    /// `lines` contains the points along the lines; `lines.len() - 1` lines
    /// are drawn.
    #[doc(alias = "SDL_RenderLines")]
    pub fn draw_lines(&self, lines: &[PointF32]) -> Result<()> {
        to_result(unsafe {
            SDL_RenderLines(
                self.handle.as_ptr(),
                lines.as_ptr().cast(),
                lines.len() as i32,
            )
        })
    }

    /// Draw a series of connected lines on the current rendering target at
    /// subpixel precision, temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderLines")]
    pub fn draw_lines_with(&self, lines: &[PointF32], col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_lines(lines);
        self.set_draw_color_f32(old);

        ret
    }

    /// Draw a point on the current rendering target at subpixel precision.
    #[doc(alias = "SDL_RenderPoint")]
    pub fn draw_point(&self, pos: PointF32) -> Result<()> {
        to_result(unsafe { SDL_RenderPoint(self.handle.as_ptr(), pos.x, pos.y) })
    }

    /// Draw a point on the current rendering target at subpixel precision,
    /// temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderPoint")]
    pub fn draw_point_with(&self, pos: PointF32, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_point(pos);
        self.set_draw_color_f32(old);

        ret
    }

    /// Draw multiple points on the current rendering target at subpixel
    /// precision.
    #[doc(alias = "SDL_RenderPoints")]
    pub fn draw_points(&self, points: &[PointF32]) -> Result<()> {
        to_result(unsafe {
            SDL_RenderPoints(
                self.handle.as_ptr(),
                points.as_ptr().cast(),
                points.len() as i32,
            )
        })
    }

    /// Draw multiple points on the current rendering target at subpixel
    /// precision, temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderPoints")]
    pub fn draw_points_with(&self, points: &[PointF32], col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_points(points);
        self.set_draw_color_f32(old);

        ret
    }

    /// Draw a rectangle on the current rendering target at subpixel
    /// precision.
    #[doc(alias = "SDL_RenderRect")]
    pub fn draw_rect(&self, rect: RectF32) -> Result<()> {
        to_result(unsafe { SDL_RenderRect(self.handle.as_ptr(), (&raw const rect).cast()) })
    }

    /// Draw a rectangle on the current rendering target at subpixel
    /// precision, temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderRect")]
    pub fn draw_rect_with(&self, rect: RectF32, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let res = self.draw_rect(rect);
        self.set_draw_color_f32(old);

        res
    }

    /// Draw a rectangle outlining the entire rendering target, at subpixel
    /// precision.
    ///
    /// Equivalent to SDL's `SDL_RenderRect` with a `NULL` rectangle.
    #[doc(alias = "SDL_RenderRect")]
    pub fn draw_target_outline(&self) -> Result<()> {
        to_result(unsafe { SDL_RenderRect(self.handle.as_ptr(), std::ptr::null()) })
    }

    /// Draw a rectangle outlining the entire rendering target, at subpixel
    /// precision, temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderRect")]
    pub fn draw_target_outline_with(&self, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_target_outline();
        self.set_draw_color_f32(old);

        ret
    }

    /// Draw some number of rectangles on the current rendering target at
    /// subpixel precision.
    #[doc(alias = "SDL_RenderRects")]
    pub fn draw_rects(&self, rects: &[RectF32]) -> Result<()> {
        to_result(unsafe {
            SDL_RenderRects(
                self.handle.as_ptr(),
                rects.as_ptr().cast(),
                rects.len() as i32,
            )
        })
    }

    /// Draw some number of rectangles on the current rendering target at
    /// subpixel precision, temporarily using `col` as the drawing color.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderRects")]
    pub fn draw_rects_with(&self, rects: &[RectF32], col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.draw_rects(rects);
        self.set_draw_color_f32(old);

        ret
    }

    /// Fill the entire rendering target with the drawing color at subpixel
    /// precision.
    ///
    /// Equivalent to SDL's `SDL_RenderFillRect` with a `NULL` rectangle.
    #[doc(alias = "SDL_RenderFillRect")]
    pub fn fill_target(&self) -> Result<()> {
        to_result(unsafe { SDL_RenderFillRect(self.handle.as_ptr(), std::ptr::null()) })
    }

    /// Fill the entire rendering target with `col` at subpixel precision.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderFillRect")]
    pub fn fill_target_with(&self, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.fill_target();
        self.set_draw_color_f32(old);

        ret
    }

    /// Fill a rectangle on the current rendering target with the drawing
    /// color at subpixel precision.
    #[doc(alias = "SDL_RenderFillRect")]
    pub fn fill_rect(&self, rect: RectF32) -> Result<()> {
        to_result(unsafe { SDL_RenderFillRect(self.handle.as_ptr(), (&raw const rect).cast()) })
    }

    /// Fill a rectangle on the current rendering target with `col` at
    /// subpixel precision.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderFillRect")]
    pub fn fill_rect_with(&self, rect: RectF32, col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let res = self.fill_rect(rect);
        self.set_draw_color_f32(old);

        res
    }

    /// Fill some number of rectangles on the current rendering target with
    /// the drawing color at subpixel precision.
    #[doc(alias = "SDL_RenderFillRects")]
    pub fn fill_rects(&self, rects: &[RectF32]) -> Result<()> {
        to_result(unsafe {
            SDL_RenderFillRects(
                self.handle.as_ptr(),
                rects.as_ptr().cast(),
                rects.len() as i32,
            )
        })
    }

    /// Fill some number of rectangles on the current rendering target with
    /// `col` at subpixel precision.
    ///
    /// The previous drawing color is restored afterwards.
    #[doc(alias = "SDL_RenderFillRects")]
    pub fn fill_rects_with(&self, rects: &[RectF32], col: RgbaF32) -> Result<()> {
        let old = self.xchg_draw_color_f32(col);
        let ret = self.fill_rects(rects);
        self.set_draw_color_f32(old);

        ret
    }

    /// Set a texture as the current rendering target.
    ///
    /// The targeted texture must be created with the
    /// `SDL_TEXTUREACCESS_TARGET` flag; [`None`] renders to the window
    /// instead of a texture.
    ///
    /// For use with [`RendererHandle::xchg_target`]. Otherwise, prefer using
    /// [`RendererHandle::set_target`] or [`RendererHandle::reset_target`].
    ///
    /// # Remarks
    ///
    /// The default render target is the window for which the renderer was
    /// created. To stop rendering to a texture and render to the window
    /// again, call this function with [`None`].
    ///
    /// Viewport, cliprect, scale, and logical presentation are unique to each
    /// render target. Get and set functions for these states apply to the
    /// current render target set by this function, and those states persist
    /// on each target when the current render target changes.
    ///
    /// # Safety
    /// If the parameter is `Some(tex)`, ensure `tex` lives for as long as it's
    /// used as the target texture.
    #[doc(alias = "SDL_SetRenderTarget")]
    pub fn set_target_opt(&self, tgt: Option<Ref<Texture>>) -> Result<()> {
        to_result(unsafe {
            SDL_SetRenderTarget(
                self.handle.as_ptr(),
                match tgt {
                    Some(h) => h.handle.as_ptr(),
                    None => std::ptr::null_mut(),
                },
            )
        })
    }

    /// Set a texture as the current rendering target.
    ///
    /// The targeted texture must be created with the `SDL_TEXTUREACCESS_TARGET`
    /// flag. See [`RendererHandle::set_target_opt`] for more details.
    #[doc(alias = "SDL_SetRenderTarget")]
    pub fn set_target(&self, tgt: Ref<Texture>) -> Result<()> {
        self.set_target_opt(Some(tgt))
    }

    /// Stop rendering to a texture and render to the window again.
    ///
    /// See [`RendererHandle::set_target_opt`] for more details.
    #[doc(alias = "SDL_SetRenderTarget")]
    pub fn reset_target(&self) -> Result<()> {
        self.set_target_opt(None)
    }

    pub fn xchg_target(&self, tgt: Ref<Texture>) -> Result<Option<Ref<'_, Texture>>> {
        let old = self.target();
        self.set_target(tgt)?;
        Ok(old)
    }

    /// Toggle VSync of the given renderer.
    ///
    /// `val` can be `1` to synchronize present with every vertical refresh,
    /// `2` to synchronize present with every second vertical refresh, etc.,
    /// [`Renderer::VSYNC_ADAPTIVE`] for late swap tearing (adaptive vsync),
    /// or [`Renderer::VSYNC_DISABLED`] to disable.
    ///
    /// Not every value is supported by every driver, so you should check
    /// the return value to see whether the requested setting is supported.
    ///
    /// # Remarks
    ///
    /// When a renderer is created, vsync defaults to
    /// [`Renderer::VSYNC_DISABLED`].
    #[doc(alias = "SDL_SetRenderVSync")]
    pub fn set_vsync(&self, val: i32) -> bool {
        unsafe { SDL_SetRenderVSync(self.handle.as_ptr(), val) }
    }

    /// Set the color used for drawing operations, in 8-bit integer
    /// components.
    ///
    /// This sets the color for drawing or filling rectangles, lines, and
    /// points, and for [`RendererHandle::clear`].
    ///
    /// The alpha value is usually `255` (`SDL_ALPHA_OPAQUE`). Use
    /// [`BlendMode::set_blend_mode`](crate::traits::BlendMode::set_blend_mode)
    /// to specify how the alpha channel is used.
    #[doc(alias = "SDL_SetRenderDrawColor")]
    pub fn set_draw_color_u8(&self, rgba: RgbaU8) {
        unsafe {
            SDL_SetRenderDrawColor(
                self.handle.as_ptr(),
                rgba.rgb.r,
                rgba.rgb.g,
                rgba.rgb.b,
                rgba.a,
            );
        }
    }

    /// Set the color used for drawing operations, in floating-point
    /// components.
    ///
    /// This sets the color for drawing or filling rectangles, lines, and
    /// points, and for [`RendererHandle::clear`].
    ///
    /// Use
    /// [`BlendMode::set_blend_mode`](crate::traits::BlendMode::set_blend_mode)
    /// to specify how the alpha channel is used.
    #[doc(alias = "SDL_SetRenderDrawColorFloat")]
    pub fn set_draw_color_f32(&self, rgba: RgbaF32) {
        unsafe {
            SDL_SetRenderDrawColorFloat(
                self.handle.as_ptr(),
                rgba.rgb.r,
                rgba.rgb.g,
                rgba.rgb.b,
                rgba.a,
            );
        }
    }

    pub fn xchg_draw_color_u8(&self, col: RgbaU8) -> RgbaU8 {
        let old = self.draw_color_u8();
        self.set_draw_color_u8(col);
        old
    }

    pub fn xchg_draw_color_f32(&self, col: RgbaF32) -> RgbaF32 {
        let old = self.draw_color_f32();
        self.set_draw_color_f32(col);
        old
    }

    pub fn set_render_state(&self, rs: Ref<RenderState>) -> Result<()> {
        to_result(unsafe { SDL_SetGPURenderState(self.as_ptr(), rs.as_ptr()) })
    }

    /// Clear custom GPU render state, reverting to the default rendering
    /// behavior.
    #[doc(alias = "SDL_SetGPURenderState")]
    pub fn clear_render_state(&self) -> Result<()> {
        to_result(unsafe { SDL_SetGPURenderState(self.as_ptr(), std::ptr::null_mut()) })
    }
}

impl traits::BlendMode for RendererHandle {
    /// Get the blend mode used for drawing operations.
    #[doc(alias = "SDL_GetRenderDrawBlendMode")]
    fn blend_mode(&self) -> BlendMode {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            SDL_GetRenderDrawBlendMode(self.handle.as_ptr(), ret.as_mut_ptr());
            BlendMode::from_sdl_unchecked(ret.assume_init())
        }
    }

    /// Set the blend mode used for drawing operations.
    ///
    /// # Remarks
    ///
    /// This blend mode is used for any drawing that doesn't involve
    /// textures.
    ///
    /// If the blend mode is not supported, the closest supported mode is
    /// chosen.
    #[doc(alias = "SDL_SetRenderDrawBlendMode")]
    fn set_blend_mode(&self, bm: BlendMode) {
        unsafe {
            SDL_SetRenderDrawBlendMode(self.handle.as_ptr(), bm.to_sdl());
        }
    }
}

impl Renderer {
    /// Disable vsync. See [`RendererHandle::set_vsync`].
    pub const VSYNC_DISABLED: i32 = SDL_RENDERER_VSYNC_DISABLED;

    /// Adaptive vsync (late swap tearing). See [`RendererHandle::set_vsync`].
    pub const VSYNC_ADAPTIVE: i32 = SDL_RENDERER_VSYNC_ADAPTIVE;

    /// Bind the builder to an existing property group.
    ///
    /// The renderer creation properties (`SDL_PROP_RENDERER_CREATE_*`)
    /// never collide with the window or GPU device ones, so a single
    /// [`Properties`] can be shared between the three builders.
    pub fn builder(props: Ref<Properties>) -> RendererBuilder {
        RendererBuilder::new(props)
    }

    /// Create a 2D rendering context for a window.
    ///
    /// `name` is the name of the rendering driver to initialize, or [`None`]
    /// to let SDL choose one.
    ///
    /// # Remarks
    ///
    /// If you want a specific renderer, you can specify its name here. A list
    /// of available renderers can be obtained by calling
    /// `SDL_GetRenderDriver` multiple times, with indices from 0 to
    /// [`Renderer::num_drivers`]`-1`. If you don't need a specific renderer,
    /// specify [`None`] and SDL will attempt to choose the best option for
    /// you, based on what is available on the user's system.
    ///
    /// If `name` is a comma-separated list, SDL will try each name, in the
    /// order listed, until one succeeds or all of them fail.
    ///
    /// By default the rendering size matches the window size in pixels, but
    /// you can call `SDL_SetRenderLogicalPresentation` to change the content
    /// size and scaling options.
    #[doc(alias = "SDL_CreateRenderer")]
    pub fn new(wnd: Ref<Window>, name: Option<&CStr>) -> Result<Renderer> {
        Self::from_ptr(unsafe {
            SDL_CreateRenderer(
                wnd.handle.as_ptr(),
                name.map_or(std::ptr::null(), CStr::as_ptr),
            )
        })
    }

    /// Create a 2D GPU rendering context.
    ///
    /// # Remarks
    ///
    /// The GPU device to use is passed in as a parameter.
    ///
    /// The window to use is passed in as a parameter. If this were [`None`],
    /// the renderer would become an offscreen renderer; in that case, you
    /// should call [`RendererHandle::set_target`] to setup rendering to a
    /// texture, and then call [`RendererHandle::present`] normally to
    /// complete drawing a frame.
    #[doc(alias = "SDL_CreateGPURenderer")]
    pub fn new_gpu(device: Ref<Device>, wnd: Ref<Window>) -> Result<Self> {
        Self::from_ptr(unsafe { SDL_CreateGPURenderer(device.as_ptr(), wnd.as_ptr()) })
    }

    /// Get the number of 2D rendering drivers available for the current
    /// display.
    ///
    /// # Remarks
    ///
    /// A render driver is a set of code that handles rendering and texture
    /// management on a particular display. Normally there is only one, but
    /// some drivers may have several available with different capabilities.
    ///
    /// There may be none if SDL was compiled without render support.
    #[doc(alias = "SDL_GetNumRenderDrivers")]
    pub fn num_drivers() -> i32 {
        unsafe { SDL_GetNumRenderDrivers() }
    }
}

/// A builder-like struct intended an an alternative
/// to `RendererHandle::draw()`.
pub struct DrawBuilder<'rnd, 'tex, 'rct> {
    renderer: Ref<'rnd, Renderer>,

    texture: Ref<'tex, Texture>,
    src: Option<&'rct RectF32>,
    dst: Option<&'rct RectF32>,
}

impl<'rnd, 'tex, 'rct> DrawBuilder<'rnd, 'tex, 'rct> {
    pub fn new(rnd: Ref<'rnd, Renderer>, tex: Ref<'tex, Texture>) -> Self {
        Self {
            renderer: rnd,
            texture: tex,
            src: None,
            dst: None,
        }
    }

    pub fn from(&mut self, src: &'rct RectF32) -> &mut Self {
        self.src = Some(src);
        self
    }

    pub fn to(&mut self, dst: &'rct RectF32) -> &mut Self {
        self.dst = Some(dst);
        self
    }

    pub fn draw(&self) -> Result<()> {
        self.renderer.draw(self.texture, self.src, self.dst)
    }
}
