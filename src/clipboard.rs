//! SDL provides access to the system clipboard, both for reading information
//! from other processes and publishing information of its own.
//!
//! This is not just text! SDL apps can access and publish data by mimetype.
//!
//! # Basic use (text)
//!
//! Obtaining and publishing simple text to the system clipboard is as easy as
//! calling [`text`] and [`set_text`], respectively. These deal with strings
//! in UTF-8 encoding. Data transmission and encoding conversion is completely
//! managed by SDL.
//!
//! # Clipboard callbacks (data other than text)
//!
//! Things get more complicated when the clipboard contains something other
//! than text. Not only can the system clipboard contain data of any type, in
//! some cases it can contain the same data in different formats! For example,
//! an image painting app might let the user copy a graphic to the clipboard,
//! and offers it in .BMP, .JPG, or .PNG format for other apps to consume.
//!
//! Obtaining clipboard data ("pasting") like this is a matter of calling
//! [`data`] and telling it the mimetype of the data you want. But how does
//! one know if that format is available? [`has_data`] can report if a
//! specific mimetype is offered, and [`mime_types`] can provide the entire
//! list of mimetypes available, so the app can decide what to do with the
//! data and what formats it can support.
//!
//! Setting the clipboard ("copying") to arbitrary data is done with
//! `SDL_SetClipboardData`. The app does not provide the data in this call,
//! but rather the mimetypes it is willing to provide and a callback function.
//! During the callback, the app will generate the data. This allows massive
//! data sets to be provided to the clipboard, without any data being copied
//! before it is explicitly requested. More specifically, it allows an app to
//! offer data in multiple formats without providing a copy of all of them
//! upfront. If the app has an image that it could provide in PNG or JPG
//! format, it doesn't have to encode it to either of those unless and until
//! something tries to paste it.
//!
//! # Primary Selection
//!
//! The X11 and Wayland video targets have a concept of the "primary
//! selection" in addition to the usual clipboard. This is generally
//! highlighted (but not explicitly copied) text from various apps. SDL offers
//! APIs for this through `SDL_GetPrimarySelectionText` and
//! `SDL_SetPrimarySelectionText`. SDL offers these APIs on platforms without
//! this concept, too, but only so far that it will keep a copy of a string
//! that the app sets for later retrieval; the operating system will not ever
//! attempt to change the string externally if it doesn't support a primary
//! selection.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryClipboard)):
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
