use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use sdl3_ttf_sys::ttf::*;

use crate::{
    Result,
    color::RgbaU8,
    surface::Surface,
    ttf::{Context, RtStr},
    util::c_ptr_to_str,
};

crate::resource::resource_new_tied!(
    /// The internal structure containing font information.
    /// Opaque data.
    TTF_Font, Font, TTF_CloseFont, Context
);

impl Clone for Font<'_> {
    /// Create a copy of an existing font.
    ///
    /// # Remarks
    ///
    /// The copy will be distinct from the original, but will share the font
    /// file and have the same size and style as the original.
    #[doc(alias = "TTF_CopyFont")]
    fn clone(&self) -> Self {
        let ptr = unsafe { TTF_CopyFont(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(ptr) };
        let inner = FontHandle { handle };

        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl FontHandle {
    /// Render a single UNICODE codepoint at high quality to a new ARGB
    /// surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, using alpha
    /// blending to dither the font with the given color.
    ///
    /// The glyph is rendered without any padding or centering in the X
    /// direction, and aligned normally in the Y direction.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_glyph_solid`],
    /// [`FontHandle::render_glyph_shaded`], and
    /// [`FontHandle::render_glyph_lcd`].
    #[doc(alias = "TTF_RenderGlyph_Blended")]
    pub fn render_glyph_blended(&self, ch: char, color: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderGlyph_Blended(self.handle.as_ptr(), ch.into(), color.into())
        })
    }

    /// Render a single UNICODE codepoint at LCD subpixel quality to a new
    /// ARGB surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, and render
    /// alpha-blended text using FreeType's LCD subpixel rendering.
    ///
    /// The glyph is rendered without any padding or centering in the X
    /// direction, and aligned normally in the Y direction.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_glyph_solid`],
    /// [`FontHandle::render_glyph_shaded`], and
    /// [`FontHandle::render_glyph_blended`].
    #[doc(alias = "TTF_RenderGlyph_LCD")]
    pub fn render_glyph_lcd(&self, ch: char, fg: RgbaU8, bg: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderGlyph_LCD(self.handle.as_ptr(), ch.into(), fg.into(), bg.into())
        })
    }

    /// Render a single UNICODE codepoint at high quality to a new 8-bit
    /// surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the specified background color, while other
    /// pixels have varying degrees of the foreground color.
    ///
    /// The glyph is rendered without any padding or centering in the X
    /// direction, and aligned normally in the Y direction.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_glyph_solid`],
    /// [`FontHandle::render_glyph_blended`], and
    /// [`FontHandle::render_glyph_lcd`].
    #[doc(alias = "TTF_RenderGlyph_Shaded")]
    pub fn render_glyph_shaded(&self, ch: char, fg: RgbaU8, bg: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderGlyph_Shaded(self.handle.as_ptr(), ch.into(), fg.into(), bg.into())
        })
    }

    /// Render a single 32-bit glyph at fast quality to a new 8-bit surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the colorkey, giving a transparent
    /// background. The 1 pixel will be set to the text color.
    ///
    /// The glyph is rendered without any padding or centering in the X
    /// direction, and aligned normally in the Y direction.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_glyph_shaded`],
    /// [`FontHandle::render_glyph_blended`], and
    /// [`FontHandle::render_glyph_lcd`].
    #[doc(alias = "TTF_RenderGlyph_Solid")]
    pub fn render_glyph_solid(&self, ch: char, color: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderGlyph_Solid(self.handle.as_ptr(), ch.into(), color.into())
        })
    }

    /// Render UTF-8 text at high quality to a new ARGB surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, using alpha
    /// blending to dither the font with the given color.
    ///
    /// This will not word-wrap the string; you'll get a surface with a single
    /// line of text, as long as the string requires. You can use
    /// [`FontHandle::render_text_blended_wrapped`] instead if you need to
    /// wrap the output to multiple lines.
    ///
    /// This will not wrap on newline characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid`],
    /// [`FontHandle::render_text_shaded`], and
    /// [`FontHandle::render_text_lcd`].
    #[doc(alias = "TTF_RenderText_Blended")]
    pub fn render_text_blended(&self, text: RtStr, color: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Blended(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                color.into(),
            )
        })
    }

    /// Render UTF-8 text at LCD subpixel quality to a new ARGB surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, and render
    /// alpha-blended text using FreeType's LCD subpixel rendering.
    ///
    /// This will not word-wrap the string; you'll get a surface with a single
    /// line of text, as long as the string requires. You can use
    /// [`FontHandle::render_text_lcd_wrapped`] instead if you need to wrap
    /// the output to multiple lines.
    ///
    /// This will not wrap on newline characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid`],
    /// [`FontHandle::render_text_shaded`], and
    /// [`FontHandle::render_text_blended`].
    #[doc(alias = "TTF_RenderText_LCD")]
    pub fn render_text_lcd(&self, text: RtStr, fg: RgbaU8, bg: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_LCD(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                fg.into(),
                bg.into(),
            )
        })
    }

    /// Render UTF-8 text at high quality to a new 8-bit surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the specified background color, while other
    /// pixels have varying degrees of the foreground color.
    ///
    /// This will not word-wrap the string; you'll get a surface with a single
    /// line of text, as long as the string requires. You can use
    /// [`FontHandle::render_text_shaded_wrapped`] instead if you need to
    /// wrap the output to multiple lines.
    ///
    /// This will not wrap on newline characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid`],
    /// [`FontHandle::render_text_blended`], and
    /// [`FontHandle::render_text_lcd`].
    #[doc(alias = "TTF_RenderText_Shaded")]
    pub fn render_text_shaded(&self, text: RtStr, fg: RgbaU8, bg: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Shaded(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                fg.into(),
                bg.into(),
            )
        })
    }

    /// Render UTF-8 text at fast quality to a new 8-bit surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the colorkey, giving a transparent
    /// background. The 1 pixel will be set to the text color.
    ///
    /// This will not word-wrap the string; you'll get a surface with a single
    /// line of text, as long as the string requires. You can use
    /// [`FontHandle::render_text_solid_wrapped`] instead if you need to wrap
    /// the output to multiple lines.
    ///
    /// This will not wrap on newline characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_shaded`],
    /// [`FontHandle::render_text_blended`], and
    /// [`FontHandle::render_text_lcd`].
    #[doc(alias = "TTF_RenderText_Solid")]
    pub fn render_text_solid(&self, text: RtStr, color: RgbaU8) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Solid(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                color.into(),
            )
        })
    }

    /// Render word-wrapped UTF-8 text at high quality to a new ARGB surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, using alpha
    /// blending to dither the font with the given color.
    ///
    /// Text is wrapped to multiple lines on line endings and on word
    /// boundaries if it extends beyond `wrap_length` in pixels. If
    /// `wrap_length` is 0, this function will only wrap on newline
    /// characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid_wrapped`],
    /// [`FontHandle::render_text_shaded_wrapped`], and
    /// [`FontHandle::render_text_lcd_wrapped`].
    #[doc(alias = "TTF_RenderText_Blended_Wrapped")]
    pub fn render_text_blended_wrapped(
        &self,
        text: RtStr,
        color: RgbaU8,
        wrap_length: i32,
    ) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Blended_Wrapped(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                color.into(),
                wrap_length,
            )
        })
    }

    /// Render word-wrapped UTF-8 text at LCD subpixel quality to a new ARGB
    /// surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 32-bit, ARGB surface, and render
    /// alpha-blended text using FreeType's LCD subpixel rendering.
    ///
    /// Text is wrapped to multiple lines on line endings and on word
    /// boundaries if it extends beyond `wrap_length` in pixels. If
    /// `wrap_length` is 0, this function will only wrap on newline
    /// characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid_wrapped`],
    /// [`FontHandle::render_text_shaded_wrapped`], and
    /// [`FontHandle::render_text_blended_wrapped`].
    #[doc(alias = "TTF_RenderText_LCD_Wrapped")]
    pub fn render_text_lcd_wrapped(
        &self,
        text: RtStr,
        fg: RgbaU8,
        bg: RgbaU8,
        wrap_length: i32,
    ) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_LCD_Wrapped(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                fg.into(),
                bg.into(),
                wrap_length,
            )
        })
    }

    /// Render word-wrapped UTF-8 text at high quality to a new 8-bit
    /// surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the specified background color, while other
    /// pixels have varying degrees of the foreground color.
    ///
    /// Text is wrapped to multiple lines on line endings and on word
    /// boundaries if it extends beyond `wrap_length` in pixels. If
    /// `wrap_length` is 0, this function will only wrap on newline
    /// characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_solid_wrapped`],
    /// [`FontHandle::render_text_blended_wrapped`], and
    /// [`FontHandle::render_text_lcd_wrapped`].
    #[doc(alias = "TTF_RenderText_Shaded_Wrapped")]
    pub fn render_text_shaded_wrapped(
        &self,
        text: RtStr,
        fg: RgbaU8,
        bg: RgbaU8,
        wrap_length: i32,
    ) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Shaded_Wrapped(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                fg.into(),
                bg.into(),
                wrap_length,
            )
        })
    }

    /// Render word-wrapped UTF-8 text at fast quality to a new 8-bit
    /// surface.
    ///
    /// # Remarks
    ///
    /// This function will allocate a new 8-bit, palettized surface. The
    /// surface's 0 pixel will be the colorkey, giving a transparent
    /// background. The 1 pixel will be set to the text color.
    ///
    /// Text is wrapped to multiple lines on line endings and on word
    /// boundaries if it extends beyond `wrap_length` in pixels. If
    /// `wrap_length` is 0, this function will only wrap on newline
    /// characters.
    ///
    /// You can render at other quality levels with
    /// [`FontHandle::render_text_shaded_wrapped`],
    /// [`FontHandle::render_text_blended_wrapped`], and
    /// [`FontHandle::render_text_lcd_wrapped`].
    #[doc(alias = "TTF_RenderText_Solid_Wrapped")]
    pub fn render_text_solid_wrapped(
        &self,
        text: RtStr,
        color: RgbaU8,
        wrap_length: i32,
    ) -> Result<Surface> {
        Surface::from_ptr(unsafe {
            TTF_RenderText_Solid_Wrapped(
                self.handle.as_ptr(),
                text.as_ptr(),
                text.len(),
                color.into(),
                wrap_length,
            )
        })
    }

    /// Query a font's family name.
    ///
    /// # Remarks
    ///
    /// This string is dictated by the contents of the font file.
    ///
    /// Note that the returned string is to internal storage, and should not
    /// be modified or free'd by the caller. The string becomes invalid, with
    /// the rest of the font, when the font is closed.
    #[doc(alias = "TTF_GetFontFamilyName")]
    pub fn family(&self) -> &str {
        unsafe { c_ptr_to_str(TTF_GetFontFamilyName(self.as_ptr())) }
    }

    /// Query whether a font is fixed-width.
    ///
    /// # Remarks
    ///
    /// A "fixed-width" font means all glyphs are the same width across; a
    /// lowercase 'i' will be the same size across as a capital 'W', for
    /// example. This is common for terminals and text editors, and other
    /// apps that treat text as a grid. Most other things (WYSIWYG word
    /// processors, web pages, etc) are more likely to not be fixed-width in
    /// most cases.
    #[doc(alias = "TTF_FontIsFixedWidth")]
    pub fn is_mono(&self) -> bool {
        unsafe { TTF_FontIsFixedWidth(self.as_ptr()) }
    }

    /// Query whether a font is scalable or not.
    ///
    /// # Remarks
    ///
    /// Scalability lets us distinguish between outline and bitmap fonts.
    #[doc(alias = "TTF_FontIsScalable")]
    pub fn is_scalable(&self) -> bool {
        unsafe { TTF_FontIsScalable(self.as_ptr()) }
    }
}

impl<'ttf> Font<'ttf> {
    /// Create a font from a file, using a specified point size.
    ///
    /// See [`Context::open`] for remarks.
    #[doc(alias = "TTF_OpenFont")]
    pub fn new(_ctx: &'ttf Context, file: &CStr, point_size: f32) -> Result<Self> {
        unsafe { Self::new_unchecked(file, point_size) }
    }

    /// # Safety
    /// Ensure a [`Context`] will exist for the entire lifetime of the returned font.
    /// That includes the point at which it's dropped. A segfault will probably
    /// happen otherwise.
    ///
    /// Create a font from a file, using a specified point size.
    ///
    /// See [`Context::open`] for remarks.
    #[doc(alias = "TTF_OpenFont")]
    pub unsafe fn new_unchecked(file: &CStr, point_size: f32) -> Result<Self> {
        Self::from_ptr(unsafe { TTF_OpenFont(file.as_ptr(), point_size) })
    }
}
