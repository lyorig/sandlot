//! Various enums related to pixels and/or graphics.
//!
//! SDL offers facilities for pixel management.
//!
//! Largely these facilities deal with pixel _format_: what does this set of
//! bits represent?
//!
//! If you mostly want to think of a pixel as some combination of red, green,
//! blue, and maybe alpha intensities, this is all pretty straightforward, and
//! in many cases, is enough information to build a perfectly fine game.
//!
//! However, the actual definition of a pixel is more complex than that:
//!
//! Pixels are a representation of a color in a particular color space.
//!
//! The first characteristic of a color space is the color type. SDL
//! understands two different color types, RGB and YCbCr, or in SDL also
//! referred to as YUV.
//!
//! RGB colors consist of red, green, and blue channels of color that are added
//! together to represent the colors we see on the screen.
//!
//! <https://en.wikipedia.org/wiki/RGB_color_model>
//!
//! YCbCr colors represent colors as a Y luma brightness component and red and
//! blue chroma color offsets. This color representation takes advantage of
//! the fact that the human eye is more sensitive to brightness than the
//! color in an image. The Cb and Cr components are often compressed and have
//! lower resolution than the luma component.
//!
//! <https://en.wikipedia.org/wiki/YCbCr>
//!
//! When the color information in YCbCr is compressed, the Y pixels are left
//! at full resolution and each Cr and Cb pixel represents an average of the
//! color information in a block of Y pixels. The chroma location determines
//! where in that block of pixels the color information is coming from.
//!
//! The color range defines how much of the pixel to use when converting a
//! pixel into a color on the display. When the full color range is used, the
//! entire numeric range of the pixel bits is significant. When narrow color
//! range is used, for historical reasons, the pixel uses only a portion of
//! the numeric range to represent colors.
//!
//! The color primaries and white point are a definition of the colors in the
//! color space relative to the standard XYZ color space.
//!
//! <https://en.wikipedia.org/wiki/CIE_1931_color_space>
//!
//! The transfer characteristic, or opto-electrical transfer function (OETF),
//! is the way a color is converted from mathematically linear space into a
//! non-linear output signals.
//!
//! <https://en.wikipedia.org/wiki/Rec._709#Transfer_characteristics>
//!
//! The matrix coefficients are used to convert between YCbCr and RGB colors.
//!
//! Blend modes decide how two colors will mix together. There are both
//! standard modes for basic needs and a means to create custom modes
//! ([`BlendMode::compose`]), dictating what sort of math to do on what color
//! components.
//!
//! Implementation checklist:
//!
//! From [CategoryPixels](https://wiki.libsdl.org/SDL3/CategoryPixels):
//! - [x] SDL_PixelFormat
//! - [x] SDL_Colorspace
//! - [ ] SDL_ArrayOrder
//! - [ ] SDL_BitmapOrder
//! - [ ] SDL_ChromaLocation
//! - [ ] SDL_ColorPrimaries
//! - [ ] SDL_ColorRange
//! - [ ] SDL_ColorType
//! - [ ] SDL_MatrixCoefficients
//! - [ ] SDL_PackedLayout
//! - [ ] SDL_PackedOrder
//! - [ ] SDL_PixelType
//! - [ ] SDL_TransferCharacteristics
//! - [ ] SDL_CreatePalette
//! - [ ] SDL_DestroyPalette
//! - [x] SDL_GetMasksForPixelFormat
//! - [ ] SDL_GetPixelFormatDetails
//! - [x] SDL_GetPixelFormatForMasks
//! - [x] SDL_GetPixelFormatName
//! - [x] SDL_GetRGB (impl'd as [`RgbU8::from_pixel`](crate::color::RgbU8::from_pixel))
//! - [x] SDL_GetRGBA (impl'd as [`RgbaU8::from_pixel`](crate::color::RgbaU8::from_pixel))
//! - [x] SDL_MapRGB (impl'd as [`RgbU8::map`](crate::color::RgbU8::map))
//! - [x] SDL_MapRGBA (impl'd as [`RgbaU8::map`](crate::color::RgbaU8::map))
//! - [x] SDL_MapSurfaceRGB (impl'd as [`SurfaceHandle::map_rgb`](crate::surface::SurfaceHandle::map_rgb))
//! - [x] SDL_MapSurfaceRGBA (impl'd as [`SurfaceHandle::map_rgba`](crate::surface::SurfaceHandle::map_rgba))
//!
//! From [CategoryBlendmode](https://wiki.libsdl.org/SDL3/CategoryBlendmode):
//! - [x] SDL_BlendMode
//! - [x] SDL_BlendFactor
//! - [x] SDL_BlendOperation
//! - [x] SDL_ComposeCustomBlendMode

use std::mem::MaybeUninit;

use sdl3_sys::{blendmode::*, pixels::*, surface::SDL_ScaleMode};

use crate::{Result, error::Error, impl_enum_transmute, util::c_ptr_to_str};

/// A set of blend modes used in drawing operations.
///
/// # Remarks
///
/// Several predefined blend modes, exposed via associated `const` items, are supported everywhere.
///
/// Custom blend modes may be composed via [`BlendMode::compose`].
#[derive(Clone, Copy)]
#[doc(alias = "SDL_BlendMode")]
pub struct BlendMode(SDL_BlendMode);

impl BlendMode {
    /// No blending: `dstRGBA = srcRGBA`.
    pub const NONE: Self = Self(SDL_BlendMode::NONE);

    /// Alpha blending:
    /// `dstRGB = (srcRGB * srcA) + (dstRGB * (1-srcA))`,
    /// `dstA = srcA + (dstA * (1-srcA))`.
    pub const BLEND: Self = Self(SDL_BlendMode::BLEND);

    /// Pre-multiplied alpha blending:
    /// `dstRGBA = srcRGBA + (dstRGBA * (1-srcA))`.
    pub const BLEND_PREMUL: Self = Self(SDL_BlendMode::BLEND_PREMULTIPLIED);

    /// Additive blending:
    /// `dstRGB = (srcRGB * srcA) + dstRGB`, `dstA = dstA`.
    pub const ADD: Self = Self(SDL_BlendMode::ADD);

    /// Pre-multiplied additive blending:
    /// `dstRGB = srcRGB + dstRGB`, `dstA = dstA`.
    pub const ADD_PREMUL: Self = Self(SDL_BlendMode::ADD_PREMULTIPLIED);

    /// Color modulate: `dstRGB = srcRGB * dstRGB`, `dstA = dstA`.
    pub const MOD: Self = Self(SDL_BlendMode::MOD);

    /// Color multiply:
    /// `dstRGB = (srcRGB * dstRGB) + (dstRGB * (1-srcA))`, `dstA = dstA`.
    pub const MUL: Self = Self(SDL_BlendMode::MUL);

    /// Compose a custom blend mode for renderers.
    ///
    /// * `src_color` is the factor applied to the red, green, and blue
    ///   components of the source pixels.
    /// * `dst_color` is the factor applied to the red, green, and blue
    ///   components of the destination pixels.
    /// * `color_op` is the operation used to combine the red, green, and blue
    ///   components of the source and destination pixels.
    /// * `src_alpha` is the factor applied to the alpha component of the source
    ///   pixels.
    /// * `dst_alpha` is the factor applied to the alpha component of the
    ///   destination pixels.
    /// * `alpha_op` is the operation used to combine the alpha component of the
    ///   source and destination pixels.
    ///
    /// A blend mode controls how the pixels from a drawing operation (source)
    /// get combined with the pixels from the render target (destination).
    /// First, the components of the source and destination pixels get
    /// multiplied with their blend factors. Then, the blend operation takes the
    /// two products and calculates the result that will get stored in the
    /// render target. Expressed in pseudocode:
    ///
    /// ```text
    /// dstRGB = color_op(srcRGB * src_color, dstRGB * dst_color);
    /// dstA = alpha_op(srcA * src_alpha, dstA * dst_alpha);
    /// ```
    ///
    /// Where the operation functions can return `src + dst`, `src - dst`,
    /// `dst - src`, `min(src, dst)`, or `max(src, dst)`.
    ///
    /// The red, green, and blue components are always multiplied with the
    /// first, second, and third components of [`BlendFactor`], respectively;
    /// the fourth component is not used. The alpha component is always
    /// multiplied with the fourth component; the other components are not used
    /// in the alpha calculation.
    ///
    /// Support for these blend modes varies for each renderer: pass the
    /// returned mode to a blend-mode setter (e.g. for a texture or the draw
    /// color) and treat an error there as "unsupported". All renderers support
    /// the predefined modes above.
    ///
    /// Note that some renderers do not provide an alpha component for the
    /// default render target; [`BlendFactor::DstAlpha`] and
    /// [`BlendFactor::OneMinusDstAlpha`] have no effect in that case.
    #[doc(alias = "SDL_ComposeCustomBlendMode")]
    pub fn compose(
        (src_color, dst_color, color_op): (BlendFactor, BlendFactor, BlendOperation),
        (src_alpha, dst_alpha, alpha_op): (BlendFactor, BlendFactor, BlendOperation),
    ) -> Self {
        let bm = SDL_ComposeCustomBlendMode(
            src_color.into(),
            dst_color.into(),
            color_op.into(),
            src_alpha.into(),
            dst_alpha.into(),
            alpha_op.into(),
        );

        bm.into()
    }
}

impl_enum_transmute!(SDL_BlendMode, BlendMode);

/// The normalized factor used to multiply pixel components.
///
/// # Remarks
///
/// The blend factors are multiplied with the pixels from a drawing operation
/// (src) and the pixels from the render target (dst) before the blend
/// operation. The comma-separated factors listed in each variant are always
/// applied in the component order red, green, blue, and alpha.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_BlendFactor")]
pub enum BlendFactor {
    /// `0, 0, 0, 0`.
    Zero = SDL_BlendFactor::ZERO.0,

    /// `1, 1, 1, 1`.
    One = SDL_BlendFactor::ONE.0,

    /// `srcR, srcG, srcB, srcA`.
    SrcColor = SDL_BlendFactor::SRC_COLOR.0,

    /// `1-srcR, 1-srcG, 1-srcB, 1-srcA`.
    OneMinusSrcColor = SDL_BlendFactor::ONE_MINUS_SRC_COLOR.0,

    /// `srcA, srcA, srcA, srcA`.
    SrcAlpha = SDL_BlendFactor::SRC_ALPHA.0,

    /// `1-srcA, 1-srcA, 1-srcA, 1-srcA`.
    OneMinusSrcAlpha = SDL_BlendFactor::ONE_MINUS_SRC_ALPHA.0,

    /// `dstR, dstG, dstB, dstA`.
    DstColor = SDL_BlendFactor::DST_COLOR.0,

    /// `1-dstR, 1-dstG, 1-dstB, 1-dstA`.
    OneMinusDstColor = SDL_BlendFactor::ONE_MINUS_DST_COLOR.0,

    /// `dstA, dstA, dstA, dstA`.
    DstAlpha = SDL_BlendFactor::DST_ALPHA.0,

    /// `1-dstA, 1-dstA, 1-dstA, 1-dstA`.
    OneMinusDstAlpha = SDL_BlendFactor::ONE_MINUS_DST_ALPHA.0,
}

impl_enum_transmute!(SDL_BlendFactor, BlendFactor);

/// The blend operation used when combining source and destination pixel
/// components.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_BlendOperation")]
pub enum BlendOperation {
    /// `dst + src`: supported by all renderers.
    Add = SDL_BlendOperation::ADD.0,

    /// `src - dst`: supported by D3D, OpenGL, OpenGLES, and Vulkan.
    Subtract = SDL_BlendOperation::SUBTRACT.0,

    /// `dst - src`: supported by D3D, OpenGL, OpenGLES, and Vulkan.
    RevSubtract = SDL_BlendOperation::REV_SUBTRACT.0,

    /// `min(dst, src)`: supported by D3D, OpenGL, OpenGLES, and Vulkan.
    Minimum = SDL_BlendOperation::MINIMUM.0,

    /// `max(dst, src)`: supported by D3D, OpenGL, OpenGLES, and Vulkan.
    Maximum = SDL_BlendOperation::MAXIMUM.0,
}

impl_enum_transmute!(SDL_BlendOperation, BlendOperation);

/// The scaling mode.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[doc(alias = "SDL_ScaleMode")]
pub enum ScaleMode {
    /// Nearest pixel sampling.
    Nearest = SDL_ScaleMode::NEAREST.0,
    /// Linear filtering.
    Linear = SDL_ScaleMode::LINEAR.0,
    /// Nearest pixel sampling with improved scaling for pixel art.
    PixelArt = SDL_ScaleMode::PIXELART.0,
}

impl_enum_transmute!(SDL_ScaleMode, ScaleMode);

/// Colorspace definitions.
///
/// # Remarks
///
/// Since similar colorspaces may vary in their details (matrix, transfer
/// function, etc.), this is not an exhaustive list, but rather a
/// representative sample of the kinds of colorspaces supported in SDL.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[doc(alias = "SDL_Colorspace")]
pub enum Colorspace {
    /// A gamma corrected colorspace, and the default colorspace for SDL
    /// rendering and 8-bit RGB surfaces.
    ///
    /// Equivalent to `DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709`.
    Srgb = SDL_Colorspace::SRGB.0,
    /// A linear colorspace and the default colorspace for floating point
    /// surfaces. On Windows this is the scRGB colorspace, and on Apple
    /// platforms this is `kCGColorSpaceExtendedLinearSRGB` for EDR content.
    ///
    /// Equivalent to `DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709`.
    SrgbLinear = SDL_Colorspace::SRGB_LINEAR.0,
    /// A non-linear HDR colorspace and the default colorspace for 10-bit
    /// surfaces.
    ///
    /// Equivalent to `DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020`.
    Hdr10 = SDL_Colorspace::HDR10.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_FULL_G22_NONE_P709_X601`.
    Jpeg = SDL_Colorspace::JPEG.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P601`.
    Bt601Limited = SDL_Colorspace::BT601_LIMITED.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_FULL_G22_LEFT_P601`.
    Bt601Full = SDL_Colorspace::BT601_FULL.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709`.
    Bt709Limited = SDL_Colorspace::BT709_LIMITED.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_FULL_G22_LEFT_P709`.
    Bt709Full = SDL_Colorspace::BT709_FULL.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P2020`.
    Bt2020Limited = SDL_Colorspace::BT2020_LIMITED.0,
    /// Equivalent to `DXGI_COLOR_SPACE_YCBCR_FULL_G22_LEFT_P2020`.
    Bt2020Full = SDL_Colorspace::BT2020_FULL.0,
}

impl Colorspace {
    /// The default colorspace for RGB surfaces if no colorspace is specified.
    pub const RGB_DEFAULT: Self = Self::from_sdl(SDL_Colorspace::RGB_DEFAULT);
    /// The default colorspace for YUV surfaces if no colorspace is specified.
    pub const YUV_DEFAULT: Self = Self::from_sdl(SDL_Colorspace::YUV_DEFAULT);
}

impl_enum_transmute!(SDL_Colorspace, Colorspace);

/// Contains parameters for [`PixelFormat::from_mask`] and [`PixelFormat::mask`].
#[derive(Clone, Copy)]
pub struct PixelFormatMask {
    /// Bits-per-pixel (usually 15, 16, or 32).
    pub bpp: i32,
    /// Red color mask.
    pub r: u32,
    /// Green color mask.
    pub g: u32,
    /// Blue color mask.
    pub b: u32,
    /// Alpha (opacity) mask.
    pub a: u32,
}

impl PixelFormatMask {
    pub fn new(bpp: i32, r: u32, g: u32, b: u32, a: u32) -> Self {
        Self { bpp, r, g, b, a }
    }
}

/// Pixel format.
///
/// # Remarks
///
/// SDL's pixel formats have the following naming convention:
///
/// - Names with a list of components and a single bit count, such as `RGB24`
///   and `ABGR32`, define a platform-independent encoding into bytes in the
///   order specified. For example, in `RGB24` data, each pixel is encoded in
///   3 bytes (red, green, blue) in that order, and in `ABGR32` data, each
///   pixel is encoded in 4 bytes (alpha, blue, green, red) in that order.
///   Use these names if the property of a format that is important to you is
///   the order of the bytes in memory or on disk.
/// - Names with a bit count per component, such as `ARGB8888` and
///   `XRGB1555`, are "packed" into an appropriately-sized integer in the
///   platform's native endianness. For example, `ARGB8888` is a sequence of
///   32-bit integers; in each integer, the most significant bits are alpha,
///   and the least significant bits are blue. On a little-endian CPU such as
///   x86, the least significant bits of each integer are arranged first in
///   memory, but on a big-endian CPU such as s390x, the most significant
///   bits are arranged first. Use these names if the property of a format
///   that is important to you is the meaning of each bit position within a
///   native-endianness integer.
/// - In indexed formats such as `INDEX4LSB`, each pixel is represented by
///   encoding an index into the palette into the indicated number of bits,
///   with multiple pixels packed into each byte if appropriate. In LSB
///   formats, the first (leftmost) pixel is stored in the least-significant
///   bits of the byte; in MSB formats, it's stored in the most-significant
///   bits. `INDEX8` does not need LSB/MSB variants, because each pixel
///   exactly fills one byte.
///
/// The 32-bit byte-array encodings such as [`PixelFormat::RGBA32`] are
/// aliases for the appropriate 8888 encoding for the current platform. For
/// example, `RGBA32` is an alias for `ABGR8888` on little-endian CPUs like
/// x86, or an alias for `RGBA8888` on big-endian CPUs.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[doc(alias = "SDL_PixelFormat")]
pub enum PixelFormat {
    Index1Lsb = SDL_PixelFormat::INDEX1LSB.0,
    Index1Msb = SDL_PixelFormat::INDEX1MSB.0,
    Index2Lsb = SDL_PixelFormat::INDEX2LSB.0,
    Index2Msb = SDL_PixelFormat::INDEX2MSB.0,
    Index4Lsb = SDL_PixelFormat::INDEX4LSB.0,
    Index4Msb = SDL_PixelFormat::INDEX4MSB.0,
    Index8 = SDL_PixelFormat::INDEX8.0,
    Rgb332 = SDL_PixelFormat::RGB332.0,
    Xrgb4444 = SDL_PixelFormat::XRGB4444.0,
    Xbgr4444 = SDL_PixelFormat::XBGR4444.0,
    Xrgb1555 = SDL_PixelFormat::XRGB1555.0,
    Xbgr1555 = SDL_PixelFormat::XBGR1555.0,
    Argb4444 = SDL_PixelFormat::ARGB4444.0,
    Rgba4444 = SDL_PixelFormat::RGBA4444.0,
    Abgr4444 = SDL_PixelFormat::ABGR4444.0,
    Bgra4444 = SDL_PixelFormat::BGRA4444.0,
    Argb1555 = SDL_PixelFormat::ARGB1555.0,
    Rgba5551 = SDL_PixelFormat::RGBA5551.0,
    Abgr1555 = SDL_PixelFormat::ABGR1555.0,
    Bgra5551 = SDL_PixelFormat::BGRA5551.0,
    Rgb565 = SDL_PixelFormat::RGB565.0,
    Bgr565 = SDL_PixelFormat::BGR565.0,
    Rgb24 = SDL_PixelFormat::RGB24.0,
    Bgr24 = SDL_PixelFormat::BGR24.0,
    Xrgb8888 = SDL_PixelFormat::XRGB8888.0,
    Rgbx8888 = SDL_PixelFormat::RGBX8888.0,
    Xbgr8888 = SDL_PixelFormat::XBGR8888.0,
    Bgrx8888 = SDL_PixelFormat::BGRX8888.0,
    Argb8888 = SDL_PixelFormat::ARGB8888.0,
    Rgba8888 = SDL_PixelFormat::RGBA8888.0,
    Abgr8888 = SDL_PixelFormat::ABGR8888.0,
    Bgra8888 = SDL_PixelFormat::BGRA8888.0,
    Xrgb2101010 = SDL_PixelFormat::XRGB2101010.0,
    Xbgr2101010 = SDL_PixelFormat::XBGR2101010.0,
    Argb2101010 = SDL_PixelFormat::ARGB2101010.0,
    Abgr2101010 = SDL_PixelFormat::ABGR2101010.0,
    Rgb48 = SDL_PixelFormat::RGB48.0,
    Bgr48 = SDL_PixelFormat::BGR48.0,
    Rgba64 = SDL_PixelFormat::RGBA64.0,
    Argb64 = SDL_PixelFormat::ARGB64.0,
    Bgra64 = SDL_PixelFormat::BGRA64.0,
    Abgr64 = SDL_PixelFormat::ABGR64.0,
    Rgb48Float = SDL_PixelFormat::RGB48_FLOAT.0,
    Bgr48Float = SDL_PixelFormat::BGR48_FLOAT.0,
    Rgba64Float = SDL_PixelFormat::RGBA64_FLOAT.0,
    Argb64Float = SDL_PixelFormat::ARGB64_FLOAT.0,
    Bgra64Float = SDL_PixelFormat::BGRA64_FLOAT.0,
    Abgr64Float = SDL_PixelFormat::ABGR64_FLOAT.0,
    Rgb96Float = SDL_PixelFormat::RGB96_FLOAT.0,
    Bgr96Float = SDL_PixelFormat::BGR96_FLOAT.0,
    Rgba128Float = SDL_PixelFormat::RGBA128_FLOAT.0,
    Argb128Float = SDL_PixelFormat::ARGB128_FLOAT.0,
    Bgra128Float = SDL_PixelFormat::BGRA128_FLOAT.0,
    Abgr128Float = SDL_PixelFormat::ABGR128_FLOAT.0,
    Yv12 = SDL_PixelFormat::YV12.0,
    Iyuv = SDL_PixelFormat::IYUV.0,
    Yuy2 = SDL_PixelFormat::YUY2.0,
    Uyvy = SDL_PixelFormat::UYVY.0,
    Yvyu = SDL_PixelFormat::YVYU.0,
    Nv12 = SDL_PixelFormat::NV12.0,
    Nv21 = SDL_PixelFormat::NV21.0,
    P010 = SDL_PixelFormat::P010.0,
    ExternalOes = SDL_PixelFormat::EXTERNAL_OES.0,
    Mjpg = SDL_PixelFormat::MJPG.0,
}

impl PixelFormat {
    /// Alias for the appropriate RGBA 8888 encoding of color data for the
    /// current platform's endianness.
    pub const RGBA32: Self = Self::from_sdl(SDL_PixelFormat::RGBA8888);
    /// Alias for the appropriate ARGB 8888 encoding of color data for the
    /// current platform's endianness.
    pub const ARGB32: Self = Self::from_sdl(SDL_PixelFormat::ARGB8888);
    /// Alias for the appropriate BGRA 8888 encoding of color data for the
    /// current platform's endianness.
    pub const BGRA32: Self = Self::from_sdl(SDL_PixelFormat::BGRA8888);
    /// Alias for the appropriate ABGR 8888 encoding of color data for the
    /// current platform's endianness.
    pub const ABGR32: Self = Self::from_sdl(SDL_PixelFormat::ABGR8888);
    /// Alias for the appropriate RGBX 8888 encoding of color data for the
    /// current platform's endianness.
    pub const RGBX32: Self = Self::from_sdl(SDL_PixelFormat::RGBX8888);
    /// Alias for the appropriate XRGB 8888 encoding of color data for the
    /// current platform's endianness.
    pub const XRGB32: Self = Self::from_sdl(SDL_PixelFormat::XRGB8888);
    /// Alias for the appropriate BGRX 8888 encoding of color data for the
    /// current platform's endianness.
    pub const BGRX32: Self = Self::from_sdl(SDL_PixelFormat::BGRX8888);
    /// Alias for the appropriate XBGR 8888 encoding of color data for the
    /// current platform's endianness.
    pub const XBGR32: Self = Self::from_sdl(SDL_PixelFormat::XBGR8888);

    /// Convert a mask to an enumerated pixel format.
    ///
    /// Returns [`None`] if there isn't a match.
    #[doc(alias = "SDL_GetPixelFormatForMasks")]
    pub fn from_mask(mask: PixelFormatMask) -> Option<Self> {
        let fmt = SDL_GetPixelFormatForMasks(mask.bpp, mask.r, mask.g, mask.b, mask.a);

        if fmt == SDL_PixelFormat::UNKNOWN {
            None
        } else {
            Some(fmt.into())
        }
    }

    /// Extract [`PixelFormatMask`] data from this pixel format.
    ///
    /// Returns [`Err`] if the data cannot be extracted.
    #[doc(alias = "SDL_GetMasksForPixelFormat")]
    pub fn mask(self) -> Result<PixelFormatMask> {
        let mut mask = MaybeUninit::<PixelFormatMask>::uninit();
        let ptr = mask.as_mut_ptr();
        unsafe {
            if SDL_GetMasksForPixelFormat(
                self.into(),
                &raw mut (*ptr).bpp,
                &raw mut (*ptr).r,
                &raw mut (*ptr).g,
                &raw mut (*ptr).b,
                &raw mut (*ptr).a,
            ) {
                Ok(mask.assume_init())
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get the human-readable SDL enum variant name of this pixel format.
    ///
    /// For example, this returns "SDL_PIXELFORMAT_RGBA8888" for [`Self::Rgba8888`].
    /// The SDL function is documented to return "SDL_PIXELFORMAT_UNKNOWN"
    /// in case of an unknown variant, but that shouldn't happen.
    #[doc(alias = "SDL_GetPixelFormatName")]
    pub fn name(self) -> &'static str {
        unsafe {
            // SAFETY: The function below returns an UTF-8 string pointer,
            // no matter what.
            let ptr = SDL_GetPixelFormatName(self.into());
            c_ptr_to_str(ptr)
        }
    }
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl_enum_transmute!(SDL_PixelFormat, PixelFormat);
