//! A 2D image stored in system RAM and directly accessible via the CPU.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategorySurface)):
//! - [ ] SDL_AddSurfaceAlternateImage
//! - [x] SDL_BlitSurface
//! - [x] SDL_BlitSurface9Grid
//! - [x] SDL_BlitSurfaceScaled
//! - [x] SDL_BlitSurfaceTiled
//! - [x] SDL_BlitSurfaceTiledWithScale
//! - [ ] SDL_BlitSurfaceUnchecked
//! - [ ] SDL_BlitSurfaceUncheckedScaled
//! - [x] SDL_ClearSurface
//! - [ ] SDL_ConvertPixels
//! - [ ] SDL_ConvertPixelsAndColorspace
//! - [ ] SDL_ConvertSurface
//! - [ ] SDL_ConvertSurfaceAndColorspace
//! - [x] SDL_CreateSurface
//! - [ ] SDL_CreateSurfaceFrom
//! - [ ] SDL_CreateSurfacePalette
//! - [x] SDL_DestroySurface
//! - [x] SDL_DuplicateSurface
//! - [x] SDL_FillSurfaceRect
//! - [x] SDL_FillSurfaceRects
//! - [x] SDL_FlipSurface
//! - [x] SDL_GetSurfaceAlphaMod
//! - [x] SDL_GetSurfaceBlendMode
//! - [ ] SDL_GetSurfaceClipRect
//! - [ ] SDL_GetSurfaceColorKey
//! - [x] SDL_GetSurfaceColorMod
//! - [ ] SDL_GetSurfaceColorspace
//! - [ ] SDL_GetSurfaceImages
//! - [ ] SDL_GetSurfacePalette
//! - [ ] SDL_GetSurfaceProperties
//! - [ ] SDL_LoadBMP
//! - [ ] SDL_LoadBMP_IO
//! - [ ] SDL_LockSurface
//! - [x] SDL_MapSurfaceRGB
//! - [x] SDL_MapSurfaceRGBA
//! - [ ] SDL_PremultiplyAlpha
//! - [ ] SDL_PremultiplySurfaceAlpha
//! - [ ] SDL_ReadSurfacePixel
//! - [ ] SDL_ReadSurfacePixelFloat
//! - [ ] SDL_RemoveSurfaceAlternateImages
//! - [ ] SDL_SaveBMP
//! - [ ] SDL_SaveBMP_IO
//! - [x] SDL_ScaleSurface
//! - [x] SDL_SetSurfaceAlphaMod
//! - [x] SDL_SetSurfaceBlendMode
//! - [ ] SDL_SetSurfaceClipRect
//! - [ ] SDL_SetSurfaceColorKey
//! - [x] SDL_SetSurfaceColorMod
//! - [ ] SDL_SetSurfaceColorspace
//! - [ ] SDL_SetSurfacePalette
//! - [ ] SDL_SetSurfaceRLE
//! - [x] SDL_StretchSurface
//! - [ ] SDL_SurfaceHasAlternateImages
//! - [ ] SDL_SurfaceHasColorKey
//! - [ ] SDL_SurfaceHasRLE
//! - [ ] SDL_UnlockSurface
//! - [ ] SDL_WriteSurfacePixel
//! - [ ] SDL_WriteSurfacePixelFloat

use std::mem::MaybeUninit;

use crate::{
    Result,
    color::{RgbU8, RgbaF32, RgbaU8},
    pixels::PixelFormat,
    pixels::{BlendMode, ScaleMode},
    rect::{PointI32, RectI32},
    resource::Ref,
    resource_new, traits,
    util::{opt2ptr, to_result},
};

use sdl3_sys::surface::*;

resource_new!(SDL_Surface, Surface, SDL_DestroySurface);

impl SurfaceHandle {
    /// Get the size of the surface.
    pub fn size(&self) -> PointI32 {
        let surf = unsafe { self.handle.as_ref() };
        PointI32::new(surf.w, surf.h)
    }

    /// Get the pixel format of the surface.
    pub fn format(&self) -> PixelFormat {
        let surf = unsafe { self.handle.as_ref() };
        surf.format.into()
    }

    /// Perform a fast fill of the entire surface with a specific color.
    ///
    /// Equivalent to SDL's `SDL_FillSurfaceRect` with a `NULL` rectangle.
    ///
    /// # Remarks
    ///
    /// If the color value contains an alpha component then the destination
    /// is simply filled with that alpha information, no blending takes
    /// place.
    ///
    /// If there is a clip rectangle set on the destination (set via
    /// `SDL_SetSurfaceClipRect`), then this function will fill based on the
    /// intersection of the clip rectangle and the whole surface.
    #[doc(alias = "SDL_FillSurfaceRect")]
    pub fn fill(&self, c: RgbaU8) -> Result<()> {
        to_result(unsafe {
            SDL_FillSurfaceRect(self.handle.as_ptr(), std::ptr::null(), self.map_rgba(c))
        })
    }

    /// Perform a fast fill of a rectangle with a specific color.
    ///
    /// # Remarks
    ///
    /// If the color value contains an alpha component then the destination
    /// is simply filled with that alpha information, no blending takes
    /// place.
    ///
    /// If there is a clip rectangle set on the destination (set via
    /// `SDL_SetSurfaceClipRect`), then this function will fill based on the
    /// intersection of the clip rectangle and `pos`.
    #[doc(alias = "SDL_FillSurfaceRect")]
    pub fn fill_rect(&self, pos: RectI32, c: RgbaU8) -> Result<()> {
        to_result(unsafe {
            SDL_FillSurfaceRect(
                self.handle.as_ptr(),
                (&raw const pos).cast(),
                self.map_rgba(c),
            )
        })
    }

    /// Perform a fast fill of a set of rectangles with a specific color.
    ///
    /// # Remarks
    ///
    /// If the color value contains an alpha component then the destination
    /// is simply filled with that alpha information, no blending takes
    /// place.
    ///
    /// If there is a clip rectangle set on the destination (set via
    /// `SDL_SetSurfaceClipRect`), then this function will fill based on the
    /// intersection of the clip rectangle and each rectangle in `pos`.
    #[doc(alias = "SDL_FillSurfaceRects")]
    pub fn fill_rects(&self, pos: &[RectI32], c: RgbaU8) -> Result<()> {
        to_result(unsafe {
            SDL_FillSurfaceRects(
                self.handle.as_ptr(),
                (&raw const pos).cast(),
                pos.len() as i32,
                self.map_rgba(c),
            )
        })
    }

    /// Clear a surface with a specific color, with floating point precision.
    ///
    /// The color components are normally in the range 0-1.
    ///
    /// # Remarks
    ///
    /// This function handles all surface formats, and ignores any clip
    /// rectangle.
    ///
    /// If the surface is YUV, the color is assumed to be in the sRGB
    /// colorspace, otherwise the color is assumed to be in the colorspace of
    /// the surface.
    #[doc(alias = "SDL_ClearSurface")]
    pub fn clear(&self, c: RgbaF32) -> Result<()> {
        to_result(unsafe { SDL_ClearSurface(self.handle.as_ptr(), c.rgb.r, c.rgb.g, c.rgb.b, c.a) })
    }

    /// Flip a surface vertically or horizontally.
    #[doc(alias = "SDL_FlipSurface")]
    pub fn flip(&self, fm: SDL_FlipMode) -> Result<()> {
        to_result(unsafe { SDL_FlipSurface(self.handle.as_ptr(), fm) })
    }

    /// Create a new surface identical to the existing surface, scaled to
    /// the desired size.
    #[doc(alias = "SDL_ScaleSurface")]
    pub fn scale(&self, size: PointI32, sm: ScaleMode) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            SDL_ScaleSurface(self.handle.as_ptr(), size.x, size.y, sm.into())
        })
    }

    /// Map an RGB triple to an opaque pixel value for a surface.
    ///
    /// The components are in the range 0-255.
    ///
    /// # Remarks
    ///
    /// This function maps the RGB color value to the specified pixel format
    /// and returns the pixel value best approximating the given RGB color
    /// value for the given pixel format.
    ///
    /// If the surface has a palette, the index of the closest matching color
    /// in the palette will be returned.
    ///
    /// If the surface pixel format has an alpha component it will be
    /// returned as all 1 bits (fully opaque).
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the
    /// unused upper bits of the return value can safely be ignored.
    #[doc(alias = "SDL_MapSurfaceRGB")]
    pub fn map_rgb(&self, rgb: RgbU8) -> u32 {
        unsafe { SDL_MapSurfaceRGB(self.handle.as_ptr(), rgb.r, rgb.g, rgb.b) }
    }

    /// Map an RGBA quadruple to a pixel value for a surface.
    ///
    /// The components are in the range 0-255.
    ///
    /// # Remarks
    ///
    /// This function maps the RGBA color value to the specified pixel format
    /// and returns the pixel value best approximating the given RGBA color
    /// value for the given pixel format.
    ///
    /// If the surface pixel format has no alpha component the alpha value
    /// will be ignored (as it will be in formats with a palette).
    ///
    /// If the surface has a palette, the index of the closest matching color
    /// in the palette will be returned.
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the
    /// unused upper bits of the return value can safely be ignored.
    #[doc(alias = "SDL_MapSurfaceRGBA")]
    pub fn map_rgba(&self, rgba: RgbaU8) -> u32 {
        unsafe {
            SDL_MapSurfaceRGBA(
                self.handle.as_ptr(),
                rgba.rgb.r,
                rgba.rgb.g,
                rgba.rgb.b,
                rgba.a,
            )
        }
    }

    /// Create a new surface identical to the existing surface.
    ///
    /// # Remarks
    ///
    /// If the original surface has alternate images, the new surface will
    /// have a reference to them as well.
    #[doc(alias = "SDL_DuplicateSurface")]
    pub fn try_clone(&self) -> Result<Surface> {
        Surface::from_ptr(unsafe { SDL_DuplicateSurface(self.handle.as_ptr()) })
    }

    /// Perform a stretched pixel copy from this surface to another.
    ///
    /// `src` selects the source rectangle, or the entire surface if
    /// [`None`]. `dst` selects the target rectangle in the destination
    /// surface, or the entire destination surface if [`None`].
    #[doc(alias = "SDL_StretchSurface")]
    pub fn stretch(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
        scale_mode: ScaleMode,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_StretchSurface(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
                scale_mode.into(),
            )
        })
    }

    /// Perform a fast blit from this surface to the destination surface with
    /// clipping.
    ///
    /// `src` selects the source rectangle, or the entire surface if
    /// [`None`]. `dst` selects the x and y position in the destination
    /// surface, or (0,0) if [`None`].
    ///
    /// # Remarks
    ///
    /// The blit function should not be called on a locked surface.
    ///
    /// The blit semantics for surfaces with and without blending and
    /// colorkey are as follows:
    ///
    /// - `RGBA->RGB`: with [`BlendMode::Blend`], alpha-blend (using the
    ///   source alpha-channel and per-surface alpha); with
    ///   [`BlendMode::None`], copy RGB (if a color key is set, only copy the
    ///   pixels that do not match its RGB values, ignoring alpha in the
    ///   comparison).
    /// - `RGB->RGBA`: with [`BlendMode::Blend`], alpha-blend (using the
    ///   source per-surface alpha); with [`BlendMode::None`], copy RGB and
    ///   set destination alpha to the source per-surface alpha value (if a
    ///   color key is set, only copy the pixels that do not match it).
    /// - `RGBA->RGBA`: with [`BlendMode::Blend`], alpha-blend (using the
    ///   source alpha-channel and per-surface alpha); with
    ///   [`BlendMode::None`], copy all of RGBA to the destination (if a
    ///   color key is set, only copy the pixels that do not match its RGB
    ///   values, ignoring alpha in the comparison).
    /// - `RGB->RGB`: with [`BlendMode::Blend`], alpha-blend (using the
    ///   source per-surface alpha); with [`BlendMode::None`], copy RGB (if a
    ///   color key is set, only copy the pixels that do not match it).
    #[doc(alias = "SDL_BlitSurface")]
    pub fn blit(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_BlitSurface(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Perform a scaled blit using the 9-grid algorithm to a destination
    /// surface, which may be of a different format.
    ///
    /// `src` selects the rectangle to be used for the 9-grid, or the entire
    /// surface if [`None`]. `dst` selects the target rectangle in the
    /// destination surface, or the entire destination surface if [`None`].
    ///
    /// The tuple elements are, in order: the width, in pixels, of the left
    /// corners in `src`; the width of the right corners; the height of the
    /// top corners; the height of the bottom corners.
    ///
    /// `scale` is the scale used to transform the corners of `src` into the
    /// corners of `dst`, or `0.0` for an unscaled blit.
    ///
    /// # Remarks
    ///
    /// The pixels in the source surface are split into a 3x3 grid, using the
    /// different corner sizes for each corner, and the sides and center
    /// making up the remaining pixels. The corners are then scaled using
    /// `scale` and fit into the corners of the destination rectangle. The
    /// sides and center are then stretched into place to cover the remaining
    /// destination rectangle.
    #[doc(alias = "SDL_BlitSurface9Grid")]
    pub fn blit_9grid(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
        (left_width, right_width, top_height, bottom_height): (i32, i32, i32, i32),
        scale: f32,
        scale_mode: ScaleMode,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_BlitSurface9Grid(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                left_width,
                right_width,
                top_height,
                bottom_height,
                scale,
                scale_mode.into(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Perform a scaled blit to a destination surface, which may be of a
    /// different format.
    ///
    /// `src` selects the source rectangle, or the entire surface if
    /// [`None`]. `dst` selects the target rectangle in the destination
    /// surface, or the entire destination surface if [`None`].
    #[doc(alias = "SDL_BlitSurfaceScaled")]
    pub fn blit_scaled(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
        scale_mode: ScaleMode,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_BlitSurfaceScaled(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
                scale_mode.into(),
            )
        })
    }

    /// Perform a tiled blit to a destination surface, which may be of a
    /// different format.
    ///
    /// `src` selects the source rectangle, or the entire surface if
    /// [`None`]. `dst` selects the target rectangle in the destination
    /// surface, or the entire destination surface if [`None`].
    ///
    /// # Remarks
    ///
    /// The pixels in `src` will be repeated as many times as needed to
    /// completely fill `dst`.
    #[doc(alias = "SDL_BlitSurfaceTiled")]
    pub fn blit_tiled(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_BlitSurfaceTiled(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
            )
        })
    }

    /// Perform a scaled and tiled blit to a destination surface, which may
    /// be of a different format.
    ///
    /// `src` selects the source rectangle, or the entire surface if
    /// [`None`]. `dst` selects the target rectangle in the destination
    /// surface, or the entire destination surface if [`None`].
    ///
    /// `scale` is the scale used to transform the source rectangle into the
    /// destination, e.g. a 32x32 surface with a scale of 2 would fill 64x64
    /// tiles.
    ///
    /// # Remarks
    ///
    /// The pixels in `src` will be scaled and repeated as many times as
    /// needed to completely fill `dst`.
    #[doc(alias = "SDL_BlitSurfaceTiledWithScale")]
    pub fn blit_tiled_scaled(
        &self,
        target: Ref<Surface>,
        src: Option<&PointI32>,
        dst: Option<&PointI32>,
        scale: f32,
        scale_mode: ScaleMode,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_BlitSurfaceTiledWithScale(
                self.handle.as_ptr(),
                opt2ptr(src).cast(),
                scale,
                scale_mode.into(),
                target.handle.as_ptr(),
                opt2ptr(dst).cast(),
            )
        })
    }
}

impl traits::BlendMode for SurfaceHandle {
    /// Get the blend mode used for blit operations.
    #[doc(alias = "SDL_GetSurfaceBlendMode")]
    fn blend_mode(&self) -> BlendMode {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            SDL_GetSurfaceBlendMode(self.handle.as_ptr(), ret.as_mut_ptr());
            ret.assume_init().into()
        }
    }

    /// Set the blend mode used for blit operations.
    ///
    /// # Remarks
    ///
    /// To copy a surface to another surface (or texture) without blending
    /// with the existing data, the blend mode of the SOURCE surface should
    /// be set to [`BlendMode::None`].
    #[doc(alias = "SDL_SetSurfaceBlendMode")]
    fn set_blend_mode(&self, bm: BlendMode) {
        unsafe {
            SDL_SetSurfaceBlendMode(self.handle.as_ptr(), bm.into());
        }
    }
}

impl traits::ColorModU8 for SurfaceHandle {
    /// Get the additional color value multiplied into blit operations.
    #[doc(alias = "SDL_GetSurfaceColorMod")]
    fn rgb_mod_u8(&self) -> RgbU8 {
        let mut ret = MaybeUninit::<RgbU8>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetSurfaceColorMod(
                self.handle.as_ptr(),
                &raw mut (*ptr).r,
                &raw mut (*ptr).g,
                &raw mut (*ptr).b,
            );
            ret.assume_init()
        }
    }

    /// Get the additional alpha value used in blit operations.
    #[doc(alias = "SDL_GetSurfaceAlphaMod")]
    fn alpha_mod_u8(&self) -> u8 {
        let mut ret = MaybeUninit::uninit();

        unsafe {
            SDL_GetSurfaceAlphaMod(self.handle.as_ptr(), ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    /// Set an additional color value multiplied into blit operations.
    ///
    /// # Remarks
    ///
    /// When this surface is blitted, during the blit operation each source
    /// color channel is modulated by the appropriate color value according
    /// to the following formula:
    ///
    /// `srcC = srcC * (color / 255)`
    #[doc(alias = "SDL_SetSurfaceColorMod")]
    fn set_rgb_mod_u8(&self, rm: RgbU8) {
        unsafe { SDL_SetSurfaceColorMod(self.handle.as_ptr(), rm.r, rm.g, rm.b) };
    }

    /// Set an additional alpha value used in blit operations.
    ///
    /// # Remarks
    ///
    /// When this surface is blitted, during the blit operation the source
    /// alpha value is modulated by this alpha value according to the
    /// following formula:
    ///
    /// `srcA = srcA * (alpha / 255)`
    #[doc(alias = "SDL_SetSurfaceAlphaMod")]
    fn set_alpha_mod_u8(&self, am: u8) {
        unsafe { SDL_SetSurfaceAlphaMod(self.handle.as_ptr(), am) };
    }
}

impl Surface {
    /// Allocate a new surface with a specific pixel format.
    ///
    /// # Remarks
    ///
    /// The pixels of the new surface are initialized to zero.
    #[doc(alias = "SDL_CreateSurface")]
    pub fn from_size_and_format(size: PointI32, format: PixelFormat) -> Result<Self> {
        Self::from_ptr(unsafe { SDL_CreateSurface(size.x, size.y, format.into()) })
    }
}
