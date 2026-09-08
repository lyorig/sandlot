//! Various enums related to pixels and/or graphics.

use sdl3_sys::{
    blendmode::SDL_BlendMode,
    pixels::{SDL_Colorspace, SDL_PixelFormat},
    surface::SDL_ScaleMode,
};

use crate::impl_enum_transmute;

/// A set of blend modes used in drawing operations.
///
/// # Remarks
///
/// These predefined blend modes are supported everywhere.
///
/// Additional values may be obtained from `SDL_ComposeCustomBlendMode`.
#[repr(u32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_BlendMode")]
pub enum BlendMode {
    /// No blending: `dstRGBA = srcRGBA`.
    None = SDL_BlendMode::NONE.0,
    /// Alpha blending:
    /// `dstRGB = (srcRGB * srcA) + (dstRGB * (1-srcA))`,
    /// `dstA = srcA + (dstA * (1-srcA))`.
    Blend = SDL_BlendMode::BLEND.0,
    /// Pre-multiplied alpha blending:
    /// `dstRGBA = srcRGBA + (dstRGBA * (1-srcA))`.
    BlendPremultiplied = SDL_BlendMode::BLEND_PREMULTIPLIED.0,
    /// Additive blending:
    /// `dstRGB = (srcRGB * srcA) + dstRGB`, `dstA = dstA`.
    Add = SDL_BlendMode::ADD.0,
    /// Pre-multiplied additive blending:
    /// `dstRGB = srcRGB + dstRGB`, `dstA = dstA`.
    AddPremultiplied = SDL_BlendMode::ADD_PREMULTIPLIED.0,
    /// Color modulate: `dstRGB = srcRGB * dstRGB`, `dstA = dstA`.
    Mod = SDL_BlendMode::MOD.0,
    /// Color multiply:
    /// `dstRGB = (srcRGB * dstRGB) + (dstRGB * (1-srcA))`, `dstA = dstA`.
    Mul = SDL_BlendMode::MUL.0,
}

impl_enum_transmute!(SDL_BlendMode, BlendMode);

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
}

impl_enum_transmute!(SDL_PixelFormat, PixelFormat);
