use std::{mem::size_of, ptr};

use rustest::test;
use sandlot::color::{OpacityBounds, RgbF32, RgbU8, RgbaF32, RgbaU8};
use sdl3_sys::pixels::SDL_Color;

#[test]
fn color_rgb_new() {
    let c = RgbU8::new(10, 20, 30);
    assert_eq!(c.r, 10);
    assert_eq!(c.g, 20);
    assert_eq!(c.b, 30);
}

#[test]
fn color_rgba_new() {
    let c = RgbaU8::new(10, 20, 30, 40);
    assert_eq!(c.rgb.r, 10);
    assert_eq!(c.rgb.g, 20);
    assert_eq!(c.rgb.b, 30);
    assert_eq!(c.a, 40);
}

#[test]
fn color_rgba_rgb() {
    let c = RgbaU8::rgb(15, 25, 35);
    assert_eq!(c.rgb.r, 15);
    assert_eq!(c.rgb.g, 25);
    assert_eq!(c.rgb.b, 35);
    assert_eq!(c.a, u8::MAX);
}

#[test]
fn color_rgb_to_rgba() {
    let rgb = RgbU8::RED;
    let rgba: RgbaU8 = rgb.to_rgba();
    assert_eq!(rgba, RgbaU8::RED);

    let rgba2 = RgbU8::new(1, 2, 3).with_alpha(128);
    assert_eq!(rgba2, RgbaU8::new(1, 2, 3, 128));
}

#[test]
fn color_rgb_u8_to_f32() {
    // Boundary: black
    let f: RgbF32 = RgbU8::BLACK.into();
    assert!((f.r - 0.0).abs() < f32::EPSILON);
    assert!((f.g - 0.0).abs() < f32::EPSILON);
    assert!((f.b - 0.0).abs() < f32::EPSILON);

    // Boundary: white
    let f: RgbF32 = RgbU8::WHITE.into();
    assert!((f.r - 1.0).abs() < f32::EPSILON);
    assert!((f.g - 1.0).abs() < f32::EPSILON);
    assert!((f.b - 1.0).abs() < f32::EPSILON);

    // Mid values
    let f: RgbF32 = RgbU8::new(128, 64, 192).into();
    assert!((f.r - 128.0 / 255.0).abs() < 0.001);
    assert!((f.g - 64.0 / 255.0).abs() < 0.001);
    assert!((f.b - 192.0 / 255.0).abs() < 0.001);
}

#[test]
fn color_rgb_f32_to_u8() {
    // Boundary: black
    let u: RgbU8 = RgbF32::BLACK.into();
    assert_eq!(u, RgbU8::BLACK);

    // Boundary: white
    let u: RgbU8 = RgbF32::WHITE.into();
    assert_eq!(u, RgbU8::WHITE);

    // Truncation: f32-to-u8 cast truncates, not rounds
    let u: RgbU8 = RgbF32::new(0.5, 0.25, 0.75).into();
    assert_eq!(u, RgbU8::new(127, 63, 191));
}

#[test]
fn color_rgb_u8_f32_roundtrip() {
    for val in [0u8, 1, 127, 128, 254, 255] {
        let orig = RgbU8::new(val, val, val);
        let f: RgbF32 = orig.into();
        let back: RgbU8 = f.into();
        assert_eq!(back, orig, "round-trip failed for u8 value {val}");
    }
}

#[test]
fn color_rgba_u8_to_f32() {
    // Boundary: black (with full alpha)
    let f: RgbaF32 = RgbaU8::BLACK.into();
    assert!((f.rgb.r - 0.0).abs() < f32::EPSILON);
    assert!((f.rgb.g - 0.0).abs() < f32::EPSILON);
    assert!((f.rgb.b - 0.0).abs() < f32::EPSILON);
    assert!((f.a - 1.0).abs() < f32::EPSILON);

    // Boundary: transparent
    let f: RgbaF32 = RgbaU8::TRANSPARENT.into();
    assert!((f.a - 0.0).abs() < f32::EPSILON);

    // Boundary: white
    let f: RgbaF32 = RgbaU8::WHITE.into();
    assert!((f.rgb.r - 1.0).abs() < f32::EPSILON);
    assert!((f.rgb.g - 1.0).abs() < f32::EPSILON);
    assert!((f.rgb.b - 1.0).abs() < f32::EPSILON);
    assert!((f.a - 1.0).abs() < f32::EPSILON);

    // Mid values
    let f: RgbaF32 = RgbaU8::new(128, 64, 192, 32).into();
    assert!((f.rgb.r - 128.0 / 255.0).abs() < 0.001);
    assert!((f.rgb.g - 64.0 / 255.0).abs() < 0.001);
    assert!((f.rgb.b - 192.0 / 255.0).abs() < 0.001);
    assert!((f.a - 32.0 / 255.0).abs() < 0.001);
}

#[test]
fn color_rgba_f32_to_u8() {
    // Boundary: black
    let u: RgbaU8 = RgbaF32::BLACK.into();
    assert_eq!(u, RgbaU8::BLACK);

    // Boundary: white
    let u: RgbaU8 = RgbaF32::WHITE.into();
    assert_eq!(u, RgbaU8::WHITE);

    // Truncation: f32-to-u8 cast truncates, not rounds
    let u: RgbaU8 = RgbaF32::new(0.5, 0.25, 0.75, 0.5).into();
    assert_eq!(u, RgbaU8::new(127, 63, 191, 127));
}

#[test]
fn color_rgba_u8_f32_roundtrip() {
    for val in [0u8, 1, 127, 128, 254, 255] {
        let orig = RgbaU8::new(val, val, val, val);
        let f: RgbaF32 = orig.into();
        let back: RgbaU8 = f.into();
        assert_eq!(back, orig, "round-trip failed for u8 value {val}");
    }
}

#[test]
fn color_rgb_into_rgba() {
    let rgb = RgbU8::new(1, 2, 3);
    let rgba: RgbaU8 = rgb.into();
    assert_eq!(rgba, RgbaU8::new(1, 2, 3, u8::MAX_OPACITY));
}

#[test]
fn color_rgba_u8_hex() {
    // rgb_hex: 0x00FF00 = green (R=0x00, G=0xFF, B=0x00)
    assert_eq!(
        RgbaU8::rgb_hex(0x00FF00),
        RgbaU8::new(0x00, 0xFF, 0x00, u8::MAX_OPACITY)
    );

    // rgb_hex: arbitrary color
    assert_eq!(
        RgbaU8::rgb_hex(0x123456),
        RgbaU8::new(0x12, 0x34, 0x56, u8::MAX_OPACITY)
    );

    // rgb_hex: 0x000000 = black with full alpha
    assert_eq!(RgbaU8::rgb_hex(0x000000), RgbaU8::BLACK);

    // rgba_hex: 0xFF0000FF = red, full alpha
    assert_eq!(RgbaU8::rgba_hex(0xFF0000FF), RgbaU8::RED);

    // rgba_hex: 0x00000000 = transparent black
    assert_eq!(RgbaU8::rgba_hex(0x00000000), RgbaU8::TRANSPARENT);

    // rgba_hex: arbitrary
    assert_eq!(
        RgbaU8::rgba_hex(0x12345678),
        RgbaU8::new(0x12, 0x34, 0x56, 0x78)
    );
}

#[test]
fn color_rgba_u8_to_sdl_color() {
    let rgba = RgbaU8::new(10, 20, 30, 40);
    let sdl: SDL_Color = rgba.into();

    assert_eq!(sdl.r, 10);
    assert_eq!(sdl.g, 20);
    assert_eq!(sdl.b, 30);
    assert_eq!(sdl.a, 40);
}

#[test]
fn color_rgba_u8_to_sdl_color_transparent() {
    let rgba = RgbaU8::TRANSPARENT;
    let sdl: SDL_Color = rgba.into();

    assert_eq!(sdl.r, 0);
    assert_eq!(sdl.g, 0);
    assert_eq!(sdl.b, 0);
    assert_eq!(sdl.a, 0);
}

#[test]
fn color_rgba_layout_matches_sdl_color() {
    assert_eq!(size_of::<RgbaU8>(), size_of::<SDL_Color>());
    assert_eq!(size_of::<RgbU8>(), 3);
    assert_eq!(size_of::<SDL_Color>(), 4);

    // Byte-for-byte equivalence confirms the #[repr(C)] layout
    // and field ordering match SDL_Color.
    let rgba = RgbaU8::new(1, 2, 3, 4);
    let sdl: SDL_Color = rgba.into();

    let rgba_bytes = unsafe { ptr::from_ref(&rgba).cast::<[u8; 4]>().as_ref_unchecked() };
    let sdl_bytes = unsafe { ptr::from_ref(&sdl).cast::<[u8; 4]>().as_ref_unchecked() };
    assert_eq!(rgba_bytes, sdl_bytes);
}
