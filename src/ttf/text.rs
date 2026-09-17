//! API checklist:
//! - [x] TTF_AppendTextString
//! - [x] TTF_CreateText
//! - [x] TTF_DeleteTextString
//! - [x] TTF_DestroyText
//! - [x] TTF_DrawRendererText
//! - [x] TTF_DrawSurfaceText
//! - [x] TTF_GetNextTextSubString
//! - [x] TTF_GetGPUTextDrawData
//! - [x] TTF_GetPreviousTextSubString
//! - [x] TTF_GetTextColor
//! - [x] TTF_GetTextColorFloat
//! - [x] TTF_GetTextDirection
//! - [x] TTF_GetTextFont
//! - [x] TTF_GetTextPosition
//! - [x] TTF_GetTextProperties
//! - [x] TTF_GetTextScript
//! - [x] TTF_GetTextSize
//! - [x] TTF_GetTextSubString
//! - [x] TTF_GetTextSubStringForLine
//! - [x] TTF_GetTextSubStringForPoint
//! - [x] TTF_GetTextSubStringsForRange
//! - [x] TTF_GetTextWrapWidth
//! - [x] TTF_InsertTextString
//! - [x] TTF_SetTextColor
//! - [x] TTF_SetTextColorFloat
//! - [x] TTF_SetTextDirection
//! - [x] TTF_SetTextFont
//! - [x] TTF_SetTextPosition
//! - [x] TTF_SetTextScript
//! - [x] TTF_SetTextString
//! - [x] TTF_SetTextWrapWhitespaceVisible
//! - [x] TTF_SetTextWrapWidth
//! - [x] TTF_TextWrapWhitespaceVisible
//! - [x] TTF_UpdateText
//! - [x] TTF_SetTextEngine
//! - [x] TTF_GetTextEngine

use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use sdl3_sys::stdinc::SDL_free;
use sdl3_ttf_sys::ttf::*;

use crate::{
    Result,
    color::{RgbaF32, RgbaU8},
    error::Error,
    properties::{Properties, PropertiesHandle},
    rect::{PointF32, PointI32, RectI32},
    resource::{Handle, Ref, Resource, resource_new},
    surface::Surface,
    ttf::{Font, FontHandle, GpuEngine, RendererEngine, SurfaceEngine, TtfStr},
    util::{impl_enum_transmute, opt2res, to_result},
};

resource_new! {
    /// Representations of text with various properties.
    ///
    /// # Drawing text
    ///
    /// A [`Text`] cannot be drawn by itself.
    ///
    /// SDL_ttf has this weird API where you first assign an engine to a text object,
    /// after which you have to call the correct drawing function. For instance:
    /// 1. set a surface engine via [`TTF_SetTextEngine`]
    /// 1. draw to a surface via [`TTF_DrawSurfaceText`]
    ///
    /// What if you, hmm, set a surface engine, but then call [`TTF_DrawRendererText`]?
    /// I'm glad you asked, 'cause that's a one-way ticket to segfault land! So we can't
    /// really simply expose a combination of `set_engine` and `draw_to_surface`.
    ///
    /// Sandlot (hopefully) removes all this hassle via "engine'd text objects" (I'm not good with naming things).
    /// These are named `<Name>Text` for every `<Name>Engine`, e.g. [`SurfaceText`] for [`SurfaceEngine`].
    /// Their purpose is to own a [`Text`] object and create a scope where it can provably be used
    /// with a given engine. With this design, a single [`Text`] can be moved between different special
    /// text objects and prevent bugs at next to no overhead.
   pub struct Text<'ttf, 'font> : TTF_Text {
       marker: PhantomData<(Ref<'font, Font<'ttf>>)>,
   }

   /// Destroys a text object created by a text engine.
   ~TTF_DestroyText
}

/// Text direction flags.
///
/// # Remarks
///
/// The values here are chosen to match HarfBuzz's `hb_direction_t`.
#[repr(u32)]
#[derive(Clone, Copy)]
#[doc(alias = "TTF_Direction")]
pub enum Direction {
    /// Left to right.
    LeftToRight = TTF_Direction::LTR.0,
    /// Right to left.
    RightToLeft = TTF_Direction::RTL.0,
    /// Top to bottom.
    TopToBottom = TTF_Direction::TTB.0,
    /// Bottom to top.
    BottomToTop = TTF_Direction::BTT.0,
}

impl_enum_transmute!(TTF_Direction, Direction, INVALID);

bitflags::bitflags! {
    /// Flags for a [`SubString`].
    #[derive(Clone, Copy)]
    #[doc(alias = "TTF_SubStringFlags")]
    pub struct SubStringFlags: u32 {
        /// The mask for the flow direction for this substring.
        const DIRECTION_MASK = TTF_SubStringFlags::DIRECTION_MASK.0;
        /// This substring contains the beginning of the text.
        const TEXT_START = TTF_SubStringFlags::TEXT_START.0;
        /// This substring contains the beginning of line `line_index`.
        const LINE_START = TTF_SubStringFlags::LINE_START.0;
        /// This substring contains the end of line `line_index`.
        const LINE_END = TTF_SubStringFlags::LINE_END.0;
        /// This substring contains the end of the text.
        const TEXT_END = TTF_SubStringFlags::TEXT_END.0;
    }
}

/// A substring and its position in a [`Text`] object.
#[repr(C)]
#[doc(alias = "TTF_SubString")]
#[derive(Clone, Copy)]
pub struct SubString {
    /// The flags for this substring.
    pub flags: SubStringFlags,
    /// The byte offset from the beginning of the text.
    pub offset: i32,
    /// The byte length starting at the offset.
    pub length: i32,
    /// The index of the line that contains this substring.
    pub line_index: i32,
    /// The internal cluster index, used for quickly iterating.
    pub cluster_index: i32,
    /// The rectangle, relative to the top left of the text, containing the
    /// substring.
    pub rect: RectI32,
}

impl From<TTF_SubString> for SubString {
    fn from(value: TTF_SubString) -> Self {
        Self {
            flags: SubStringFlags::from_bits_retain(value.flags.0),
            offset: value.offset,
            length: value.length,
            line_index: value.line_index,
            cluster_index: value.cluster_index,
            rect: RectI32::from_sdl(value.rect),
        }
    }
}

impl<'ttf, 'font> TextHandle<'ttf, 'font> {
    /// Get the size of a text object, in pixels.
    ///
    /// # Remarks
    ///
    /// The size of the text may change when the font or font style and size
    /// change.
    #[doc(alias = "TTF_GetTextSize")]
    pub fn size(self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            TTF_GetTextSize(self.as_raw(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the color of a text object, in 8-bit components.
    #[doc(alias = "TTF_GetTextColor")]
    pub fn color_u8(self) -> RgbaU8 {
        let mut col = MaybeUninit::<RgbaU8>::uninit();
        let ptr = col.as_mut_ptr();

        unsafe {
            TTF_GetTextColor(
                self.as_raw(),
                &raw mut (*ptr).rgb.r,
                &raw mut (*ptr).rgb.g,
                &raw mut (*ptr).rgb.b,
                &raw mut (*ptr).a,
            );
            col.assume_init()
        }
    }

    /// Set the color of a text object, in 8-bit components in the range of
    /// 0-255.
    ///
    /// # Remarks
    ///
    /// The default text color is white (255, 255, 255, 255).
    #[doc(alias = "TTF_SetTextColor")]
    pub fn set_color_u8(self, color: RgbaU8) -> Result<()> {
        to_result(unsafe {
            TTF_SetTextColor(
                self.as_raw(),
                color.rgb.r,
                color.rgb.g,
                color.rgb.b,
                color.a,
            )
        })
    }

    /// Get the color of a text object, in floating-point components
    /// (normally in the range of 0-1).
    #[doc(alias = "TTF_GetTextColorFloat")]
    pub fn color_f32(self) -> RgbaF32 {
        let mut color = MaybeUninit::<RgbaF32>::uninit();
        let ptr = color.as_mut_ptr();
        unsafe {
            TTF_GetTextColorFloat(
                self.as_raw(),
                &raw mut (*ptr).rgb.r,
                &raw mut (*ptr).rgb.g,
                &raw mut (*ptr).rgb.b,
                &raw mut (*ptr).a,
            );
            color.assume_init()
        }
    }

    /// Set the color of a text object, in floating-point components
    /// (normally in the range of 0-1).
    ///
    /// # Remarks
    ///
    /// The default text color is white (1.0, 1.0, 1.0, 1.0).
    #[doc(alias = "TTF_SetTextColorFloat")]
    pub fn set_color_f32(self, color: RgbaF32) -> Result<()> {
        to_result(unsafe {
            TTF_SetTextColorFloat(
                self.as_raw(),
                color.rgb.r,
                color.rgb.g,
                color.rgb.b,
                color.a,
            )
        })
    }

    /// Get the direction to be used for text shaping a text object.
    ///
    /// # Remarks
    ///
    /// This defaults to the direction of the font used by the text object.
    #[doc(alias = "TTF_GetTextDirection")]
    pub fn direction(self) -> Direction {
        let dir = unsafe { TTF_GetTextDirection(self.as_raw()) };
        unsafe { Direction::from_sdl_unchecked(dir) }
    }

    /// Set the direction to be used for text shaping a text object.
    ///
    /// # Remarks
    ///
    /// This function only supports left-to-right text shaping if SDL_ttf was
    /// not built with HarfBuzz support.
    #[doc(alias = "TTF_SetTextDirection")]
    pub fn set_direction(self, direction: Direction) -> Result<()> {
        to_result(unsafe { TTF_SetTextDirection(self.as_raw(), direction.to_sdl()) })
    }

    /// Get the script used for text shaping a text object.
    ///
    /// Returns an
    /// [ISO 15924 code](https://unicode.org/iso15924/iso15924-codes.html),
    /// or 0 if a script hasn't been set on either the text object or the
    /// font.
    ///
    /// # Remarks
    ///
    /// This defaults to the script of the font used by the text object.
    #[doc(alias = "TTF_GetTextScript")]
    pub fn script(self) -> u32 {
        unsafe { TTF_GetTextScript(self.as_raw()) }
    }

    /// Set the script to be used for text shaping a text object.
    ///
    /// `script` is an
    /// [ISO 15924 code](https://unicode.org/iso15924/iso15924-codes.html).
    ///
    /// # Remarks
    ///
    /// This function fails if SDL_ttf isn't built with HarfBuzz support.
    #[doc(alias = "TTF_SetTextScript")]
    pub fn set_script(self, script: u32) -> Result<()> {
        to_result(unsafe { TTF_SetTextScript(self.as_raw(), script) })
    }

    /// Get the position of a text object.
    ///
    /// Returns the offset of the upper left corner of this text, in pixels.
    #[doc(alias = "TTF_GetTextPosition")]
    pub fn position(self) -> Result<PointI32> {
        let mut position = MaybeUninit::<PointI32>::uninit();
        let ptr = position.as_mut_ptr();
        to_result(unsafe {
            TTF_GetTextPosition(self.as_raw(), &raw mut (*ptr).x, &raw mut (*ptr).y)
        })?;
        Ok(unsafe { position.assume_init() })
    }

    /// Set the position of a text object.
    ///
    /// `position` is the offset of the upper left corner of this text, in
    /// pixels.
    ///
    /// # Remarks
    ///
    /// This can be used to position multiple text objects within a single
    /// wrapping text area.
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_SetTextPosition")]
    pub fn set_position(self, position: PointI32) -> Result<()> {
        to_result(unsafe { TTF_SetTextPosition(self.as_raw(), position.x, position.y) })
    }

    /// Get whether wrapping is enabled on a text object.
    ///
    /// Returns the maximum width in pixels, or 0 if the text is being wrapped
    /// on newline characters.
    #[doc(alias = "TTF_GetTextWrapWidth")]
    pub fn wrap_width(self) -> Result<i32> {
        let mut width = 0;
        to_result(unsafe { TTF_GetTextWrapWidth(self.as_raw(), &raw mut width) })?;
        Ok(width)
    }

    /// Set whether wrapping is enabled on a text object.
    ///
    /// `width` is the maximum width in pixels, or 0 to wrap on newline
    /// characters.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_SetTextWrapWidth")]
    pub fn set_wrap_width(self, width: i32) -> Result<()> {
        to_result(unsafe { TTF_SetTextWrapWidth(self.as_raw(), width) })
    }

    /// Return whether whitespace is shown when wrapping a text object.
    #[doc(alias = "TTF_TextWrapWhitespaceVisible")]
    pub fn wrap_whitespace_visible(self) -> bool {
        unsafe { TTF_TextWrapWhitespaceVisible(self.as_raw()) }
    }

    /// Set whether whitespace should be visible when wrapping a text object.
    ///
    /// # Remarks
    ///
    /// If the whitespace is visible, it will take up space for purposes of
    /// alignment and wrapping. This is good for editing, but looks better
    /// when centered or aligned if whitespace around line wrapping is
    /// hidden. This defaults to false.
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_SetTextWrapWhitespaceVisible")]
    pub fn set_wrap_whitespace_visible(self, visible: bool) -> Result<()> {
        to_result(unsafe { TTF_SetTextWrapWhitespaceVisible(self.as_raw(), visible) })
    }

    /// Get the font used by a text object.
    #[doc(alias = "TTF_GetTextFont")]
    pub fn font(self) -> Result<Ref<'font, Font<'ttf>>> {
        let font = unsafe { TTF_GetTextFont(self.as_raw()) };
        let handle = FontHandle::from_ptr(font).ok_or_else(Error::current)?;
        Ok(unsafe { Ref::from_handle(handle) })
    }

    /// Set the font used by a text object.
    ///
    /// # Remarks
    ///
    /// When a text object has a font, any changes to the font will
    /// automatically regenerate the text. If you set the font to [`None`],
    /// the text will continue to render but changes to the font will no
    /// longer affect the text.
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_SetTextFont")]
    pub fn set_font(self, font: Option<Ref<'font, Font<'ttf>>>) -> Result<()> {
        let font = font.map_or(std::ptr::null_mut(), |font| font.as_raw());
        to_result(unsafe { TTF_SetTextFont(self.as_raw(), font) })
    }

    /// Get the properties associated with a text object.
    #[doc(alias = "TTF_GetTextProperties")]
    pub fn properties(&self) -> Result<Ref<'_, Properties>> {
        let id = unsafe { TTF_GetTextProperties(self.as_raw()) };
        let handle = PropertiesHandle::from_id(id).ok_or_else(Error::current)?;
        Ok(unsafe { Ref::from_handle(handle) })
    }

    /// Get the substring of a text object that surrounds a text offset.
    ///
    /// `offset` is a byte offset into the text string.
    ///
    /// # Remarks
    ///
    /// If `offset` is less than 0, this will return a zero length substring
    /// at the beginning of the text with the `TTF_SUBSTRING_TEXT_START` flag
    /// set. If `offset` is greater than or equal to the length of the text
    /// string, this will return a zero length substring at the end of the
    /// text with the `TTF_SUBSTRING_TEXT_END` flag set.
    #[doc(alias = "TTF_GetTextSubString")]
    pub fn substring(self, offset: i32) -> Result<SubString> {
        let mut value = MaybeUninit::uninit();
        to_result(unsafe { TTF_GetTextSubString(self.as_raw(), offset, value.as_mut_ptr()) })?;
        Ok(SubString::from(unsafe { value.assume_init() }))
    }

    /// Get the substring of a text object that contains the given line.
    ///
    /// `line` is a zero-based line index, in the range
    /// `[0 .. num_lines-1]`.
    ///
    /// # Remarks
    ///
    /// If `line` is less than 0, this will return a zero length substring at
    /// the beginning of the text with the `TTF_SUBSTRING_TEXT_START` flag
    /// set. If `line` is greater than or equal to the number of lines, this
    /// will return a zero length substring at the end of the text with the
    /// `TTF_SUBSTRING_TEXT_END` flag set.
    #[doc(alias = "TTF_GetTextSubStringForLine")]
    pub fn substring_for_line(self, line: i32) -> Result<SubString> {
        let mut value = MaybeUninit::uninit();
        to_result(unsafe { TTF_GetTextSubStringForLine(self.as_raw(), line, value.as_mut_ptr()) })?;
        Ok(SubString::from(unsafe { value.assume_init() }))
    }

    /// Get the portion of a text string that is closest to a point.
    ///
    /// The point is relative to the top left of the text and may be outside
    /// the bounds of the text area.
    #[doc(alias = "TTF_GetTextSubStringForPoint")]
    pub fn substring_for_point(self, point: PointI32) -> Result<SubString> {
        let mut value = MaybeUninit::uninit();
        to_result(unsafe {
            TTF_GetTextSubStringForPoint(self.as_raw(), point.x, point.y, value.as_mut_ptr())
        })?;
        Ok(SubString::from(unsafe { value.assume_init() }))
    }

    /// Get the previous substring in a text object.
    ///
    /// # Remarks
    ///
    /// If called at the start of the text, this will return a zero length
    /// substring with the `TTF_SUBSTRING_TEXT_START` flag set.
    #[doc(alias = "TTF_GetPreviousTextSubString")]
    pub fn previous_substring(self, value: SubString) -> Result<SubString> {
        let mut previous = MaybeUninit::uninit();
        let value: TTF_SubString = unsafe { std::mem::transmute(value) };
        to_result(unsafe {
            TTF_GetPreviousTextSubString(self.as_raw(), &raw const value, previous.as_mut_ptr())
        })?;
        Ok(SubString::from(unsafe { previous.assume_init() }))
    }

    /// Get the next substring in a text object.
    ///
    /// # Remarks
    ///
    /// If called at the end of the text, this will return a zero length
    /// substring with the `TTF_SUBSTRING_TEXT_END` flag set.
    #[doc(alias = "TTF_GetNextTextSubString")]
    pub fn next_substring(self, value: SubString) -> Result<SubString> {
        let mut next = MaybeUninit::uninit();
        let value: TTF_SubString = unsafe { std::mem::transmute(value) };
        to_result(unsafe {
            TTF_GetNextTextSubString(self.as_raw(), &raw const value, next.as_mut_ptr())
        })?;
        Ok(SubString::from(unsafe { next.assume_init() }))
    }

    /// Get the substrings of a text object that contain a range of text.
    ///
    /// `offset` is a byte offset into the text string; `length` is the
    /// length of the range being queried, in bytes, or -1 for the remainder
    /// of the string.
    #[doc(alias = "TTF_GetTextSubStringsForRange")]
    pub fn substrings_for_range(self, offset: i32, length: i32) -> Result<Vec<SubString>> {
        let mut count = 0;
        let values =
            unsafe { TTF_GetTextSubStringsForRange(self.as_raw(), offset, length, &raw mut count) };

        let values = NonNull::new(values).ok_or_else(Error::current)?;
        let values = unsafe { std::slice::from_raw_parts(values.as_ptr(), count.max(0) as usize) };
        let result = values
            .iter()
            .filter_map(|value| unsafe { value.as_ref() })
            .copied()
            .map(SubString::from)
            .collect();

        unsafe { SDL_free(values.as_ptr().cast_mut().cast()) };

        Ok(result)
    }

    /// Update the layout of a text object.
    ///
    /// # Remarks
    ///
    /// This is automatically done when the layout is requested or the text
    /// is rendered, but you can call this if you need more control over the
    /// timing of when the layout and text engine representation are updated.
    #[doc(alias = "TTF_UpdateText")]
    pub fn update(self) -> Result<()> {
        to_result(unsafe { TTF_UpdateText(self.as_raw()) })
    }

    /// Set the text engine used by this text object.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be rebuilt.
    #[doc(alias = "TTF_SetTextEngine")]
    fn set_engine<H, R>(self, engine: Ref<R>)
    where
        H: Handle<Raw = *mut TTF_TextEngine>,
        R: Resource<Handle = H>,
    {
        // NOTE: Infallible, since the only error condition is an invalid engine object.
        unsafe { TTF_SetTextEngine(self.as_raw(), engine.as_raw()) };
    }

    /// # Safety
    /// Currently provided only for API coverage completeness, without a proper wrapper.
    /// See [`TTF_GetTextEngine`] docs for more info.
    ///
    /// Get the text engine used by a text object.
    #[doc(alias = "TTF_GetTextEngine")]
    pub unsafe fn engine(self) -> Result<NonNull<TTF_TextEngine>> {
        let eng = unsafe { TTF_GetTextEngine(self.as_raw()) };
        opt2res(NonNull::new(eng))
    }

    /// Set the UTF-8 text used by a text object.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_SetTextString")]
    pub fn set_string(self, text: TtfStr) -> Result<()> {
        to_result(unsafe { TTF_SetTextString(self.as_raw(), text.as_ptr(), text.len()) })
    }

    /// Insert UTF-8 text into a text object.
    ///
    /// `offset` is the offset, in bytes, from the beginning of the string if
    /// \>= 0, the offset from the end of the string if < 0. Note that this
    /// does not do UTF-8 validation, so you should only insert at UTF-8
    /// sequence boundaries.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_InsertTextString")]
    pub fn insert_string(self, offset: i32, text: TtfStr) -> Result<()> {
        to_result(unsafe { TTF_InsertTextString(self.as_raw(), offset, text.as_ptr(), text.len()) })
    }

    /// Append UTF-8 text to a text object.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_AppendTextString")]
    pub fn append_string(self, text: TtfStr) -> Result<()> {
        to_result(unsafe { TTF_AppendTextString(self.as_raw(), text.as_ptr(), text.len()) })
    }

    /// Delete UTF-8 text from a text object.
    ///
    /// `offset` is the offset, in bytes, from the beginning of the string if
    /// \>= 0, the offset from the end of the string if < 0; note that this
    /// does not do UTF-8 validation, so you should only delete at UTF-8
    /// sequence boundaries. `length` is the length of text to delete, in
    /// bytes, or -1 for the remainder of the string.
    ///
    /// # Remarks
    ///
    /// This function may cause the internal text representation to be
    /// rebuilt.
    #[doc(alias = "TTF_DeleteTextString")]
    pub fn delete_string(self, offset: i32, length: i32) -> Result<()> {
        to_result(unsafe { TTF_DeleteTextString(self.as_raw(), offset, length) })
    }
}

impl<'ttf, 'font> Text<'ttf, 'font> {
    /// Create a text object from UTF-8 text.
    #[doc(alias = "TTF_CreateText")]
    pub fn new(font: Ref<'font, Font<'ttf>>, text: TtfStr) -> Result<Self> {
        Self::from_ptr(unsafe {
            TTF_CreateText(
                std::ptr::null_mut(),
                font.as_raw(),
                text.as_ptr(),
                text.len(),
            )
        })
    }
}

/// A capsule for a [`Text`] object, during whose existence it is possible
/// to draw it using a [`RendererEngine`].
pub struct RendererText<'ttf, 'font, 'eng, 'ctx, 'vid, 'wnd, 'rnd> {
    text: Text<'ttf, 'font>,
    marker: PhantomData<Ref<'eng, RendererEngine<'ctx, 'vid, 'wnd, 'rnd>>>,
}

impl<'ttf, 'font, 'eng, 'ctx, 'vid, 'wnd, 'rnd>
    RendererText<'ttf, 'font, 'eng, 'ctx, 'vid, 'wnd, 'rnd>
{
    pub fn new(
        text: Text<'ttf, 'font>,
        eng: Ref<'eng, RendererEngine<'ctx, 'vid, 'wnd, 'rnd>>,
    ) -> Self {
        text.set_engine(eng);

        Self {
            text,
            marker: PhantomData,
        }
    }

    /// Consumes `self`, returning the underlying [`Text`].
    ///
    /// This makes it possible to use it with another text-with-renderer instance.
    pub fn into_text(self) -> Text<'ttf, 'font> {
        self.text
    }

    /// Draw text to an SDL renderer.
    ///
    /// `pos` is the coordinate in pixels, positive from the top left edge
    /// towards the bottom right.
    ///
    /// # Remarks
    ///
    /// The text must have been created using a renderer text engine, and will
    /// draw using the renderer passed to that engine.
    #[doc(alias = "TTF_DrawRendererText")]
    pub fn draw(&self, pos: PointF32) -> Result<()> {
        to_result(unsafe { TTF_DrawRendererText(self.as_raw(), pos.x, pos.y) })
    }
}

impl<'ttf, 'font, 'eng, 'ctx, 'vid, 'wnd, 'rnd> std::ops::Deref
    for RendererText<'ttf, 'font, 'eng, 'ctx, 'vid, 'wnd, 'rnd>
{
    type Target = Text<'ttf, 'font>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

/// A capsule for a [`Text`] object, during whose existence it is possible
/// to draw it using a [`SurfaceEngine`].
pub struct SurfaceText<'ttf, 'font, 'eng> {
    text: Text<'ttf, 'font>,
    marker: PhantomData<Ref<'eng, SurfaceEngine>>,
}

impl<'ttf, 'font, 'eng> SurfaceText<'ttf, 'font, 'eng> {
    pub fn new(text: Text<'ttf, 'font>, eng: Ref<'eng, SurfaceEngine>) -> Self {
        text.set_engine(eng);

        Self {
            text,
            marker: PhantomData,
        }
    }

    /// Consumes `self`, returning the underlying [`Text`].
    ///
    /// This makes it possible to use it with another text-with-renderer instance.
    pub fn into_text(self) -> Text<'ttf, 'font> {
        self.text
    }

    /// Draw text to an SDL surface.
    ///
    /// `pos` is the coordinate in pixels, positive from the top left edge
    /// towards the bottom right.
    #[doc(alias = "TTF_DrawSurfaceText")]
    pub fn draw(&self, surf: Ref<Surface>, pos: PointI32) -> Result<()> {
        to_result(unsafe { TTF_DrawSurfaceText(self.as_raw(), pos.x, pos.y, surf.as_raw()) })
    }
}

impl<'ttf, 'font, 'eng> std::ops::Deref for SurfaceText<'ttf, 'font, 'eng> {
    type Target = Text<'ttf, 'font>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

/// A capsule for a [`Text`] object, during whose existence it is possible
/// to draw it using a [`GpuEngine`].
pub struct GpuText<'ttf, 'font, 'eng, 'ctx, 'vid, 'dev> {
    text: Text<'ttf, 'font>,
    marker: PhantomData<Ref<'eng, GpuEngine<'ctx, 'vid, 'dev>>>,
}

impl<'ttf, 'font, 'eng, 'ctx, 'vid, 'dev> GpuText<'ttf, 'font, 'eng, 'ctx, 'vid, 'dev> {
    pub fn new(text: Text<'ttf, 'font>, eng: Ref<'eng, GpuEngine<'ctx, 'vid, 'dev>>) -> Self {
        text.set_engine(eng);

        Self {
            text,
            marker: PhantomData,
        }
    }

    /// Consumes `self`, returning the underlying [`Text`].
    ///
    /// This makes it possible to use it with another text-with-renderer instance.
    pub fn into_text(self) -> Text<'ttf, 'font> {
        self.text
    }

    /// # Safety
    ///
    /// Currently provided only for API coverage completeness, without a proper wrapper.
    /// See [`TTF_GetGPUTextDrawData`] docs for more info.
    ///
    /// Get the geometry data needed for drawing the text.
    ///
    /// Returns a NULL terminated linked list of atlas draw sequences, or an
    /// error if the passed text is empty or in case of failure.
    ///
    /// # Remarks
    ///
    /// The text must have been created using a GPU text engine.
    ///
    /// The positive X-axis is taken towards the right and the positive Y-axis
    /// is taken upwards for both the vertex and the texture coordinates, i.e,
    /// it follows the same convention used by the SDL_GPU API. If you want to
    /// use a different coordinate system you will need to transform the
    /// vertices yourself.
    ///
    /// If the text looks blocky use linear filtering.
    #[doc(alias = "TTF_GetGPUTextDrawData")]
    pub unsafe fn gpu_draw_data(&self) -> Result<NonNull<TTF_GPUAtlasDrawSequence>> {
        let data = unsafe { TTF_GetGPUTextDrawData(self.as_raw()) };
        opt2res(NonNull::new(data))
    }
}

impl<'ttf, 'font, 'eng, 'ctx, 'vid, 'dev> std::ops::Deref
    for GpuText<'ttf, 'font, 'eng, 'ctx, 'vid, 'dev>
{
    type Target = Text<'ttf, 'font>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}
