use crate::{
    color::{RgbF32, RgbU8, RgbaF32, RgbaU8},
    pixels,
};

pub trait BlendMode: Copy {
    fn blend_mode(self) -> pixels::BlendMode;
    fn set_blend_mode(self, bm: pixels::BlendMode);

    /// Exchanges the blend mode, returning the old one.
    fn xchg_blend_mode(self, bm: pixels::BlendMode) -> pixels::BlendMode {
        let old = self.blend_mode();
        self.set_blend_mode(bm);
        old
    }
}

pub trait ColorModU8: Copy {
    fn rgb_mod_u8(self) -> RgbU8;
    fn alpha_mod_u8(self) -> u8;
    fn color_mod_u8(self) -> RgbaU8 {
        self.rgb_mod_u8().with_alpha(self.alpha_mod_u8())
    }

    fn set_rgb_mod_u8(self, rm: RgbU8);
    fn set_alpha_mod_u8(self, am: u8);
    fn set_color_mod_u8(self, col: RgbaU8) {
        self.set_rgb_mod_u8(col.rgb);
        self.set_alpha_mod_u8(col.a);
    }

    /// Exchanges the RGB mod, returning the old one.
    fn xchg_rgb_mod_u8(self, rm: RgbU8) -> RgbU8 {
        let old = self.rgb_mod_u8();
        self.set_rgb_mod_u8(rm);
        old
    }

    /// Exchanges the alpha mod, returning the old one.
    fn xchg_alpha_mod_u8(self, am: u8) -> u8 {
        let old = self.alpha_mod_u8();
        self.set_alpha_mod_u8(am);
        old
    }

    /// Exchanges the color mod, returning the old one.
    fn xchg_color_mod_u8(self, col: RgbaU8) -> RgbaU8 {
        let old = self.color_mod_u8();
        self.set_color_mod_u8(col);
        old
    }
}

pub trait ColorModF32: Copy {
    fn rgb_mod_f32(self) -> RgbF32;
    fn alpha_mod_f32(self) -> f32;
    fn color_mod_f32(self) -> RgbaF32 {
        self.rgb_mod_f32().with_alpha(self.alpha_mod_f32())
    }

    fn set_rgb_mod_f32(self, rm: RgbF32);
    fn set_alpha_mod_f32(self, am: f32);
    fn set_color_mod_f32(self, col: RgbaF32) {
        self.set_rgb_mod_f32(col.rgb);
        self.set_alpha_mod_f32(col.a);
    }

    /// Exchanges the RGB mod, returning the old one.
    fn xchg_rgb_mod_f32(self, rm: RgbF32) -> RgbF32 {
        let old = self.rgb_mod_f32();
        self.set_rgb_mod_f32(rm);
        old
    }

    /// Exchanges the alpha mod, returning the old one.
    fn xchg_alpha_mod_f32(self, am: f32) -> f32 {
        let old = self.alpha_mod_f32();
        self.set_alpha_mod_f32(am);
        old
    }

    /// Exchanges the color mod, returning the old one.
    fn xchg_color_mod_f32(self, col: RgbaF32) -> RgbaF32 {
        let old = self.color_mod_f32();
        self.set_color_mod_f32(col);
        old
    }
}
