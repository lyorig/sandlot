use std::{ffi::CStr, mem::MaybeUninit};

use sdl3_sys::{clipboard::*, keyboard::*, keycode::SDL_Keymod, video::*};

use crate::{
    Result,
    boxed::Box,
    display::Display,
    error::Error,
    init::{Events, subsystem_new},
    keyboard::KeyboardState,
    rect::{PointI32, RectI32},
    resource,
    str::Str,
    string::String,
    util::to_result,
    window::{Window, WindowHandle, WindowId},
};

#[expect(unused_imports)] // doc-only
use crate::{event::Event, init::EventsHandle};

#[expect(unused_imports)] // doc-only
use sdl3_sys::events::SDL_WindowEvent;

subsystem_new!(
    /// The video subsystem provides access to the display and windowing system.
    /// Also initializes the events subsystem, accessible via [`VideoHandle::events`].
    Video, VIDEO, Events => events);

impl<'ctx> VideoHandle<'ctx> {
    /// Get the window that currently has an input grab enabled.
    ///
    /// Returns [`None`] if input is not grabbed.
    ///
    /// # Safety
    ///
    /// The caller must only use the returned handle before its respective window is destroyed.
    #[doc(alias = "SDL_GetGrabbedWindow")]
    pub unsafe fn grabbed_window(&self) -> Option<WindowHandle<'ctx, '_>> {
        WindowHandle::from_raw(unsafe { SDL_GetGrabbedWindow() })
    }

    /// Get a list of valid windows.
    ///
    /// # Safety
    ///
    /// The caller must only use the returned handles before their respective windows are destroyed.
    #[doc(alias = "SDL_GetWindows")]
    pub unsafe fn windows(&self) -> Result<Box<[WindowHandle<'ctx, '_>]>> {
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetWindows(count.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `count` window pointers.
        // `Ref<Window>` as the same size and alignment as `*mut SDL_Window`.
        let bx = unsafe { Box::from_raw_parts(ptr.cast(), count.assume_init() as usize) };
        bx.ok_or_else(Error::current)
    }

    /// Get a window from a stored ID.
    ///
    /// Returns [`None`] if no window with that ID exists.
    ///
    /// # Safety
    ///
    /// The caller must only use the returned handle before its respective window is destroyed.
    ///
    /// # Remarks
    ///
    /// The numeric ID is what [`SDL_WindowEvent`] references, and is necessary
    /// to map these events to specific window objects.
    #[doc(alias = "SDL_GetWindowFromID")]
    pub unsafe fn window_from_raw(&self, id: WindowId) -> Option<WindowHandle<'ctx, '_>> {
        let ptr = unsafe { SDL_GetWindowFromID(id.as_raw()) };
        WindowHandle::from_raw(ptr)
    }

    /// Get a list of currently connected displays.
    #[doc(alias = "SDL_GetDisplays")]
    pub fn displays_all(&self) -> Result<Box<[Display<'ctx, '_>]>> {
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetDisplays(count.as_mut_ptr()) };

        let bx = unsafe { Box::from_raw_parts(ptr.cast(), count.assume_init() as _) };
        bx.ok_or_else(Error::current)
    }

    /// Return the primary display.
    #[doc(alias = "SDL_GetPrimaryDisplay")]
    pub fn display_primary(&self) -> Result<Display<'ctx, '_>> {
        Display::from_sdl(unsafe { SDL_GetPrimaryDisplay() })
    }

    /// Get the display containing a point.
    #[doc(alias = "SDL_GetDisplayForPoint")]
    pub fn display_for_point(&self, point: PointI32) -> Result<Display<'ctx, '_>> {
        Display::from_sdl(unsafe { SDL_GetDisplayForPoint(point.as_sdl_ptr()) })
    }

    /// Get the display primarily containing a rect.
    ///
    /// Returns the display entirely containing the rect, or closest to the
    /// center of the rect.
    #[doc(alias = "SDL_GetDisplayForRect")]
    pub fn display_for_rect(&self, rect: RectI32) -> Result<Display<'ctx, '_>> {
        Display::from_sdl(unsafe { SDL_GetDisplayForRect(rect.as_sdl_ptr()) })
    }

    /// Clear the clipboard data.
    #[doc(alias = "SDL_ClearClipboardData")]
    pub fn clipboard_clear_data(self) -> Result<()> {
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
    pub fn clipboard_data(self, mime_type: &CStr) -> Result<Box<[u8]>> {
        let mut len = MaybeUninit::<usize>::uninit();
        let ptr = unsafe { SDL_GetClipboardData(mime_type.as_ptr(), len.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `len` bytes.
        let bx = unsafe { Box::from_raw_parts(ptr.cast(), len.assume_init() as _) };
        bx.ok_or_else(Error::current)
    }

    /// Retrieve the list of mime types available in the clipboard.
    #[doc(alias = "SDL_GetClipboardMimeTypes")]
    pub fn clipboard_mime_types(&self) -> Result<Box<[Str<'_>]>> {
        let mut len = MaybeUninit::<usize>::uninit();
        let ptr = unsafe { SDL_GetClipboardMimeTypes(len.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `len` mime type strings.
        let bx = unsafe { Box::from_raw_parts(ptr.cast(), len.assume_init()) };
        bx.ok_or_else(Error::current)
    }

    /// Get UTF-8 text from the clipboard.
    ///
    /// Returns an empty string if there is not enough memory left for a copy of
    /// the clipboard's content.
    #[doc(alias = "SDL_GetClipboardText")]
    pub fn clipboard_text(self) -> String {
        let ptr = unsafe { SDL_GetClipboardText() };

        // SAFETY: `SDL_GetClipboardText()` always returns a valid string.
        unsafe { String::from_ptr_unchecked(ptr) }
    }

    /// Query whether there is data in the clipboard for the provided mime type.
    #[doc(alias = "SDL_HasClipboardData")]
    pub fn clipboard_has_data(self, mime_type: &CStr) -> bool {
        unsafe { SDL_HasClipboardData(mime_type.as_ptr()) }
    }

    /// Query whether the clipboard exists and contains a non-empty text string.
    #[doc(alias = "SDL_HasClipboardText")]
    pub fn clipboard_has_text(self) -> bool {
        unsafe { SDL_HasClipboardText() }
    }

    /// Put UTF-8 text into the clipboard.
    #[doc(alias = "SDL_SetClipboardText")]
    pub fn clipboard_set_text(self, text: Str) -> Result<()> {
        to_result(unsafe { SDL_SetClipboardText(text.as_ptr()) })
    }

    /// Check whether the screensaver is currently enabled.
    ///
    /// # Remarks
    ///
    /// The screensaver is disabled by default.
    ///
    /// The default can also be changed using
    /// `SDL_HINT_VIDEO_ALLOW_SCREENSAVER`.
    #[doc(alias = "SDL_ScreenSaverEnabled")]
    pub fn is_screen_saver_enabled(self) -> bool {
        unsafe { SDL_ScreenSaverEnabled() }
    }

    /// Allow the screen to be blanked by a screen saver.
    #[doc(alias = "SDL_EnableScreenSaver")]
    pub fn enable_screen_saver(self) -> Result<()> {
        to_result(unsafe { SDL_EnableScreenSaver() })
    }

    /// Prevent the screen from being blanked by a screen saver.
    ///
    /// # Remarks
    ///
    /// If you disable the screensaver, it is automatically re-enabled when SDL
    /// quits.
    ///
    /// The screensaver is disabled by default, but this may be changed by
    /// `SDL_HINT_VIDEO_ALLOW_SCREENSAVER`.
    #[doc(alias = "SDL_DisableScreenSaver")]
    pub fn disable_screen_saver(self) -> Result<()> {
        to_result(unsafe { SDL_DisableScreenSaver() })
    }

    /// Get a snapshot of the current state of the keyboard.
    ///
    /// # Safety
    ///
    /// The caller must only read the array before another call to [`EventsHandle::pump`].
    ///
    /// This is because the contained reference violates Rust's aliasing rules.
    /// SDL modifies the array during event pumping, so it's only immutable between
    /// calls to [`EventsHandle::pump`] (often called implicitly by other functions).
    ///
    /// Keeping the reference across these "pumps" risks the underlying data
    /// being mutated by SDL and causing UB.
    ///
    /// # Remarks
    ///
    /// The returned slice points to an internal SDL array. It will be valid for
    /// the whole lifetime of the application and should not be freed by the
    /// caller.
    ///
    /// Use [`EventsHandle::pump`] to update the state array.
    ///
    /// This function gives you the current state after all events have been
    /// processed, so if a key or button has been pressed and released before you
    /// process events, then the pressed state will never show up in the
    /// returned snapshot.
    ///
    /// Note: This function doesn't take into account whether shift has been
    /// pressed or not.
    #[doc(alias = "SDL_GetKeyboardState")]
    pub unsafe fn keyboard_state(&self) -> KeyboardState<'ctx, '_> {
        unsafe { KeyboardState::new() }
    }

    /// Get the current key modifier state for the keyboard.
    ///
    /// Returns an OR'd combination of the modifier keys for the keyboard.
    #[doc(alias = "SDL_GetModState")]
    pub fn keyboard_mod_state(self) -> SDL_Keymod {
        unsafe { SDL_GetModState() }
    }

    /// Start accepting Unicode text input events in a window.
    ///
    /// # Remarks
    ///
    /// This function will enable text input ([`Event::TextInput`] and
    /// [`Event::TextEditing`] events) in the specified window. Please use
    /// this function paired with [`VideoHandle::disable_text_input`].
    ///
    /// Text input events are not received by default.
    ///
    /// On some platforms using this function shows the screen keyboard and/or
    /// activates an IME, which can prevent some key press events from being
    /// passed through.
    #[doc(alias = "SDL_StartTextInput")]
    pub fn enable_text_input(self, wnd: resource::Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StartTextInput(wnd.as_raw()) })
    }

    /// Stop receiving any text input events in a window.
    ///
    /// # Remarks
    ///
    /// If [`VideoHandle::enable_text_input`] showed the screen keyboard,
    /// this function will hide it.
    #[doc(alias = "SDL_StopTextInput")]
    pub fn disable_text_input(self, wnd: resource::Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StopTextInput(wnd.as_raw()) })
    }

    /// Check whether or not Unicode text input events are enabled for a window.
    #[doc(alias = "SDL_TextInputActive")]
    pub fn is_text_input_enabled(self, wnd: resource::Ref<Window>) -> bool {
        unsafe { SDL_TextInputActive(wnd.as_raw()) }
    }
}
