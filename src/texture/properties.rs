use std::{
    ffi::{c_char, c_int, c_void},
    marker::PhantomData,
    ptr::NonNull,
};

use sdl3_sys::{
    pixels::{SDL_Colorspace, SDL_PixelFormat},
    render::*,
};

use crate::{
    gpu,
    pixels::Colorspace,
    properties::Properties,
    resource::Ref,
    texture::{PixelFormat, Texture, TextureAccess},
};

/// Read-only properties of a texture, as documented by
/// [`SDL_GetTextureProperties`](https://wiki.libsdl.org/SDL3/SDL_GetTextureProperties).
///
/// Generic properties are returned bare since the docs guarantee their
/// existence; backend properties are returned as `Option` since they only
/// exist on their respective backends.
///
/// Covers the D3D11, D3D12, OpenGL, Vulkan and GPU backends. Not covered:
/// the Metal and OpenGLES2 backends, the plane-specific texture pointers,
/// the OpenGL texture target, and `SDL_PROP_TEXTURE_OPENGL_TEX_W_FLOAT` /
/// `TEX_H_FLOAT`.
#[derive(Clone, Copy)]
pub struct TextureProperties<'ctx, 'vid, 'wnd, 'rnd, 'tex> {
    inner: Ref<'tex, Properties>,
    marker: PhantomData<Ref<'tex, Texture<'ctx, 'vid, 'wnd, 'rnd>>>,
}

impl<'ctx, 'vid, 'wnd, 'rnd, 'tex> TextureProperties<'ctx, 'vid, 'wnd, 'rnd, 'tex> {
    pub(super) fn new(inner: Ref<'tex, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Returns the [`Colorspace`] describing the texture's colorspace.
    #[doc(alias = "SDL_PROP_TEXTURE_COLORSPACE_NUMBER")]
    pub fn colorspace(self) -> Colorspace {
        unsafe {
            let cs =
                SDL_Colorspace(self.inner.number(SDL_PROP_TEXTURE_COLORSPACE_NUMBER, 0) as u32);
            Colorspace::from_sdl_unchecked(cs)
        }
    }

    /// Returns the [`PixelFormat`] of the texture.
    #[doc(alias = "SDL_PROP_TEXTURE_FORMAT_NUMBER")]
    pub fn format(self) -> PixelFormat {
        unsafe {
            let pf = SDL_PixelFormat(self.inner.number(SDL_PROP_TEXTURE_FORMAT_NUMBER, 0) as c_int);
            PixelFormat::from_sdl_unchecked(pf)
        }
    }

    /// Returns the [`TextureAccess`] mode of the texture.
    #[doc(alias = "SDL_PROP_TEXTURE_ACCESS_NUMBER")]
    pub fn access(self) -> TextureAccess {
        unsafe {
            let ta =
                SDL_TextureAccess(self.inner.number(SDL_PROP_TEXTURE_ACCESS_NUMBER, 0) as c_int);
            TextureAccess::from_sdl_unchecked(ta)
        }
    }

    /// Returns the width of the texture in pixels.
    #[doc(alias = "SDL_PROP_TEXTURE_WIDTH_NUMBER")]
    pub fn width(self) -> i64 {
        unsafe { self.inner.number(SDL_PROP_TEXTURE_WIDTH_NUMBER, 0) }
    }

    /// Returns the height of the texture in pixels.
    #[doc(alias = "SDL_PROP_TEXTURE_HEIGHT_NUMBER")]
    pub fn height(self) -> i64 {
        unsafe { self.inner.number(SDL_PROP_TEXTURE_HEIGHT_NUMBER, 0) }
    }

    /// Returns the value of 100% diffuse white for HDR10 and floating-point
    /// textures.
    ///
    /// Higher values are displayed in the HDR headroom. This defaults to `100`
    /// for HDR10 textures and `1.0` for other textures.
    #[doc(alias = "SDL_PROP_TEXTURE_SDR_WHITE_POINT_FLOAT")]
    pub fn sdr_white_point(self) -> f32 {
        unsafe { self.inner.float(SDL_PROP_TEXTURE_SDR_WHITE_POINT_FLOAT, 0.) }
    }

    /// Returns the maximum dynamic range used by HDR10 and floating-point
    /// texture content, relative to the SDR white point.
    ///
    /// If this is defined, values outside the range supported by the display
    /// are scaled into the available HDR headroom; otherwise they are clipped.
    /// This defaults to `1.0` for SDR textures and `4.0` for HDR10 textures,
    /// and has no default for floating-point textures.
    #[doc(alias = "SDL_PROP_TEXTURE_HDR_HEADROOM_FLOAT")]
    pub fn hdr_headroom(self) -> f32 {
        unsafe { self.inner.float(SDL_PROP_TEXTURE_HDR_HEADROOM_FLOAT, 0.) }
    }

    /// (Direct3D 11 only) Returns the `ID3D11Texture2D` associated with the
    /// texture.
    #[doc(alias = "SDL_PROP_TEXTURE_D3D11_TEXTURE_POINTER")]
    pub fn d3d11_texture(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_TEXTURE_D3D11_TEXTURE_POINTER)
    }

    /// (Direct3D 12 only) Returns the `ID3D12Resource` associated with the
    /// texture.
    #[doc(alias = "SDL_PROP_TEXTURE_D3D12_TEXTURE_POINTER")]
    pub fn d3d12_texture(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_TEXTURE_D3D12_TEXTURE_POINTER)
    }

    /// (OpenGL only) Returns the `GLuint` texture associated with the texture.
    #[doc(alias = "SDL_PROP_TEXTURE_OPENGL_TEXTURE_NUMBER")]
    pub fn opengl_texture(self) -> Option<i64> {
        self.opt_number(SDL_PROP_TEXTURE_OPENGL_TEXTURE_NUMBER)
    }

    /// (Vulkan only) Returns the `VkImage` associated with the texture.
    ///
    /// For NV12-style textures, this is the single two-plane `VkImage` holding
    /// both the Y and UV planes. For YUV-style textures, it is the `VkImage`
    /// holding the Y plane.
    #[doc(alias = "SDL_PROP_TEXTURE_VULKAN_TEXTURE_NUMBER")]
    pub fn vulkan_texture(self) -> Option<i64> {
        self.opt_number(SDL_PROP_TEXTURE_VULKAN_TEXTURE_NUMBER)
    }

    /// (GPU only) Returns the [`gpu::Texture`] associated with the texture.
    ///
    /// # Safety
    ///
    /// The device (`'dev`) lifetime is inferred.
    #[doc(alias = "SDL_PROP_TEXTURE_GPU_TEXTURE_POINTER")]
    pub unsafe fn gpu_texture<'dev>(self) -> Option<Ref<'tex, gpu::Texture<'ctx, 'vid, 'dev>>> {
        self.opt_ptr(SDL_PROP_TEXTURE_GPU_TEXTURE_POINTER)
            .map(|ptr| unsafe {
                let handle = gpu::TextureHandle::from_ptr(ptr.as_ptr().cast()).unwrap_unchecked();
                Ref::from_handle(handle)
            })
    }

    fn opt_number(self, key: *const c_char) -> Option<i64> {
        unsafe { self.inner.has(key).then(|| self.inner.number(key, 0)) }
    }

    fn opt_ptr(self, key: *const c_char) -> Option<NonNull<c_void>> {
        let p = unsafe { self.inner.pointer(key, std::ptr::null_mut()) };
        NonNull::new(p)
    }
}
