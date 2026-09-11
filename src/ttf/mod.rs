//! Loading and rendering using TrueType fonts.
//!
//! Implementation checklist:
//! - [ ] TTF_AddFallbackFont
//! - [ ] TTF_ClearFallbackFonts
//! - [x] TTF_CloseFont
//! - [x] TTF_CopyFont
//! - [ ] TTF_FontHasGlyph
//! - [x] TTF_FontIsFixedWidth
//! - [x] TTF_FontIsScalable
//! - [ ] TTF_GetFontAscent
//! - [ ] TTF_GetFontCharSpacing
//! - [ ] TTF_GetFontDescent
//! - [ ] TTF_GetFontDirection
//! - [ ] TTF_GetFontDPI
//! - [x] TTF_GetFontFamilyName
//! - [ ] TTF_GetFontGeneration
//! - [ ] TTF_GetFontHeight
//! - [ ] TTF_GetFontHinting
//! - [ ] TTF_GetFontKerning
//! - [ ] TTF_GetFontLineSkip
//! - [ ] TTF_GetFontOutline
//! - [ ] TTF_GetFontProperties
//! - [ ] TTF_GetFontScript
//! - [ ] TTF_GetFontSDF
//! - [ ] TTF_GetFontSize
//! - [ ] TTF_GetFontStyle
//! - [ ] TTF_GetFontStyleName
//! - [ ] TTF_GetFontWeight
//! - [ ] TTF_GetFontWrapAlignment
//! - [ ] TTF_GetFreeTypeVersion
//! - [ ] TTF_GetGlyphImage
//! - [ ] TTF_GetGlyphImageForIndex
//! - [ ] TTF_GetGlyphKerning
//! - [ ] TTF_GetGlyphMetrics
//! - [ ] TTF_GetGlyphScript
//! - [ ] TTF_GetHarfBuzzVersion
//! - [ ] TTF_GetNumFontFaces
//! - [ ] TTF_GetStringSize
//! - [ ] TTF_GetStringSizeWrapped
//! - [x] TTF_Init
//! - [ ] TTF_MeasureString
//! - [x] TTF_OpenFont
//! - [ ] TTF_OpenFontIO
//! - [ ] TTF_OpenFontWithProperties
//! - [x] TTF_Quit
//! - [ ] TTF_RemoveFallbackFont
//! - [x] TTF_RenderGlyph_Blended
//! - [x] TTF_RenderGlyph_LCD
//! - [x] TTF_RenderGlyph_Shaded
//! - [x] TTF_RenderGlyph_Solid
//! - [x] TTF_RenderText_Blended
//! - [x] TTF_RenderText_Blended_Wrapped
//! - [x] TTF_RenderText_LCD
//! - [x] TTF_RenderText_LCD_Wrapped
//! - [x] TTF_RenderText_Shaded
//! - [x] TTF_RenderText_Shaded_Wrapped
//! - [x] TTF_RenderText_Solid
//! - [x] TTF_RenderText_Solid_Wrapped
//! - [ ] TTF_SetFontCharSpacing
//! - [ ] TTF_SetFontDirection
//! - [ ] TTF_SetFontHinting
//! - [ ] TTF_SetFontKerning
//! - [ ] TTF_SetFontLanguage
//! - [ ] TTF_SetFontLineSkip
//! - [ ] TTF_SetFontOutline
//! - [ ] TTF_SetFontScript
//! - [ ] TTF_SetFontSDF
//! - [ ] TTF_SetFontSize
//! - [ ] TTF_SetFontSizeDPI
//! - [ ] TTF_SetFontStyle
//! - [ ] TTF_SetFontWrapAlignment
//! - [ ] TTF_StringToTag
//! - [ ] TTF_TagToString
//! - [x] TTF_Version
//! - [x] TTF_WasInit

mod_reexport!(engine);
mod_reexport!(font);
mod_reexport!(rt_str);
mod_reexport!(text);

use std::ffi::CStr;

use sdl3_ttf_sys::ttf::*;

use crate::{Result, error::Error, util::mod_reexport};

/// Check if SDL_ttf is initialized.
///
/// Returns the current number of initialization calls, that need to
/// eventually be paired with this many calls to SDL_ttf's quit function.
///
/// # Remarks
///
/// This reports the number of times the library has been initialized by a
/// call to [`Context::new`], without a paired deinitialization request.
///
/// In short: if it's greater than zero, the library is currently initialized
/// and ready to work. If zero, it is not initialized.
///
/// Despite the return value being a signed integer, this function should not
/// return a negative number.
#[doc(alias = "TTF_WasInit")]
pub fn num_init() -> i32 {
    unsafe { TTF_WasInit() }
}

/// Check if SDL_ttf is initialized.
#[doc(alias = "TTF_WasInit")]
pub fn is_init() -> bool {
    num_init() != 0
}

/// Get the version of the dynamically linked SDL_ttf library.
#[doc(alias = "TTF_Version")]
pub fn version() -> i32 {
    TTF_Version()
}

/// Ensures SDL_ttf (de)initialization.
///
/// You must successfully create this before it is safe to call any other
/// function in this library. It is safe to create more than once, and each
/// successful creation should be paired with a matching deinitialization
/// (the destructor), which happens when the [`Context`] goes out of scope.
pub struct Context;

impl Context {
    /// Initialize SDL_ttf.
    #[doc(alias = "TTF_Init")]
    pub fn new() -> Result<Self> {
        if unsafe { TTF_Init() } {
            Ok(Self {})
        } else {
            Err(Error::current())
        }
    }

    /// Create a font from a file, using a specified point size.
    ///
    /// # Remarks
    ///
    /// Some .fon fonts will have several sizes embedded in the file, so the
    /// point size becomes the index of choosing which size. If the value is
    /// too high, the last indexed size will be the default.
    #[doc(alias = "TTF_OpenFont")]
    pub fn open(&self, file: &CStr, point_size: f32) -> Result<Font<'_>> {
        unsafe { Font::new_unchecked(file, point_size) }
    }
}

impl Drop for Context {
    /// Deinitialize SDL_ttf.
    ///
    /// # Remarks
    ///
    /// You must call this when done with the library, to free internal
    /// resources. It is safe to do so when the library isn't initialized, as
    /// it will just return immediately.
    ///
    /// Once there are as many quit calls as there have been successful
    /// initializations, the library will actually deinitialize.
    ///
    /// Please note that this does not automatically close any fonts that are
    /// still open at the time of deinitialization, and it is possibly not
    /// safe to close them afterwards, as parts of the library will no longer
    /// be initialized to deal with it. A well-written program should close
    /// any open fonts before this happens!
    #[doc(alias = "TTF_Quit")]
    fn drop(&mut self) {
        unsafe { TTF_Quit() };
    }
}
