//! Keyboard management.
//!
//! Please refer to the [Best Keyboard Practices](https://wiki.libsdl.org/SDL3/BestKeyboardPractices)
//! document for details on how best to accept keyboard input in various types of programs:
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryKeyboard)):
//! - [ ] SDL_ClearComposition
//! - [ ] SDL_GetKeyboardFocus
//! - [ ] SDL_GetKeyboardNameForID
//! - [ ] SDL_GetKeyboards
//! - [x] SDL_GetKeyboardState
//! - [ ] SDL_GetKeyFromName
//! - [ ] SDL_GetKeyFromScancode
//! - [x] SDL_GetKeyName
//! - [x] SDL_GetModState
//! - [ ] SDL_GetScancodeFromKey
//! - [ ] SDL_GetScancodeFromName
//! - [x] SDL_GetScancodeName
//! - [ ] SDL_GetTextInputArea
//! - [ ] SDL_HasKeyboard
//! - [ ] SDL_HasScreenKeyboardSupport
//! - [ ] SDL_ResetKeyboard
//! - [ ] SDL_ScreenKeyboardShown
//! - [ ] SDL_SetModState
//! - [ ] SDL_SetScancodeName
//! - [x] SDL_StartTextInput (impl'd as [`Event::enable_text_input`])
//! - [ ] SDL_StartTextInputWithProperties
//! - [x] SDL_StopTextInput (impl'd as [`Event::disable_text_input`])
//! - [x] SDL_TextInputActive (impl'd as [`Event::is_text_input_enabled`])
//! - [ ] SDL_SetTextInputArea

use std::ffi::CStr;

use sdl3_sys::{
    keyboard::*,
    keycode::{SDL_Keycode, SDL_Keymod},
    scancode::{SDL_SCANCODE_COUNT, SDL_Scancode},
};

// doc-only
#[expect(unused_imports)]
use crate::event::Event;

const NUM_SCANCODES: usize = SDL_SCANCODE_COUNT.0 as usize;

/// Get a human-readable name for a scancode.
///
/// Returns an empty string if the scancode doesn't have a name.
///
/// # Warning
///
/// The returned name is by design not stable across platforms,
/// e.g. the name for `SDL_SCANCODE_LGUI` is "Left GUI" under Linux but
/// "Left Windows" under Microsoft Windows, and some scancodes like
/// `SDL_SCANCODE_NONUSBACKSLASH` don't have any name at all. There are even
/// scancodes that share names, e.g. `SDL_SCANCODE_RETURN` and
/// `SDL_SCANCODE_RETURN2` (both called "Return"). This function is
/// therefore unsuitable for creating a stable cross-platform two-way
/// mapping between strings and scancodes.
#[doc(alias = "SDL_GetScancodeName")]
pub fn scancode_name(scancode: SDL_Scancode) -> &'static str {
    unsafe {
        let cstr = CStr::from_ptr(SDL_GetScancodeName(scancode)).to_bytes();
        str::from_utf8_unchecked(cstr)
    }
}

/// Get a human-readable name for a key, in UTF-8.
///
/// Returns an empty string if the key doesn't have a name.
///
/// # Remarks
///
/// Letters will be presented in their uppercase form, if applicable.
#[doc(alias = "SDL_GetKeyName")]
pub fn key_name(key: SDL_Keycode) -> &'static str {
    unsafe {
        let cstr = CStr::from_ptr(SDL_GetKeyName(key)).to_bytes();
        str::from_utf8_unchecked(cstr)
    }
}

/// Get a snapshot of the current state of the keyboard.
///
/// Returns an array indexed by [`SDL_Scancode`] values, whose elements are
/// `true` when the key is pressed and `false` when it is not.
///
/// # Remarks
///
/// The returned slice points to an internal SDL array. It will be valid for
/// the whole lifetime of the application and should not be freed by the
/// caller.
///
/// Use `SDL_PumpEvents` to update the state array.
///
/// This function gives you the current state after all events have been
/// processed, so if a key or button has been pressed and released before you
/// process events, then the pressed state will never show up in the
/// returned snapshot.
///
/// Note: This function doesn't take into account whether shift has been
/// pressed or not.
#[doc(alias = "SDL_GetKeyboardState")]
pub fn keyboard_state() -> &'static [bool; NUM_SCANCODES] {
    unsafe {
        let ptr = SDL_GetKeyboardState(std::ptr::null_mut()).cast::<[bool; NUM_SCANCODES]>();
        ptr.as_ref_unchecked()
    }
}

/// Get the current key modifier state for the keyboard.
///
/// Returns an OR'd combination of the modifier keys for the keyboard.
#[doc(alias = "SDL_GetModState")]
pub fn mod_state() -> SDL_Keymod {
    unsafe { SDL_GetModState() }
}
