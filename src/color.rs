use std::mem::MaybeUninit;

use sdl3_sys::pixels::*;

use crate::util::opt2ptr;

/// "Sub-struct" of [`Rgba`], because some SDL functions only use
/// the RGB components and don't require the alpha.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

pub type RgbU8 = Rgb<u8>;
pub type RgbF32 = Rgb<f32>;

impl<T> Rgb<T> {
    pub const fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

impl<T: OpacityBounds> Rgb<T> {
    pub const BLACK: Self = Self::new(T::MIN_OPACITY, T::MIN_OPACITY, T::MIN_OPACITY);
    pub const RED: Self = Self::new(T::MAX_OPACITY, T::MIN_OPACITY, T::MIN_OPACITY);
    pub const GREEN: Self = Self::new(T::MIN_OPACITY, T::MAX_OPACITY, T::MIN_OPACITY);
    pub const BLUE: Self = Self::new(T::MIN_OPACITY, T::MIN_OPACITY, T::MAX_OPACITY);
    pub const CYAN: Self = Self::new(T::MIN_OPACITY, T::MAX_OPACITY, T::MAX_OPACITY);
    pub const WHITE: Self = Self::new(T::MAX_OPACITY, T::MAX_OPACITY, T::MAX_OPACITY);

    pub const fn to_rgba(&self) -> Rgba<T> {
        Rgba {
            rgb: *self,
            a: T::MAX_OPACITY,
        }
    }

    pub const fn with_alpha(&self, a: T) -> Rgba<T> {
        Rgba { rgb: *self, a }
    }
}

impl RgbU8 {
    /// Get RGB values from a pixel in the specified format.
    ///
    /// # Remarks
    ///
    /// This function uses the entire 8-bit [0..255] range when converting
    /// color components from pixel formats with less than 8-bits per RGB
    /// component (e.g., a completely white pixel in 16-bit RGB565 format
    /// would return [0xff, 0xff, 0xff] not [0xf8, 0xfc, 0xf8]).
    #[doc(alias = "SDL_GetRGB")]
    pub fn from_pixel(pixel: u32, fmt: &SDL_PixelFormatDetails, pal: Option<&SDL_Palette>) -> Self {
        let mut ret = MaybeUninit::<Self>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetRGB(
                pixel,
                std::ptr::from_ref(fmt),
                opt2ptr(pal),
                &raw mut (*ptr).r,
                &raw mut (*ptr).g,
                &raw mut (*ptr).b,
            );

            ret.assume_init()
        }
    }

    /// Map an RGB triple to an opaque pixel value for a given pixel format.
    ///
    /// # Remarks
    ///
    /// This function maps the RGB color value to the specified pixel format
    /// and returns the pixel value best approximating the given RGB color
    /// value for the given pixel format.
    ///
    /// If the format has a palette (8-bit) the index of the closest matching
    /// color in the palette will be returned.
    ///
    /// If the specified pixel format has an alpha component it will be
    /// returned as all 1 bits (fully opaque).
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the
    /// unused upper bits of the return value can safely be ignored.
    #[doc(alias = "SDL_MapRGB")]
    pub fn map(self, fmt: &SDL_PixelFormatDetails, pal: Option<&SDL_Palette>) -> u32 {
        unsafe {
            SDL_MapRGB(
                std::ptr::from_ref(fmt),
                opt2ptr(pal),
                self.r,
                self.g,
                self.b,
            )
        }
    }
}

impl From<RgbU8> for RgbF32 {
    fn from(value: RgbU8) -> Self {
        Self::new(
            f32::from(value.r) / 255.0,
            f32::from(value.g) / 255.0,
            f32::from(value.b) / 255.0,
        )
    }
}

impl From<RgbF32> for RgbU8 {
    fn from(value: RgbF32) -> Self {
        Self::new(
            (value.r * 255.0) as _,
            (value.g * 255.0) as _,
            (value.b * 255.0) as _,
        )
    }
}

impl From<RgbaU8> for RgbaF32 {
    fn from(value: RgbaU8) -> Self {
        Self::new(
            f32::from(value.rgb.r) / 255.0,
            f32::from(value.rgb.g) / 255.0,
            f32::from(value.rgb.b) / 255.0,
            f32::from(value.a) / 255.0,
        )
    }
}

impl From<RgbaF32> for RgbaU8 {
    fn from(value: RgbaF32) -> Self {
        Self::new(
            (value.rgb.r * 255.0) as _,
            (value.rgb.g * 255.0) as _,
            (value.rgb.b * 255.0) as _,
            (value.a * 255.0) as _,
        )
    }
}

/// Wrapper around `SDL_(F)Color`. Can be transmuted.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba<T> {
    pub rgb: Rgb<T>,
    pub a: T,
}

pub type RgbaU8 = Rgba<u8>;
pub type RgbaF32 = Rgba<f32>;

pub trait OpacityBounds: Copy {
    const MIN_OPACITY: Self;
    const MAX_OPACITY: Self;
}

impl OpacityBounds for u8 {
    const MIN_OPACITY: Self = 0;
    const MAX_OPACITY: Self = Self::MAX;
}

impl OpacityBounds for f32 {
    const MIN_OPACITY: Self = 0.0;
    const MAX_OPACITY: Self = 1.0;
}

impl<T: OpacityBounds> Rgba<T> {
    pub const fn rgb(r: T, g: T, b: T) -> Self {
        Self::new(r, g, b, T::MAX_OPACITY)
    }

    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self {
            rgb: Rgb::new(r, g, b),
            a,
        }
    }

    pub const BLACK: Self = Rgb::BLACK.to_rgba();
    pub const RED: Self = Rgb::RED.to_rgba();
    pub const GREEN: Self = Rgb::GREEN.to_rgba();
    pub const BLUE: Self = Rgb::BLUE.to_rgba();
    pub const CYAN: Self = Rgb::CYAN.to_rgba();
    pub const WHITE: Self = Rgb::WHITE.to_rgba();

    pub const TRANSPARENT: Self = Rgb::BLACK.with_alpha(T::MIN_OPACITY);
}

impl RgbaU8 {
    /// Create an [`RgbaU8`] from a hex (0xRRGGBB) representation.
    /// This forwards the extracted red, green, and blue components
    /// to [`RgbaU8::rgb`].
    pub const fn rgb_hex(val: u32) -> Self {
        Self::rgb((val >> 16) as u8, (val >> 8) as u8, val as u8)
    }

    /// Create an [`RgbaU8`] from a hex (0xRRGGBBAA) representation.
    /// This forwards the extracted red, green, blue, and alpha components
    /// to [`RgbaU8::new`].
    pub const fn rgba_hex(val: u32) -> Self {
        Self::new(
            (val >> 24) as u8,
            (val >> 16) as u8,
            (val >> 8) as u8,
            val as u8,
        )
    }

    /// Get RGBA values from a pixel in the specified format.
    ///
    /// # Remarks
    ///
    /// This function uses the entire 8-bit [0..255] range when converting
    /// color components from pixel formats with less than 8-bits per RGB
    /// component (e.g., a completely white pixel in 16-bit RGB565 format
    /// would return [0xff, 0xff, 0xff] not [0xf8, 0xfc, 0xf8]).
    ///
    /// If the format has no alpha component, the alpha will be returned as
    /// 0xff (100% opaque).
    #[doc(alias = "SDL_GetRGBA")]
    pub fn from_pixel(pixel: u32, fmt: &SDL_PixelFormatDetails, pal: Option<&SDL_Palette>) -> Self {
        let mut ret = MaybeUninit::<Self>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetRGBA(
                pixel,
                std::ptr::from_ref(fmt),
                opt2ptr(pal),
                &raw mut (*ptr).rgb.r,
                &raw mut (*ptr).rgb.g,
                &raw mut (*ptr).rgb.b,
                &raw mut (*ptr).a,
            );

            ret.assume_init()
        }
    }

    /// Map an RGBA quadruple to a pixel value for a given pixel format.
    ///
    /// # Remarks
    ///
    /// This function maps the RGBA color value to the specified pixel format
    /// and returns the pixel value best approximating the given RGBA color
    /// value for the given pixel format.
    ///
    /// If the specified pixel format has no alpha component the alpha value
    /// will be ignored (as it will be in formats with a palette).
    ///
    /// If the format has a palette (8-bit) the index of the closest matching
    /// color in the palette will be returned.
    ///
    /// If the pixel format bpp (color depth) is less than 32-bpp then the
    /// unused upper bits of the return value can safely be ignored.
    #[doc(alias = "SDL_MapRGBA")]
    pub fn map(self, fmt: &SDL_PixelFormatDetails, pal: Option<&SDL_Palette>) -> u32 {
        unsafe {
            SDL_MapRGBA(
                std::ptr::from_ref(fmt),
                opt2ptr(pal),
                self.rgb.r,
                self.rgb.g,
                self.rgb.b,
                self.a,
            )
        }
    }
}

impl<T: OpacityBounds> From<Rgb<T>> for Rgba<T> {
    fn from(value: Rgb<T>) -> Self {
        value.to_rgba()
    }
}

impl From<RgbaU8> for SDL_Color {
    fn from(value: RgbaU8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<SDL_Color> for RgbaU8 {
    fn from(value: SDL_Color) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<RgbaF32> for SDL_FColor {
    fn from(value: RgbaF32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<SDL_FColor> for RgbaF32 {
    fn from(value: SDL_FColor) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
