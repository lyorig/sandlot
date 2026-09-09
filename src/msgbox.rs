//! SDL offers a simple message box API, which is useful for simple alerts,
//! such as informing the user when something fatal happens at startup without
//! the need to build a UI for it (or informing the user _before_ your UI is
//! ready).
//!
//! These message boxes are native system dialogs where possible.
//!
//! There is both a customizable function (`SDL_ShowMessageBox`) that offers
//! lots of options for what to display and reports on what choice the user
//! made, and also a much-simplified version ([`show`]), which merely takes a
//! text message and title, and waits until the user presses a single "OK" UI
//! button. Often, this is all that is necessary.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryMessagebox)):
//! - [ ] SDL_ShowMessageBox
//! - [x] SDL_ShowSimpleMessageBox

use std::ffi::CStr;

use sdl3_sys::messagebox::*;

use crate::{Result, util::to_result};

/// Message box severity.
///
/// If supported, this will display a warning icon, etc.
#[repr(u32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_MessageBoxFlags")]
pub enum Severity {
    /// Error dialog.
    Error = SDL_MessageBoxFlags::ERROR.0,
    /// Warning dialog.
    Warning = SDL_MessageBoxFlags::WARNING.0,
    /// Informational dialog.
    Info = SDL_MessageBoxFlags::INFORMATION.0,
}

/// Message box button layout.
#[repr(u32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_MessageBoxFlags")]
pub enum ButtonLayout {
    /// Buttons placed left to right.
    LeftToRight = SDL_MessageBoxFlags::BUTTONS_LEFT_TO_RIGHT.0,
    /// Buttons placed right to left.
    RightToLeft = SDL_MessageBoxFlags::BUTTONS_RIGHT_TO_LEFT.0,
}

/// Display a simple modal message box.
///
/// `title` and `message` are UTF-8 text. The message box has no parent
/// window.
///
/// This function blocks execution of the calling thread until the user
/// clicks a button or closes the message box.
///
/// # Remarks
///
/// If your needs aren't complex, this is preferred over the full message
/// box API.
///
/// This function may be called at any time, even before SDL
/// initialization. This makes it useful for reporting errors like a
/// failure to create a renderer or OpenGL context.
///
/// On X11, SDL rolls its own dialog box with X11 primitives instead of a
/// formal toolkit like GTK+ or Qt.
///
/// Note that if SDL initialization would fail because there isn't any
/// available video target, this function is likely to fail for the same
/// reasons. If this is a concern, check the return value from this
/// function and fall back to writing to stderr if you can.
#[doc(alias = "SDL_ShowSimpleMessageBox")]
pub fn show(sev: Severity, bl: ButtonLayout, title: &CStr, message: &CStr) -> Result<()> {
    let flags = sev as u32 | bl as u32;
    to_result(unsafe {
        SDL_ShowSimpleMessageBox(
            SDL_MessageBoxFlags::new(flags),
            title.as_ptr(),
            message.as_ptr(),
            std::ptr::null_mut(),
        )
    })
}
