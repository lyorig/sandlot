//! System clipboard introspection and manipulation.
//!
//! API checklist ([source](https://wiki.libsdl.org/SDL3/CategoryRender)):
//! - [x] SDL_ClearClipboardData
//! - [x] SDL_GetClipboardData
//! - [x] SDL_GetClipboardMimeTypes
//! - [x] SDL_GetClipboardText
//! - [ ] SDL_GetPrimarySelectionText
//! - [x] SDL_HasClipboardData
//! - [x] SDL_HasClipboardText
//! - [ ] SDL_HasPrimarySelectionText
//! - [ ] SDL_SetClipboardData
//! - [x] SDL_SetClipboardText
//! - [ ] SDL_SetPrimarySelectionText

use std::{ffi::CStr, mem::MaybeUninit, ptr::NonNull};

use sdl3_sys::clipboard::*;

use crate::{Result, boxed::Box, string::String, util::to_result};

/// Clear the clipboard data.
#[doc(alias = "SDL_ClearClipboardData")]
pub fn clear_data() -> Result<()> {
    to_result(unsafe { SDL_ClearClipboardData() })
}

/// Get the data from the clipboard for a given mime type.
///
/// Returns the retrieved data buffer.
///
/// # Remarks
///
/// The size of text data does not include the terminator, but the text is
/// guaranteed to be null-terminated.
#[doc(alias = "SDL_GetClipboardData")]
pub fn data(mime_type: &CStr) -> Result<Box<[u8]>> {
    let mut len = MaybeUninit::<usize>::uninit();
    let ptr = unsafe { SDL_GetClipboardData(mime_type.as_ptr(), len.as_mut_ptr()) };
    // SAFETY: On success, SDL allocates `len` bytes.
    unsafe { Box::from_raw_parts_nullck(ptr.cast(), len.assume_init() as _) }
}

/// Retrieve the list of mime types available in the clipboard.
#[doc(alias = "SDL_GetClipboardMimeTypes")]
pub fn mime_types() -> Result<Box<[NonNull<i8>]>> {
    let mut len = MaybeUninit::<usize>::uninit();
    let ptr = unsafe { SDL_GetClipboardMimeTypes(len.as_mut_ptr()) };
    // SAFETY: On success, SDL allocates `len` mime type strings.
    unsafe { Box::from_raw_parts_nullck(ptr.cast(), len.assume_init()) }
}

/// Get UTF-8 text from the clipboard.
///
/// Returns an empty string if there is not enough memory left for a copy of
/// the clipboard's content.
#[doc(alias = "SDL_GetClipboardText")]
pub fn text() -> String {
    let ptr = unsafe { SDL_GetClipboardText() };

    // SAFETY: `SDL_GetClipboardText()` always returns a valid string.
    unsafe { String::from_raw(ptr) }
}

/// Query whether there is data in the clipboard for the provided mime type.
#[doc(alias = "SDL_HasClipboardData")]
pub fn has_data(mime_type: &CStr) -> bool {
    unsafe { SDL_HasClipboardData(mime_type.as_ptr()) }
}

/// Query whether the clipboard exists and contains a non-empty text string.
#[doc(alias = "SDL_HasClipboardText")]
pub fn has_text() -> bool {
    unsafe { SDL_HasClipboardText() }
}

/// Put UTF-8 text into the clipboard.
#[doc(alias = "SDL_SetClipboardText")]
pub fn set_text(text: &CStr) -> Result<()> {
    to_result(unsafe { SDL_SetClipboardText(text.as_ptr()) })
}
