use std::ffi::c_char;

use sdl3_sys::render::*;

use crate::{
    Result,
    pixels::Colorspace,
    pixels::PixelFormat,
    properties::Properties,
    rect::PointI32,
    renderer::Renderer,
    resource::Ref,
    texture::{Texture, TextureAccess},
};

const CREATE_PROPERTIES: [*const c_char; 7] = [
    SDL_PROP_TEXTURE_CREATE_COLORSPACE_NUMBER,
    SDL_PROP_TEXTURE_CREATE_FORMAT_NUMBER,
    SDL_PROP_TEXTURE_CREATE_ACCESS_NUMBER,
    SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER,
    SDL_PROP_TEXTURE_CREATE_HEIGHT_NUMBER,
    SDL_PROP_TEXTURE_CREATE_SDR_WHITE_POINT_FLOAT,
    SDL_PROP_TEXTURE_CREATE_HDR_HEADROOM_FLOAT,
];

/// Builder for [`Texture`], using
/// [`SDL_CreateTextureWithProperties`](https://wiki.libsdl.org/SDL3/SDL_CreateTextureWithProperties).
///
/// The backend-specific properties that wrap existing native textures
/// (D3D11, D3D12, Metal, OpenGL, OpenGLES2, Vulkan and GPU), as well as
/// [`SDL_PROP_TEXTURE_CREATE_PALETTE_POINTER`], are not covered.
pub struct TextureBuilder<'a> {
    inner: Ref<'a, Properties>,
}

impl<'a> TextureBuilder<'a> {
    pub(super) fn new(inner: Ref<'a, Properties>) -> Self {
        Self { inner }
    }

    /// A [`Colorspace`] value describing the texture colorspace. Defaults
    /// to `SDL_COLORSPACE_SRGB_LINEAR` for floating point textures,
    /// `SDL_COLORSPACE_HDR10` for 10-bit textures, `SDL_COLORSPACE_SRGB` for
    /// other RGB textures and `SDL_COLORSPACE_JPEG` for YUV textures.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_COLORSPACE_NUMBER")]
    pub fn colorspace(&mut self, value: Colorspace) -> &mut Self {
        self.set_number(
            SDL_PROP_TEXTURE_CREATE_COLORSPACE_NUMBER,
            i64::from(value as u32),
        )
    }

    /// One of the enumerated values in [`PixelFormat`]. Defaults to the
    /// best RGBA format for the renderer.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_FORMAT_NUMBER")]
    pub fn format(&mut self, value: PixelFormat) -> &mut Self {
        self.set_number(
            SDL_PROP_TEXTURE_CREATE_FORMAT_NUMBER,
            i64::from(value as i32),
        )
    }

    /// One of the enumerated values in [`TextureAccess`]. Defaults to
    /// [`TextureAccess::Static`].
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_ACCESS_NUMBER")]
    pub fn access(&mut self, value: TextureAccess) -> &mut Self {
        self.set_number(
            SDL_PROP_TEXTURE_CREATE_ACCESS_NUMBER,
            i64::from(value as i32),
        )
    }

    /// The width of the texture in pixels. Required.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER")]
    pub fn width(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER, value)
    }

    /// The height of the texture in pixels. Required.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_HEIGHT_NUMBER")]
    pub fn height(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_TEXTURE_CREATE_HEIGHT_NUMBER, value)
    }

    /// Utility method that calls `self.width()` and `self.height()`.
    pub fn size(&mut self, size: PointI32) -> &mut Self {
        self.width(size.x.into());
        self.height(size.y.into())
    }

    /// For HDR10 and floating point textures, the value of 100% diffuse
    /// white, with higher values being displayed in the High Dynamic Range
    /// headroom. Defaults to 100 for HDR10 textures and 1.0 for floating
    /// point textures.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_SDR_WHITE_POINT_FLOAT")]
    pub fn sdr_white_point(&mut self, value: f32) -> &mut Self {
        self.set_float(SDL_PROP_TEXTURE_CREATE_SDR_WHITE_POINT_FLOAT, value)
    }

    /// For HDR10 and floating point textures, the maximum dynamic range used
    /// by the content, in terms of the SDR white point. If defined, any
    /// values outside the range supported by the display will be scaled into
    /// the available HDR headroom, otherwise they are clipped.
    #[doc(alias = "SDL_PROP_TEXTURE_CREATE_HDR_HEADROOM_FLOAT")]
    pub fn hdr_headroom(&mut self, value: f32) -> &mut Self {
        self.set_float(SDL_PROP_TEXTURE_CREATE_HDR_HEADROOM_FLOAT, value)
    }

    /// Clear all texture creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the texture.
    #[doc(alias = "SDL_CreateTextureWithProperties")]
    pub fn build(&self, rnd: Ref<Renderer>) -> Result<Texture> {
        Texture::from_ptr(unsafe {
            SDL_CreateTextureWithProperties(rnd.handle.as_ptr(), self.inner.id())
        })
    }

    /// Build the texture, and cleanup all properties.
    /// See the [crate::properties] module docs for more info.
    #[doc(alias = "SDL_CreateTextureWithProperties")]
    pub fn build_cleanup(&self, rnd: Ref<Renderer>) -> Result<Texture> {
        let res = Texture::from_ptr(unsafe {
            SDL_CreateTextureWithProperties(rnd.handle.as_ptr(), self.inner.id())
        });
        Self::clear_from(self.inner);
        res
    }

    fn set_number(&mut self, key: *const c_char, value: i64) -> &mut Self {
        _ = unsafe { self.inner.set_number(key, value) };
        self
    }

    fn set_float(&mut self, key: *const c_char, value: f32) -> &mut Self {
        _ = unsafe { self.inner.set_float(key, value) };
        self
    }
}
