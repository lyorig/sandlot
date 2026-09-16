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
//! - [x] SDL_StartTextInput (impl'd as [`VideoHandle::enable_text_input`])
//! - [ ] SDL_StartTextInputWithProperties
//! - [x] SDL_StopTextInput (impl'd as [`VideoHandle::disable_text_input`])
//! - [x] SDL_TextInputActive (impl'd as [`VideoHandle::is_text_input_enabled`])
//! - [ ] SDL_SetTextInputArea

use std::marker::PhantomData;

use sdl3_sys::{keyboard::*, keycode::SDL_Keycode, scancode::SDL_Scancode};

use crate::{init, util::c_ptr_to_str};

// doc-only
#[expect(unused_imports)]
use crate::init::{EventsHandle, VideoHandle};

const NUM_SCANCODES: usize = SDL_Scancode::COUNT.0 as _;

/// Wrapper around the pointer returned by [`SDL_GetKeyboardState`].
#[derive(Clone, Copy)]
pub struct KeyboardState<'ctx, 'vid> {
    state: &'vid [bool; NUM_SCANCODES],
    marker: PhantomData<init::Ref<'vid, init::Video<'ctx>>>,
}

impl<'ctx, 'vid> KeyboardState<'ctx, 'vid> {
    /// # Safety
    ///
    /// The caller must only read the array before SDL has a chance to modify it
    /// (e.g. between frames).
    ///
    /// This is because the contained reference violates Rust's aliasing rules.
    /// SDL modifies the array during event pumping, so it's only immutable between
    /// calls to [`EventsHandle::pump`] (often called implicitly by other functions).
    ///
    /// Keeping the reference across these "pumps" risks the underlying data
    /// being mutated by SDL and causing UB.
    pub(crate) unsafe fn new() -> Self {
        let state = {
            let ptr = unsafe { SDL_GetKeyboardState(std::ptr::null_mut()) };
            unsafe { ptr.cast::<[bool; NUM_SCANCODES]>().as_ref_unchecked() }
        };

        Self {
            state,
            marker: PhantomData,
        }
    }

    /// Returns whether `sc` has been pressed during the last call to [`EventsHandle::pump`].
    pub fn pressed(self, sc: SDL_Scancode) -> bool {
        self.state[sc.0 as usize]
    }
}

/// Get a human-readable name for a scancode.
///
/// Returns an empty string if the scancode doesn't have a name.
///
/// # Warning
///
/// The returned name is by design not stable across platforms,
/// e.g. the name for [`SDL_Scancode::LGUI`] is "Left GUI" under Linux but
/// "Left Windows" under Microsoft Windows, and some scancodes like
/// [`SDL_Scancode::NONUSBACKSLASH`] don't have any name at all. There are even
/// scancodes that share names, e.g. [`SDL_Scancode::RETURN`] and
/// [`SDL_Scancode::RETURN2`] (both called "Return"). This function is
/// therefore unsuitable for creating a stable cross-platform two-way
/// mapping between strings and scancodes.
#[doc(alias = "SDL_GetScancodeName")]
pub fn scancode_name(scancode: SDL_Scancode) -> &'static str {
    unsafe {
        let ptr = SDL_GetScancodeName(scancode);
        c_ptr_to_str(ptr)
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
        let ptr = SDL_GetKeyName(key);
        c_ptr_to_str(ptr)
    }
}
