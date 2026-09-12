//! GPU textures residing in VRAM.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryRender)):
//! - [x] SDL_CreateTexture
//! - [x] SDL_CreateTextureFromSurface
//! - [x] SDL_CreateTextureWithProperties
//! - [x] SDL_DestroyTexture
//! - [ ] SDL_GetDefaultTextureScaleMode
//! - [x] SDL_GetTextureAlphaMod
//! - [x] SDL_GetTextureAlphaModFloat
//! - [x] SDL_GetTextureBlendMode
//! - [x] SDL_GetTextureColorMod
//! - [x] SDL_GetTextureColorModFloat
//! - [x] SDL_GetTextureProperties
//! - [x] SDL_GetTextureScaleMode
//! - [x] SDL_GetTextureSize
//! - [ ] SDL_LockTexture
//! - [ ] SDL_LockTextureToSurface
//! - [ ] SDL_SetDefaultTextureScaleMode
//! - [x] SDL_SetTextureAlphaMod
//! - [x] SDL_SetTextureAlphaModFloat
//! - [x] SDL_SetTextureBlendMode
//! - [x] SDL_SetTextureColorMod
//! - [x] SDL_SetTextureColorModFloat
//! - [x] SDL_SetTextureScaleMode
//! - [ ] SDL_UnlockTexture
//! - [ ] SDL_UpdateNVTexture
//! - [ ] SDL_UpdateTexture
//! - [ ] SDL_UpdateYUVTexture
//! - [x] SDL_GetRendererFromTexture

use std::mem::MaybeUninit;

use sdl3_sys::{render::*, surface::SDL_ScaleMode};

use crate::{
    Result,
    color::{RgbF32, RgbU8},
    pixels::{BlendMode, PixelFormat, ScaleMode},
    properties::{Properties, PropertiesHandle},
    rect::{PointF32, PointI32},
    renderer::{Renderer, RendererHandle},
    resource::Ref,
    resource::resource_new,
    surface::Surface,
    traits,
    util::impl_enum_transmute,
    util::mod_reexport,
};

mod_reexport!(builder);
mod_reexport!(properties);

/// The access pattern allowed for a texture.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[doc(alias = "SDL_TextureAccess")]
pub enum TextureAccess {
    /// Changes rarely, not lockable.
    Static = SDL_TextureAccess::STATIC.0,
    /// Changes frequently, lockable.
    Streaming = SDL_TextureAccess::STREAMING.0,
    /// Texture can be used as a render target.
    Target = SDL_TextureAccess::TARGET.0,
}

impl_enum_transmute!(SDL_TextureAccess, TextureAccess);

resource_new! {
    /// An efficient driver-specific representation of pixel data.
    pub struct Texture<> : SDL_Texture, ~SDL_DestroyTexture {
        marker: PhantomData<()>,
    }
}

impl TextureHandle {
    /// Get the size of a texture, as floating point values.
    #[doc(alias = "SDL_GetTextureSize")]
    pub fn size(&self) -> PointF32 {
        let mut ret = MaybeUninit::<PointF32>::uninit();
        let ptr = ret.as_mut_ptr();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureSize(self.handle.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the scale mode used for texture scale operations.
    #[doc(alias = "SDL_GetTextureScaleMode")]
    pub fn scale_mode(&self) -> ScaleMode {
        let mut ret = MaybeUninit::<SDL_ScaleMode>::uninit();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureScaleMode(self.handle.as_ptr(), ret.as_mut_ptr());
            ScaleMode::from_sdl_unchecked(ret.assume_init())
        }
    }

    /// Get the renderer that created a texture.
    ///
    /// Returns [`None`] on failure.
    #[doc(alias = "SDL_GetRendererFromTexture")]
    pub fn renderer(&self) -> Option<RendererHandle> {
        RendererHandle::from_ptr(unsafe { SDL_GetRendererFromTexture(self.handle.as_ptr()) })
    }

    /// Set the scale mode used for texture scale operations.
    ///
    /// # Remarks
    ///
    /// The default texture scale mode is [`ScaleMode::Linear`].
    ///
    /// If the scale mode is not supported, the closest supported mode is
    /// chosen.
    #[doc(alias = "SDL_SetTextureScaleMode")]
    pub fn set_scale_mode(&mut self, sm: ScaleMode) {
        unsafe {
            SDL_SetTextureScaleMode(self.handle.as_ptr(), sm.to_sdl());
        }
    }
}

impl traits::BlendMode for TextureHandle {
    /// Get the blend mode used for texture copy operations.
    #[doc(alias = "SDL_GetTextureBlendMode")]
    fn blend_mode(&self) -> BlendMode {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            SDL_GetTextureBlendMode(self.handle.as_ptr(), ret.as_mut_ptr());
            BlendMode::from_sdl_unchecked(ret.assume_init())
        }
    }

    /// Set the blend mode for a texture, used by
    /// [`RendererHandle::draw`] and its variants.
    ///
    /// # Remarks
    ///
    /// If the blend mode is not supported, the closest supported mode is
    /// chosen.
    #[doc(alias = "SDL_SetTextureBlendMode")]
    fn set_blend_mode(&self, bm: BlendMode) {
        unsafe {
            SDL_SetTextureBlendMode(self.handle.as_ptr(), bm.to_sdl());
        }
    }
}

impl traits::ColorModU8 for TextureHandle {
    /// Get the additional color value multiplied into render copy
    /// operations.
    #[doc(alias = "SDL_GetTextureColorMod")]
    fn rgb_mod_u8(&self) -> RgbU8 {
        let mut ret = MaybeUninit::<RgbU8>::uninit();
        let ptr = ret.as_mut_ptr();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureColorMod(
                self.handle.as_ptr(),
                &raw mut (*ptr).r,
                &raw mut (*ptr).g,
                &raw mut (*ptr).b,
            );

            ret.assume_init()
        }
    }

    /// Get the additional alpha value multiplied into render copy
    /// operations.
    #[doc(alias = "SDL_GetTextureAlphaMod")]
    fn alpha_mod_u8(&self) -> u8 {
        let mut ret = MaybeUninit::<u8>::uninit();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureAlphaMod(self.handle.as_ptr(), ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    /// Set an additional color value multiplied into render copy
    /// operations.
    ///
    /// # Remarks
    ///
    /// When this texture is rendered, during the copy operation each source
    /// color channel is modulated by the appropriate color value according
    /// to the following formula:
    ///
    /// `srcC = srcC * (color / 255)`
    ///
    /// Color modulation is not always supported by the renderer.
    #[doc(alias = "SDL_SetTextureColorMod")]
    fn set_rgb_mod_u8(&self, rm: RgbU8) {
        unsafe {
            SDL_SetTextureColorMod(self.handle.as_ptr(), rm.r, rm.g, rm.b);
        }
    }

    /// Set an additional alpha value multiplied into render copy
    /// operations.
    ///
    /// # Remarks
    ///
    /// When this texture is rendered, during the copy operation the source
    /// alpha value is modulated by this alpha value according to the
    /// following formula:
    ///
    /// `srcA = srcA * (alpha / 255)`
    ///
    /// Alpha modulation is not always supported by the renderer.
    #[doc(alias = "SDL_SetTextureAlphaMod")]
    fn set_alpha_mod_u8(&self, am: u8) {
        unsafe {
            SDL_SetTextureAlphaMod(self.handle.as_ptr(), am);
        }
    }
}

impl traits::ColorModF32 for TextureHandle {
    /// Get the additional color value multiplied into render copy
    /// operations.
    #[doc(alias = "SDL_GetTextureColorModFloat")]
    fn rgb_mod_f32(&self) -> RgbF32 {
        let mut ret = MaybeUninit::<RgbF32>::uninit();
        let ptr = ret.as_mut_ptr();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureColorModFloat(
                self.handle.as_ptr(),
                &raw mut (*ptr).r,
                &raw mut (*ptr).g,
                &raw mut (*ptr).b,
            );

            ret.assume_init()
        }
    }

    /// Get the additional alpha value multiplied into render copy
    /// operations.
    #[doc(alias = "SDL_GetTextureAlphaModFloat")]
    fn alpha_mod_f32(&self) -> f32 {
        let mut ret = MaybeUninit::<f32>::uninit();

        // SAFETY: This function only reads struct fields.
        unsafe {
            SDL_GetTextureAlphaModFloat(self.handle.as_ptr(), ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    /// Set an additional color value multiplied into render copy
    /// operations.
    ///
    /// # Remarks
    ///
    /// When this texture is rendered, during the copy operation each source
    /// color channel is modulated by the appropriate color value according
    /// to the following formula:
    ///
    /// `srcC = srcC * color`
    ///
    /// Color modulation is not always supported by the renderer.
    #[doc(alias = "SDL_SetTextureColorModFloat")]
    fn set_rgb_mod_f32(&self, rm: RgbF32) {
        unsafe {
            SDL_SetTextureColorModFloat(self.handle.as_ptr(), rm.r, rm.g, rm.b);
        }
    }

    /// Set an additional alpha value multiplied into render copy
    /// operations.
    ///
    /// # Remarks
    ///
    /// When this texture is rendered, during the copy operation the source
    /// alpha value is modulated by this alpha value according to the
    /// following formula:
    ///
    /// `srcA = srcA * alpha`
    ///
    /// Alpha modulation is not always supported by the renderer.
    #[doc(alias = "SDL_SetTextureAlphaModFloat")]
    fn set_alpha_mod_f32(&self, am: f32) {
        unsafe {
            SDL_SetTextureAlphaModFloat(self.handle.as_ptr(), am);
        }
    }
}

impl TextureHandle {
    /// Get the properties associated with a texture.
    ///
    /// Read-only properties of this texture, as documented by
    /// [`SDL_GetTextureProperties`](https://wiki.libsdl.org/SDL3/SDL_GetTextureProperties).
    ///
    /// Covers the generic properties plus the D3D11, D3D12, OpenGL, Vulkan
    /// and GPU backends.
    ///
    /// Not covered:
    /// - the Metal and OpenGLES2 backends
    /// - plane-specific texture pointers
    /// - the OpenGL texture target
    /// - `SDL_PROP_TEXTURE_OPENGL_TEX_{W,H}_FLOAT`
    #[doc(alias = "SDL_GetTextureProperties")]
    pub fn properties(&self) -> TextureProperties<'_> {
        unsafe {
            let id = SDL_GetTextureProperties(self.handle.as_ptr());
            let handle = PropertiesHandle::from_id(id).unwrap_unchecked();
            let r = Ref::from_handle(handle);

            TextureProperties::new(r)
        }
    }
}

impl Texture {
    /// Bind the builder to a renderer and an existing property group.
    ///
    /// Unlike the window, renderer and GPU device builders, the renderer is
    /// a required parameter here, since `SDL_CreateTextureWithProperties`
    /// takes it directly.
    pub fn builder<'a>(props: Ref<'a, Properties>) -> TextureBuilder<'a> {
        TextureBuilder::new(props)
    }

    /// Create a texture for a rendering context.
    ///
    /// `fmt` is the pixel format of the texture, `access` its allowed access
    /// pattern, and `size` its width and height in pixels.
    ///
    /// # Remarks
    ///
    /// The contents of a texture when first created are not defined.
    #[doc(alias = "SDL_CreateTexture")]
    pub fn new(
        rnd: Ref<Renderer>,
        fmt: PixelFormat,
        access: TextureAccess,
        size: PointI32,
    ) -> Result<Texture> {
        Self::from_ptr(unsafe {
            SDL_CreateTexture(
                rnd.handle.as_ptr(),
                fmt.to_sdl(),
                access.into(),
                size.x,
                size.y,
            )
        })
    }

    /// Create a texture from an existing surface.
    ///
    /// # Remarks
    ///
    /// The surface is not modified or freed by this function.
    ///
    /// The access hint for the created texture is
    /// [`TextureAccess::Static`].
    ///
    /// The pixel format of the created texture may be different from the
    /// pixel format of the surface, and can be queried using the
    /// `SDL_PROP_TEXTURE_FORMAT_NUMBER` property (see
    /// [`TextureHandle::properties`]).
    #[doc(alias = "SDL_CreateTextureFromSurface")]
    pub fn from_surface(rnd: Ref<Renderer>, surf: Ref<Surface>) -> Result<Texture> {
        Self::from_ptr(unsafe {
            SDL_CreateTextureFromSurface(rnd.handle.as_ptr(), surf.handle.as_ptr())
        })
    }
}
