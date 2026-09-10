//! SDL's window API wrapper.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryVideo)):
//! - [x] SDL_CreatePopupWindow
//! - [x] SDL_CreateWindow
//! - [x] SDL_CreateWindowWithProperties
//! - [x] SDL_DestroyWindow
//! - [x] SDL_DestroyWindowSurface
//! - [x] SDL_DisableScreenSaver
//! - [ ] SDL_EGL_GetCurrentConfig
//! - [ ] SDL_EGL_GetCurrentDisplay
//! - [ ] SDL_EGL_GetProcAddress
//! - [ ] SDL_EGL_GetWindowSurface
//! - [ ] SDL_EGL_SetAttributeCallbacks
//! - [x] SDL_EnableScreenSaver
//! - [x] SDL_FlashWindow
//! - [x] SDL_GetCurrentVideoDriver
//! - [x] SDL_GetDisplayForWindow
//! - [x] SDL_GetGrabbedWindow
//! - [x] SDL_GetNumVideoDrivers
//! - [x] SDL_GetSystemTheme
//! - [x] SDL_GetVideoDriver
//! - [x] SDL_GetWindowAspectRatio
//! - [x] SDL_GetWindowBordersSize
//! - [x] SDL_GetWindowDisplayScale
//! - [x] SDL_GetWindowFlags
//! - [x] SDL_GetWindowFromID
//! - [x] SDL_GetWindowFullscreenMode
//! - [x] SDL_GetWindowICCProfile
//! - [x] SDL_GetWindowID
//! - [x] SDL_GetWindowKeyboardGrab
//! - [x] SDL_GetWindowMaximumSize
//! - [x] SDL_GetWindowMinimumSize
//! - [x] SDL_GetWindowMouseGrab
//! - [x] SDL_GetWindowMouseRect
//! - [x] SDL_GetWindowOpacity
//! - [x] SDL_GetWindowParent
//! - [x] SDL_GetWindowPixelDensity
//! - [x] SDL_GetWindowPixelFormat
//! - [x] SDL_GetWindowPosition
//! - [x] SDL_GetWindowProgressState
//! - [x] SDL_GetWindowProgressValue
//! - [x] SDL_GetWindowProperties
//! - [x] SDL_GetWindows
//! - [x] SDL_GetWindowSafeArea
//! - [x] SDL_GetWindowSize
//! - [x] SDL_GetWindowSizeInPixels
//! - [x] SDL_GetWindowSurface
//! - [x] SDL_GetWindowSurfaceVSync
//! - [x] SDL_GetWindowTitle
//! - [ ] SDL_GL_CreateContext
//! - [ ] SDL_GL_DestroyContext
//! - [ ] SDL_GL_ExtensionSupported
//! - [ ] SDL_GL_GetAttribute
//! - [ ] SDL_GL_GetCurrentContext
//! - [ ] SDL_GL_GetCurrentWindow
//! - [ ] SDL_GL_GetProcAddress
//! - [ ] SDL_GL_GetSwapInterval
//! - [ ] SDL_GL_LoadLibrary
//! - [ ] SDL_GL_MakeCurrent
//! - [ ] SDL_GL_ResetAttributes
//! - [ ] SDL_GL_SetAttribute
//! - [ ] SDL_GL_SetSwapInterval
//! - [ ] SDL_GL_SwapWindow
//! - [ ] SDL_GL_UnloadLibrary
//! - [x] SDL_HideWindow
//! - [x] SDL_MaximizeWindow
//! - [x] SDL_MinimizeWindow
//! - [x] SDL_RaiseWindow
//! - [x] SDL_RestoreWindow
//! - [x] SDL_ScreenSaverEnabled
//! - [x] SDL_SetWindowAlwaysOnTop
//! - [x] SDL_SetWindowAspectRatio
//! - [x] SDL_SetWindowBordered
//! - [x] SDL_SetWindowFocusable
//! - [x] SDL_SetWindowFullscreen
//! - [x] SDL_SetWindowFullscreenMode
//! - [ ] SDL_SetWindowHitTest
//! - [x] SDL_SetWindowIcon
//! - [x] SDL_SetWindowKeyboardGrab
//! - [x] SDL_SetWindowMaximumSize
//! - [x] SDL_SetWindowMinimumSize
//! - [x] SDL_SetWindowModal
//! - [x] SDL_SetWindowMouseGrab
//! - [x] SDL_SetWindowMouseRect
//! - [x] SDL_SetWindowOpacity
//! - [x] SDL_SetWindowParent
//! - [x] SDL_SetWindowPosition
//! - [x] SDL_SetWindowProgressState
//! - [x] SDL_SetWindowProgressValue
//! - [x] SDL_SetWindowResizable
//! - [x] SDL_SetWindowShape
//! - [x] SDL_SetWindowSize
//! - [x] SDL_SetWindowSurfaceVSync
//! - [x] SDL_SetWindowTitle
//! - [x] SDL_ShowWindow
//! - [x] SDL_ShowWindowSystemMenu
//! - [x] SDL_SyncWindow
//! - [x] SDL_UpdateWindowSurface
//! - [x] SDL_UpdateWindowSurfaceRects
//! - [x] SDL_WindowHasSurface
//! - [x] SDL_GetRenderer
//! - [x] SDL_CreateWindowAndRenderer
//!
//! The remaining `[ ]` entries are OpenGL/EGL interop and the hit-testing
//! callback, which each need their own dedicated abstraction.

use crate::{
    Result,
    boxed::Box,
    display::Display,
    error::Error,
    impl_enum_transmute, mod_reexport,
    properties::{Properties, PropertiesHandle},
    rect::{PointI32, RectI32},
    renderer::{Renderer, RendererHandle},
    resource::Ref,
    surface::Surface,
    util::{c_ptr_to_str, opt2ptr, opt2res_map, to_result},
};

// doc-only
#[expect(unused_imports)]
use crate::event::Event;

use bitflags::bitflags;
use sdl3_sys::{
    pixels::SDL_PixelFormat,
    render::{SDL_CreateWindowAndRenderer, SDL_GetRenderer, SDL_Renderer},
    video::*,
};

use std::{
    ffi::{CStr, c_char},
    mem::{MaybeUninit, transmute},
    num::NonZero,
    ptr::NonNull,
};

mod_reexport!(builder);
mod_reexport!(properties);

bitflags! {
    /// The flags on a window.
    ///
    /// # Remarks
    ///
    /// These cover a lot of true/false, or on/off, window state. Some of it
    /// is immutable after being set at creation time, some of it can be
    /// changed on existing windows by the app, and some of it might be
    /// altered by the user or system outside of the app's control.
    #[derive(Clone, Copy)]
    #[doc(alias = "SDL_WindowFlags")]
    pub struct WindowFlags: u64 {
        /// Window is in fullscreen mode.
        const FULLSCREEN = SDL_WINDOW_FULLSCREEN.0;
        /// Window usable with an OpenGL context.
        const OPENGL = SDL_WINDOW_OPENGL.0;
        /// Window is occluded.
        const OCCLUDED = SDL_WINDOW_OCCLUDED.0;
        /// Window is neither mapped onto the desktop nor shown in the
        /// taskbar/dock/window list; calling [`show`](WindowHandle::show) is required
        /// for it to become visible.
        const HIDDEN = SDL_WINDOW_HIDDEN.0;
        /// No window decoration.
        const BORDERLESS = SDL_WINDOW_BORDERLESS.0;
        /// Window can be resized.
        const RESIZABLE = SDL_WINDOW_RESIZABLE.0;
        /// Window is minimized.
        const MINIMIZED = SDL_WINDOW_MINIMIZED.0;
        /// Window is maximized.
        const MAXIMIZED = SDL_WINDOW_MAXIMIZED.0;
        /// Window has grabbed mouse input.
        const MOUSE_GRABBED = SDL_WINDOW_MOUSE_GRABBED.0;
        /// Window has input focus.
        const INPUT_FOCUS = SDL_WINDOW_INPUT_FOCUS.0;
        /// Window has mouse focus.
        const MOUSE_FOCUS = SDL_WINDOW_MOUSE_FOCUS.0;
        /// Window not created by SDL.
        const EXTERNAL = SDL_WINDOW_EXTERNAL.0;
        /// Window is modal.
        const MODAL = SDL_WINDOW_MODAL.0;
        /// Window uses a high pixel density back buffer if possible.
        const HIGH_PIXEL_DENSITY = SDL_WINDOW_HIGH_PIXEL_DENSITY.0;
        /// Window has mouse captured (unrelated to [`MouseGrabbed`](WindowFlags::MouseGrabbed)).
        const MOUSE_CAPTURE = SDL_WINDOW_MOUSE_CAPTURE.0;
        /// Window has relative mode enabled.
        const MOUSE_RELATIVE_MODE = SDL_WINDOW_MOUSE_RELATIVE_MODE.0;
        /// Window should always be above others.
        const ALWAYS_ON_TOP = SDL_WINDOW_ALWAYS_ON_TOP.0;
        /// Window should be treated as a utility window, not showing in the task bar
        /// and window list.
        const UTILITY = SDL_WINDOW_UTILITY.0;
        /// Window should be treated as a tooltip and does not get mouse or keyboard
        /// focus; requires a parent window.
        const TOOLTIP = SDL_WINDOW_TOOLTIP.0;
        /// Window should be treated as a popup menu; requires a parent window.
        const POPUP_MENU = SDL_WINDOW_POPUP_MENU.0;
        /// Window has grabbed keyboard input.
        const KEYBOARD_GRABBED = SDL_WINDOW_KEYBOARD_GRABBED.0;
        /// Window is in fill-document mode (Emscripten only).
        const FILL_DOCUMENT = SDL_WINDOW_FILL_DOCUMENT.0;
        /// Window usable for a Vulkan surface.
        const VULKAN = SDL_WINDOW_VULKAN.0;
        /// Window usable for a Metal view.
        const METAL = SDL_WINDOW_METAL.0;
        /// Window with a transparent buffer.
        const TRANSPARENT = SDL_WINDOW_TRANSPARENT.0;
        /// Window should not be focusable.
        const NOT_FOCUSABLE = SDL_WINDOW_NOT_FOCUSABLE.0;
    }
}

/// System theme.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemTheme {
    /// Unknown system theme.
    Unknown = SDL_SystemTheme::UNKNOWN.0,
    /// Light colored system theme.
    Light = SDL_SystemTheme::LIGHT.0,
    /// Dark colored system theme.
    Dark = SDL_SystemTheme::DARK.0,
}

/// Window progress state.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressState {
    /// No progress bar is shown.
    None = SDL_ProgressState::NONE.0,
    /// The progress bar is shown in an indeterminate state.
    Indeterminate = SDL_ProgressState::INDETERMINATE.0,
    /// The progress bar is shown in a normal state.
    Normal = SDL_ProgressState::NORMAL.0,
    /// The progress bar is shown in a paused state.
    Paused = SDL_ProgressState::PAUSED.0,
    /// The progress bar is shown in a state indicating the application had
    /// an error.
    Error = SDL_ProgressState::ERROR.0,
}

impl_enum_transmute!(SDL_WindowFlags, WindowFlags);
impl_enum_transmute!(SDL_SystemTheme, SystemTheme);
impl_enum_transmute!(SDL_ProgressState, ProgressState);

impl std::fmt::Display for SystemTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

#[derive(Clone, Copy)]
pub struct WindowId {
    inner: NonZero<u32>,
}

impl WindowId {
    fn from_raw(raw: u32) -> Result<Self> {
        opt2res_map(NonZero::new(raw), |inner| Self { inner })
    }

    const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        let inner = unsafe { NonZero::new_unchecked(raw) };
        Self { inner }
    }

    const fn as_sdl(self) -> SDL_WindowID {
        SDL_WindowID(self.inner.get())
    }
}

pub use crate::generated::{Window, WindowHandle};

/// Get the number of video drivers compiled into SDL.
#[doc(alias = "SDL_GetNumVideoDrivers")]
pub fn num_video_drivers() -> i32 {
    unsafe { SDL_GetNumVideoDrivers() }
}

/// Get the name of a built in video driver.
///
/// Returns [`None`] if `index` is out of bounds.
///
/// # Remarks
///
/// The video drivers are presented in the order in which they are normally
/// checked during initialization.
///
/// The names of drivers are all simple, low-ASCII identifiers, like
/// "cocoa", "x11" or "windows". These never have Unicode characters, and
/// are not meant to be proper names.
#[doc(alias = "SDL_GetVideoDriver")]
pub fn video_driver(index: i32) -> Option<&'static str> {
    let ptr = unsafe { SDL_GetVideoDriver(index) };

    if ptr.is_null() {
        None
    } else {
        // SAFETY: Video driver names are valid UTF-8 and stored statically.
        Some(unsafe { c_ptr_to_str(ptr) })
    }
}

/// Get the name of the currently initialized video driver.
///
/// Returns [`None`] if no driver has been initialized.
///
/// # Remarks
///
/// The names of drivers are all simple, low-ASCII identifiers, like
/// "cocoa", "x11" or "windows". These never have Unicode characters, and
/// are not meant to be proper names.
#[doc(alias = "SDL_GetCurrentVideoDriver")]
pub fn current_video_driver() -> Option<&'static str> {
    let ptr = unsafe { SDL_GetCurrentVideoDriver() };

    if ptr.is_null() {
        None
    } else {
        // SAFETY: Video driver names are valid UTF-8 and stored statically.
        Some(unsafe { c_ptr_to_str(ptr) })
    }
}

/// Get the current system theme.
///
/// Returns the current system theme: light, dark, or unknown.
#[doc(alias = "SDL_GetSystemTheme")]
pub fn system_theme() -> SystemTheme {
    // SAFETY: `SystemTheme` has the same representation as `SDL_SystemTheme`.
    unsafe { transmute(SDL_GetSystemTheme()) }
}

/// Get the window that currently has an input grab enabled.
///
/// Returns [`None`] if input is not grabbed.
#[doc(alias = "SDL_GetGrabbedWindow")]
pub fn grabbed_window() -> Option<WindowHandle> {
    WindowHandle::from_ptr(unsafe { SDL_GetGrabbedWindow() })
}

/// Get a list of valid windows.
#[doc(alias = "SDL_GetWindows")]
pub fn windows() -> Result<Box<[WindowHandle]>> {
    let mut count = MaybeUninit::uninit();
    let ptr = unsafe { SDL_GetWindows(count.as_mut_ptr()) };

    // SAFETY: On success, SDL allocates `count` window pointers. `WindowHandle`
    // is a `Copy` wrapper around `NonNull<SDL_Window>`, which has the same size
    // and alignment as `*mut SDL_Window`.
    unsafe { Box::from_raw_parts_nullck(ptr.cast(), count.assume_init() as _) }
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
pub fn screen_saver_enabled() -> bool {
    unsafe { SDL_ScreenSaverEnabled() }
}

/// Allow the screen to be blanked by a screen saver.
#[doc(alias = "SDL_EnableScreenSaver")]
pub fn enable_screen_saver() -> Result<()> {
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
pub fn disable_screen_saver() -> Result<()> {
    to_result(unsafe { SDL_DisableScreenSaver() })
}

impl WindowHandle {
    /// Block until any pending window state is finalized.
    ///
    /// # Remarks
    ///
    /// On asynchronous windowing systems, this acts as a synchronization
    /// barrier for pending window state. It will attempt to wait until any
    /// pending window state has been applied and is guaranteed to return
    /// within finite time. Note that for how long it can potentially block
    /// depends on the underlying window system, as window state changes may
    /// involve somewhat lengthy animations that must complete before the
    /// window is in its final requested state.
    ///
    /// On windowing systems where changes are immediate, this does nothing.
    #[doc(alias = "SDL_SyncWindow")]
    pub fn sync(&self) -> Result<()> {
        to_result(unsafe { SDL_SyncWindow(self.handle.as_ptr()) })
    }

    /// Request a window to demand attention from the user.
    #[doc(alias = "SDL_FlashWindow")]
    pub fn flash(&self, op: SDL_FlashOperation) -> Result<()> {
        to_result(unsafe { SDL_FlashWindow(self.handle.as_ptr(), op) })
    }

    /// Get the size of a window's client area.
    ///
    /// # Remarks
    ///
    /// The window pixel size may differ from its window coordinate size if
    /// the window is on a high pixel density display. Use
    /// [`WindowHandle::size_in_pixels`] or
    /// [`RendererHandle::output_size`] to get the real client area size in
    /// pixels.
    #[doc(alias = "SDL_GetWindowSize")]
    pub fn size(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetWindowSize(self.handle.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the size of a window's client area, in pixels.
    #[doc(alias = "SDL_GetWindowSizeInPixels")]
    pub fn size_in_pixels(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetWindowSizeInPixels(self.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the minimum size of a window's client area.
    #[doc(alias = "SDL_GetWindowMinimumSize")]
    pub fn minimum_size(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetWindowMinimumSize(self.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the maximum size of a window's client area.
    #[doc(alias = "SDL_GetWindowMaximumSize")]
    pub fn maximum_size(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetWindowMaximumSize(self.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the position of a window.
    ///
    /// # Remarks
    ///
    /// This is the current position of the window as last reported by the
    /// windowing system.
    #[doc(alias = "SDL_GetWindowPosition")]
    pub fn position(&self) -> PointI32 {
        let mut ret = MaybeUninit::<PointI32>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            SDL_GetWindowPosition(self.handle.as_ptr(), &raw mut (*ptr).x, &raw mut (*ptr).y);
            ret.assume_init()
        }
    }

    /// Get the title of a window, in UTF-8 format.
    ///
    /// The returned pointer is managed by SDL; it is empty ("") if there is
    /// no title.
    #[doc(alias = "SDL_GetWindowTitle")]
    pub fn title(&self) -> NonNull<c_char> {
        unsafe {
            NonNull::new(SDL_GetWindowTitle(self.handle.as_ptr()).cast_mut()).unwrap_unchecked()
        }
    }

    /// Get the window flags.
    ///
    /// Returns a mask of the [`WindowFlags`] associated with this window.
    #[doc(alias = "SDL_GetWindowFlags")]
    pub fn flags(&self) -> WindowFlags {
        unsafe { SDL_GetWindowFlags(self.handle.as_ptr()) }.into()
    }

    /// Get the renderer associated with a window.
    ///
    /// Returns [`None`] on failure.
    #[doc(alias = "SDL_GetRenderer")]
    pub fn renderer(&self) -> Option<RendererHandle> {
        RendererHandle::from_ptr(unsafe { SDL_GetRenderer(self.handle.as_ptr()) })
    }

    /// Get the display associated with a window.
    ///
    /// Returns the display containing the center of the window.
    #[doc(alias = "SDL_GetDisplayForWindow")]
    pub fn display(&self) -> Result<Display> {
        let raw = unsafe { SDL_GetDisplayForWindow(self.handle.as_ptr()) };
        Display::from_sdl(raw)
    }

    /// Get the parent of a window.
    ///
    /// Returns [`None`] if the window has no parent.
    #[doc(alias = "SDL_GetWindowParent")]
    pub fn parent(&self) -> Option<WindowHandle> {
        WindowHandle::from_ptr(unsafe { SDL_GetWindowParent(self.as_ptr()) })
    }

    /// Get the pixel density of a window.
    ///
    /// # Remarks
    ///
    /// This is a ratio of pixel size to window size. For example, if the
    /// window is 1920x1080 and it has a high density back buffer of
    /// 3840x2160 pixels, it would have a pixel density of 2.0.
    #[doc(alias = "SDL_GetWindowPixelDensity")]
    pub fn pixel_density(&self) -> Result<f32> {
        let ret = unsafe { SDL_GetWindowPixelDensity(self.as_ptr()) };
        if ret == 0. {
            Err(Error::current())
        } else {
            Ok(ret)
        }
    }

    /// Get the content display scale relative to a window's pixel size.
    ///
    /// # Remarks
    ///
    /// This is a combination of the window pixel density and the display
    /// content scale, and is the expected scale for displaying content in
    /// this window. For example, if a 3840x2160 window had a display scale
    /// of 2.0, the user expects the content to take twice as many pixels
    /// and be the same physical size as if it were being displayed in a
    /// 1920x1080 window with a display scale of 1.0.
    ///
    /// Conceptually this value corresponds to the scale display setting,
    /// and is updated when that setting is changed, or the window moves to
    /// a display with a different scale setting.
    #[doc(alias = "SDL_GetWindowDisplayScale")]
    pub fn display_scale(&self) -> Result<f32> {
        let ret = unsafe { SDL_GetWindowDisplayScale(self.as_ptr()) };
        if ret == 0. {
            Err(Error::current())
        } else {
            Ok(ret)
        }
    }

    /// Get the opacity of a window.
    ///
    /// Returns the opacity, from 0.0 (transparent) to 1.0 (opaque).
    ///
    /// # Remarks
    ///
    /// If transparency isn't supported on this platform, opacity will be
    /// returned as 1.0 without error.
    #[doc(alias = "SDL_GetWindowOpacity")]
    pub fn opacity(&self) -> Result<f32> {
        let ret = unsafe { SDL_GetWindowOpacity(self.as_ptr()) };
        if ret < 0. {
            Err(Error::current())
        } else {
            Ok(ret)
        }
    }

    /// Get the pixel format associated with the window.
    #[doc(alias = "SDL_GetWindowPixelFormat")]
    pub fn pixel_format(&self) -> SDL_PixelFormat {
        unsafe { SDL_GetWindowPixelFormat(self.as_ptr()) }
    }

    /// Query the display mode to use when a window is visible at fullscreen.
    ///
    /// Returns the exclusive fullscreen mode to use, or [`None`] for
    /// borderless fullscreen desktop mode.
    ///
    /// The returned pointer is managed by SDL and is valid for as long as the
    /// window is fullscreen.
    #[doc(alias = "SDL_GetWindowFullscreenMode")]
    pub fn fullscreen_mode(&self) -> Option<NonNull<SDL_DisplayMode>> {
        NonNull::new(unsafe { SDL_GetWindowFullscreenMode(self.as_ptr()) }.cast_mut())
    }

    /// Get the raw ICC profile data for the screen the window is currently
    /// on.
    #[doc(alias = "SDL_GetWindowICCProfile")]
    pub fn icc_profile(&self) -> Result<Box<[u8]>> {
        let mut size = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetWindowICCProfile(self.as_ptr(), size.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `size` bytes.
        unsafe { Box::from_raw_parts_nullck(ptr.cast(), size.assume_init()) }
    }

    /// Get the aspect ratio of a window's client area.
    ///
    /// Returns `(min_aspect, max_aspect)`.
    #[doc(alias = "SDL_GetWindowAspectRatio")]
    pub fn aspect_ratio(&self) -> Result<(f32, f32)> {
        let mut min = MaybeUninit::uninit();
        let mut max = MaybeUninit::uninit();

        unsafe {
            if SDL_GetWindowAspectRatio(self.as_ptr(), min.as_mut_ptr(), max.as_mut_ptr()) {
                Ok((min.assume_init(), max.assume_init()))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get the size of a window's borders (decorations) around the client
    /// area.
    ///
    /// Returns the border sizes in the order `(top, left, bottom, right)`.
    ///
    /// # Remarks
    ///
    /// If this function fails, the size values are initialized to 0, as if
    /// the window in question was borderless.
    ///
    /// This function may fail on systems where the window has not yet been
    /// decorated by the display server (for example, immediately after
    /// creation). It is recommended that you wait at least until the window
    /// has been presented and composited, so that the window system has a
    /// chance to decorate the window and provide the border dimensions to
    /// SDL.
    ///
    /// This function also fails if getting the information is not supported.
    #[doc(alias = "SDL_GetWindowBordersSize")]
    pub fn borders_size(&self) -> Result<(i32, i32, i32, i32)> {
        let mut top = MaybeUninit::uninit();
        let mut left = MaybeUninit::uninit();
        let mut bottom = MaybeUninit::uninit();
        let mut right = MaybeUninit::uninit();

        unsafe {
            if SDL_GetWindowBordersSize(
                self.as_ptr(),
                top.as_mut_ptr(),
                left.as_mut_ptr(),
                bottom.as_mut_ptr(),
                right.as_mut_ptr(),
            ) {
                Ok((
                    top.assume_init(),
                    left.assume_init(),
                    bottom.assume_init(),
                    right.assume_init(),
                ))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get the safe area for this window.
    ///
    /// # Remarks
    ///
    /// Some devices have portions of the screen which are partially obscured
    /// or not interactive, possibly due to on-screen controls, curved edges,
    /// camera notches, TV overscan, etc. This function provides the area of
    /// the window which is safe to have interactable content. You should
    /// continue rendering into the rest of the window, but it should not
    /// contain visually important or interactable content.
    #[doc(alias = "SDL_GetWindowSafeArea")]
    pub fn safe_area(&self) -> Result<RectI32> {
        let mut ret = MaybeUninit::uninit();

        unsafe {
            if SDL_GetWindowSafeArea(self.as_ptr(), ret.as_mut_ptr()) {
                Ok(std::mem::transmute_copy(ret.assume_init_ref()))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get the mouse confinement rectangle of a window.
    ///
    /// Returns [`None`] if there isn't one.
    ///
    /// The returned pointer is managed by SDL and is owned by the window.
    #[doc(alias = "SDL_GetWindowMouseRect")]
    pub fn mouse_rect(&self) -> Option<NonNull<RectI32>> {
        NonNull::new(unsafe { SDL_GetWindowMouseRect(self.as_ptr()).cast::<RectI32>() }.cast_mut())
    }

    /// Get the SDL surface associated with the window.
    ///
    /// # Remarks
    ///
    /// A new surface will be created with the optimal format for the window,
    /// if necessary. This surface will be freed when the window is destroyed.
    /// Do not free this surface.
    ///
    /// This surface will be invalidated if the window is resized. After
    /// resizing a window this function must be called again to return a valid
    /// surface.
    ///
    /// You may not combine this with 3D or the rendering API on this window.
    ///
    /// This function is affected by `SDL_HINT_FRAMEBUFFER_ACCELERATION`.
    #[doc(alias = "SDL_GetWindowSurface")]
    pub fn surface(&self) -> Result<Surface> {
        Surface::from_ptr(unsafe { SDL_GetWindowSurface(self.as_ptr()) })
    }

    /// Get VSync for the window surface.
    ///
    /// Returns the current vertical refresh sync interval. See
    /// [`WindowHandle::set_surface_vsync`] for the meaning of the value.
    #[doc(alias = "SDL_GetWindowSurfaceVSync")]
    pub fn surface_vsync(&self) -> Result<i32> {
        let mut ret = MaybeUninit::uninit();

        unsafe {
            if SDL_GetWindowSurfaceVSync(self.as_ptr(), ret.as_mut_ptr()) {
                Ok(ret.assume_init())
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get a window's keyboard grab mode.
    #[doc(alias = "SDL_GetWindowKeyboardGrab")]
    pub fn keyboard_grabbed(&self) -> bool {
        unsafe { SDL_GetWindowKeyboardGrab(self.as_ptr()) }
    }

    /// Get a window's mouse grab mode.
    #[doc(alias = "SDL_GetWindowMouseGrab")]
    pub fn mouse_grabbed(&self) -> bool {
        unsafe { SDL_GetWindowMouseGrab(self.as_ptr()) }
    }

    /// Get the state of the progress bar for the given window's taskbar
    /// icon.
    #[doc(alias = "SDL_GetWindowProgressState")]
    pub fn progress_state(&self) -> Result<ProgressState> {
        let ps = unsafe { SDL_GetWindowProgressState(self.as_ptr()) };
        if ps == SDL_ProgressState::INVALID {
            Err(Error::current())
        } else {
            type Src = SDL_ProgressState;
            type Dst = ProgressState;

            Ok(unsafe { transmute::<Src, Dst>(ps) })
        }
    }

    /// Get the value of the progress bar for the given window's taskbar
    /// icon.
    ///
    /// Returns the progress value in the range of `[0.0 - 1.0]`, or `-1.0`
    /// on failure.
    #[doc(alias = "SDL_GetWindowProgressValue")]
    pub fn progress_value(&self) -> f32 {
        unsafe { SDL_GetWindowProgressValue(self.as_ptr()) }
    }

    /// Return whether the window has a surface associated with it.
    #[doc(alias = "SDL_WindowHasSurface")]
    pub fn has_surface(&self) -> bool {
        unsafe { SDL_WindowHasSurface(self.as_ptr()) }
    }

    /// Request that the size of a window's client area be set.
    ///
    /// # Remarks
    ///
    /// If the window is in a fullscreen or maximized state, this request has
    /// no effect.
    ///
    /// To change the exclusive fullscreen mode of a window, use
    /// [`WindowHandle::set_fullscreen_mode`].
    ///
    /// On some windowing systems, this request is asynchronous and the new
    /// window size may not have been applied immediately upon the return of
    /// this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window size changes, an [`Event::WindowResized`] event will
    /// be emitted with the new window dimensions. Note that the new
    /// dimensions may not match the exact size requested, as some windowing
    /// systems can restrict the window size in certain scenarios
    /// (e.g. constraining the size of the content area to remain within the
    /// usable desktop bounds). Additionally, as this is just a request, it
    /// can be denied by the windowing system.
    #[doc(alias = "SDL_SetWindowSize")]
    pub fn set_size(&self, size: PointI32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowSize(self.as_ptr(), size.x, size.y) })
    }

    /// Set the minimum size of a window's client area.
    ///
    /// A dimension of 0 means no limit.
    #[doc(alias = "SDL_SetWindowMinimumSize")]
    pub fn set_min_size(&self, size: PointI32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowMinimumSize(self.as_ptr(), size.x, size.y) })
    }

    /// Set the maximum size of a window's client area.
    ///
    /// A dimension of 0 means no limit.
    #[doc(alias = "SDL_SetWindowMaximumSize")]
    pub fn set_max_size(&self, size: PointI32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowMaximumSize(self.as_ptr(), size.x, size.y) })
    }

    /// Request that the window's position be set.
    ///
    /// Coordinates may be [`Window::POS_CENTERED`] or
    /// [`Window::POS_UNDEFINED`].
    ///
    /// # Remarks
    ///
    /// If the window is in an exclusive fullscreen or maximized state, this
    /// request has no effect.
    ///
    /// This can be used to reposition fullscreen-desktop windows onto a
    /// different display, however, as exclusive fullscreen windows are
    /// locked to a specific display, they can only be repositioned
    /// programmatically via [`WindowHandle::set_fullscreen_mode`].
    ///
    /// On some windowing systems this request is asynchronous and the new
    /// coordinates may not have been applied immediately upon the return of
    /// this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window position changes, an [`Event::WindowMoved`] event
    /// will be emitted with the window's new coordinates. Note that the new
    /// coordinates may not match the exact coordinates requested, as some
    /// windowing systems can restrict the position of the window in certain
    /// scenarios. Additionally, as this is just a request, it can be denied
    /// by the windowing system.
    #[doc(alias = "SDL_SetWindowPosition")]
    pub fn set_pos(&self, pos: PointI32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowPosition(self.as_ptr(), pos.x, pos.y) })
    }

    /// Set the title of a window, in UTF-8 encoding.
    #[doc(alias = "SDL_SetWindowTitle")]
    pub fn set_title(&self, title: &CStr) -> Result<()> {
        to_result(unsafe { SDL_SetWindowTitle(self.as_ptr(), title.as_ptr()) })
    }

    /// Set the icon for a window.
    ///
    /// # Remarks
    ///
    /// If this function is passed a surface with alternate representations
    /// added using `SDL_AddSurfaceAlternateImage`, the surface will be
    /// interpreted as the content to be used for 100% display scale, and the
    /// alternate representations will be used for high DPI situations. For
    /// example, if the original surface is 32x32, then on a 2x macOS display
    /// or 200% display scale on Windows, a 64x64 version of the image will
    /// be used, if available. If a matching version of the image isn't
    /// available, the closest larger size image will be downscaled to the
    /// appropriate size and be used instead, if available. Otherwise, the
    /// closest smaller image will be upscaled and be used instead.
    #[doc(alias = "SDL_SetWindowIcon")]
    pub fn set_icon(&self, icon: Ref<Surface>) -> Result<()> {
        to_result(unsafe { SDL_SetWindowIcon(self.as_ptr(), icon.handle.as_ptr()) })
    }

    /// Set the shape of a transparent window.
    ///
    /// # Remarks
    ///
    /// This sets the alpha channel of a transparent window and any fully
    /// transparent areas are also transparent to mouse clicks. If you are
    /// using something besides the SDL render API, then you are responsible
    /// for drawing the alpha channel of the window to match the shape alpha
    /// channel to get consistent cross-platform results.
    ///
    /// The shape is copied inside this function, so you can free it
    /// afterwards. If your shape surface changes, you should call this
    /// function again to update the window. This is an expensive operation,
    /// so should be done sparingly.
    ///
    /// The window must have been created with the `SDL_WINDOW_TRANSPARENT`
    /// flag ([`WindowFlags::TRANSPARENT`]).
    #[doc(alias = "SDL_SetWindowShape")]
    pub fn set_shape(&self, shape: Ref<Surface>) -> Result<()> {
        to_result(unsafe { SDL_SetWindowShape(self.as_ptr(), shape.handle.as_ptr()) })
    }

    /// Request that the aspect ratio of a window's client area be set.
    ///
    /// A limit of `0.0` means no limit.
    ///
    /// # Remarks
    ///
    /// The aspect ratio is the ratio of width divided by height, e.g.
    /// 2560x1600 would be 1.6. Larger aspect ratios are wider and smaller
    /// aspect ratios are narrower.
    ///
    /// If, at the time of this request, the window is in a fixed-size state,
    /// such as maximized or fullscreen, the request will be deferred until
    /// the window exits this state and becomes resizable again.
    ///
    /// On some windowing systems, this request is asynchronous and the new
    /// window aspect ratio may not have been applied immediately upon the
    /// return of this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    #[doc(alias = "SDL_SetWindowAspectRatio")]
    pub fn set_aspect_ratio(&self, min: f32, max: f32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowAspectRatio(self.as_ptr(), min, max) })
    }

    /// Set the border state of a window.
    ///
    /// # Remarks
    ///
    /// This will add or remove the window's `SDL_WINDOW_BORDERLESS` flag and
    /// add or remove the border from the actual window. This is a no-op if
    /// the window's border already matches the requested state.
    ///
    /// You can't change the border state of a fullscreen window.
    #[doc(alias = "SDL_SetWindowBordered")]
    pub fn set_bordered(&self, bordered: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowBordered(self.as_ptr(), bordered) })
    }

    /// Set the user-resizable state of a window.
    ///
    /// # Remarks
    ///
    /// This will add or remove the window's `SDL_WINDOW_RESIZABLE` flag and
    /// allow/disallow user resizing of the window. This is a no-op if the
    /// window's resizable state already matches the requested state.
    ///
    /// You can't change the resizable state of a fullscreen window.
    #[doc(alias = "SDL_SetWindowResizable")]
    pub fn set_resizable(&self, value: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowResizable(self.handle.as_ptr(), value) })
    }

    /// Set the window to always be above the others.
    ///
    /// # Remarks
    ///
    /// This will add or remove the window's `SDL_WINDOW_ALWAYS_ON_TOP` flag.
    /// This will bring the window to the front and keep the window above
    /// the rest.
    #[doc(alias = "SDL_SetWindowAlwaysOnTop")]
    pub fn set_always_on_top(&self, on_top: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowAlwaysOnTop(self.as_ptr(), on_top) })
    }

    /// Request that the window's fullscreen state be changed.
    ///
    /// # Remarks
    ///
    /// By default a window in fullscreen state uses borderless fullscreen
    /// desktop mode, but a specific exclusive display mode can be set using
    /// [`WindowHandle::set_fullscreen_mode`].
    ///
    /// On some windowing systems this request is asynchronous and the new
    /// fullscreen state may not have been applied immediately upon the
    /// return of this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, an [`Event::WindowEnteredFullscreen`]
    /// or [`Event::WindowLeftFullscreen`] event will be emitted. Note
    /// that, as this is just a request, it can be denied by the windowing
    /// system.
    #[doc(alias = "SDL_SetWindowFullscreen")]
    pub fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowFullscreen(self.as_ptr(), fullscreen) })
    }

    /// Set the display mode to use when a window is visible and fullscreen.
    ///
    /// `mode` can be [`None`] for borderless fullscreen desktop mode, or one
    /// of the fullscreen modes returned by `SDL_GetFullscreenDisplayModes`
    /// to set an exclusive fullscreen mode.
    ///
    /// # Remarks
    ///
    /// This only affects the display mode used when the window is fullscreen.
    /// To change the window size when the window is not fullscreen, use
    /// [`WindowHandle::set_size`].
    ///
    /// If the window is currently in the fullscreen state, this request is
    /// asynchronous on some windowing systems and the new mode dimensions may
    /// not be applied immediately upon the return of this function. If an
    /// immediate change is required, call [`WindowHandle::sync`] to block
    /// until the changes have taken effect.
    ///
    /// When the new mode takes effect, an [`Event::WindowResized`] and/or
    /// an [`Event::WindowPixelSizeChanged`] event will be emitted with
    /// the new mode dimensions.
    #[doc(alias = "SDL_SetWindowFullscreenMode")]
    pub fn set_fullscreen_mode(&self, mode: Option<&SDL_DisplayMode>) -> Result<()> {
        to_result(unsafe { SDL_SetWindowFullscreenMode(self.as_ptr(), opt2ptr(mode)) })
    }

    /// Set a window's keyboard grab mode.
    ///
    /// # Remarks
    ///
    /// Keyboard grab enables capture of system keyboard shortcuts like
    /// Alt+Tab or the Meta/Super key. Note that not all system keyboard
    /// shortcuts can be captured by applications (one example is
    /// Ctrl+Alt+Del on Windows).
    ///
    /// This is primarily intended for specialized applications such as VNC
    /// clients or VM frontends. Normal games should not use keyboard grab.
    ///
    /// When keyboard grab is enabled, SDL will continue to handle Alt+Tab
    /// when the window is full-screen to ensure the user is not trapped in
    /// your application. If you have a custom keyboard shortcut to exit
    /// fullscreen mode, you may suppress this behavior with
    /// `SDL_HINT_ALLOW_ALT_TAB_WHILE_GRABBED`.
    ///
    /// If the caller enables a grab while another window is currently
    /// grabbed, the other window loses its grab in favor of the caller's
    /// window.
    #[doc(alias = "SDL_SetWindowKeyboardGrab")]
    pub fn set_keyboard_grab(&self, grabbed: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowKeyboardGrab(self.as_ptr(), grabbed) })
    }

    /// Set a window's mouse grab mode.
    ///
    /// # Remarks
    ///
    /// Mouse grab confines the mouse cursor to the window.
    #[doc(alias = "SDL_SetWindowMouseGrab")]
    pub fn set_mouse_grab(&self, grabbed: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowMouseGrab(self.as_ptr(), grabbed) })
    }

    /// Confines the cursor to the specified area of a window.
    ///
    /// `rect` is a rectangle area in window-relative coordinates, or [`None`]
    /// to remove the confinement.
    ///
    /// # Remarks
    ///
    /// Note that this does NOT grab the cursor, it only defines the area a
    /// cursor is restricted to when the window has mouse focus.
    #[doc(alias = "SDL_SetWindowMouseRect")]
    pub fn set_mouse_rect(&self, rect: Option<&RectI32>) -> Result<()> {
        to_result(unsafe { SDL_SetWindowMouseRect(self.as_ptr(), opt2ptr(rect).cast()) })
    }

    /// Set the opacity for a window.
    ///
    /// `opacity` ranges from 0.0 (transparent) to 1.0 (opaque), and is
    /// clamped internally.
    ///
    /// This function fails if setting the opacity isn't supported.
    #[doc(alias = "SDL_SetWindowOpacity")]
    pub fn set_opacity(&self, opacity: f32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowOpacity(self.as_ptr(), opacity) })
    }

    /// Set the window as a child of a parent window.
    ///
    /// `parent` of [`None`] unparents the window and removes child window
    /// status.
    ///
    /// # Remarks
    ///
    /// If the window is already the child of an existing window, it will be
    /// reparented to the new owner.
    ///
    /// If a parent window is hidden or destroyed, the operation will be
    /// recursively applied to child windows. Child windows hidden with the
    /// parent that did not have their hidden status explicitly set will be
    /// restored when the parent is shown.
    ///
    /// Attempting to set the parent of a window that is currently in the
    /// modal state will fail. Use [`WindowHandle::set_modal`] to cancel the
    /// modal status before attempting to change the parent.
    ///
    /// Popup windows cannot change parents and attempts to do so will fail.
    ///
    /// Setting a parent window that is currently the sibling or descendent
    /// of the child window results in undefined behavior.
    #[doc(alias = "SDL_SetWindowParent")]
    pub fn set_parent(&self, parent: Option<Ref<Window>>) -> Result<()> {
        let ptr = parent.map_or(std::ptr::null_mut(), |p| p.handle.as_ptr());
        to_result(unsafe { SDL_SetWindowParent(self.as_ptr(), ptr) })
    }

    /// Toggle the state of the window as modal.
    ///
    /// # Remarks
    ///
    /// To enable modal status on a window, the window must currently be the
    /// child window of a parent, or toggling modal status on will fail.
    #[doc(alias = "SDL_SetWindowModal")]
    pub fn set_modal(&self, modal: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowModal(self.as_ptr(), modal) })
    }

    /// Set whether the window may have input focus.
    #[doc(alias = "SDL_SetWindowFocusable")]
    pub fn set_focusable(&self, focusable: bool) -> Result<()> {
        to_result(unsafe { SDL_SetWindowFocusable(self.as_ptr(), focusable) })
    }

    /// Display the system-level window menu.
    ///
    /// `pos` is relative to the origin (top-left) of the client area.
    ///
    /// # Remarks
    ///
    /// This default window menu is provided by the system and on some
    /// platforms provides functionality for setting or changing privileged
    /// state on the window, such as moving it between workspaces or displays,
    /// or toggling the always-on-top property.
    ///
    /// On platforms or desktops where this is unsupported, this function
    /// does nothing.
    #[doc(alias = "SDL_ShowWindowSystemMenu")]
    pub fn show_system_menu(&self, pos: PointI32) -> Result<()> {
        to_result(unsafe { SDL_ShowWindowSystemMenu(self.as_ptr(), pos.x, pos.y) })
    }

    /// Toggle VSync for the window surface.
    ///
    /// `vsync` can be `1` to synchronize present with every vertical refresh,
    /// `2` to synchronize present with every second vertical refresh, etc.,
    /// `-1` (`SDL_WINDOW_SURFACE_VSYNC_ADAPTIVE`) for late swap tearing
    /// (adaptive vsync), or `0`
    /// (`SDL_WINDOW_SURFACE_VSYNC_DISABLED`) to disable.
    ///
    /// Not every value is supported by every driver, so you should check the
    /// return value to see whether the requested setting is supported.
    ///
    /// # Remarks
    ///
    /// When a window surface is created, vsync defaults to disabled.
    #[doc(alias = "SDL_SetWindowSurfaceVSync")]
    pub fn set_surface_vsync(&self, vsync: i32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowSurfaceVSync(self.as_ptr(), vsync) })
    }

    /// Copy the window surface to the screen.
    ///
    /// This is the function you use to reflect any changes to the surface on
    /// the screen.
    #[doc(alias = "SDL_UpdateWindowSurface")]
    pub fn update_surface(&self) -> Result<()> {
        to_result(unsafe { SDL_UpdateWindowSurface(self.as_ptr()) })
    }

    /// Copy areas of the window surface to the screen.
    ///
    /// # Remarks
    ///
    /// This is the function you use to reflect changes to portions of the
    /// surface on the screen.
    ///
    /// Note that this function will update *at least* the rectangles
    /// specified, but this is only intended as an optimization; in practice,
    /// this might update more of the screen (or all of the screen!),
    /// depending on what method SDL uses to send pixels to the system.
    #[doc(alias = "SDL_UpdateWindowSurfaceRects")]
    pub fn update_surface_rects(&self, rects: &[RectI32]) -> Result<()> {
        to_result(unsafe {
            SDL_UpdateWindowSurfaceRects(self.as_ptr(), rects.as_ptr().cast(), rects.len() as i32)
        })
    }

    /// Destroy the surface associated with the window.
    #[doc(alias = "SDL_DestroyWindowSurface")]
    pub fn destroy_surface(&self) -> Result<()> {
        to_result(unsafe { SDL_DestroyWindowSurface(self.as_ptr()) })
    }

    /// Show a window.
    #[doc(alias = "SDL_ShowWindow")]
    pub fn show(&self) -> Result<()> {
        to_result(unsafe { SDL_ShowWindow(self.as_ptr()) })
    }

    /// Hide a window.
    #[doc(alias = "SDL_HideWindow")]
    pub fn hide(&self) -> Result<()> {
        to_result(unsafe { SDL_HideWindow(self.as_ptr()) })
    }

    /// Request that a window be raised above other windows and gain the
    /// input focus.
    ///
    /// # Remarks
    ///
    /// The result of this request is subject to desktop window manager
    /// policy, particularly if raising the requested window would result in
    /// stealing focus from another application. If the window is
    /// successfully raised and gains input focus, an
    /// [`Event::WindowFocusGained`] event will be emitted, and the window
    /// will have the `SDL_WINDOW_INPUT_FOCUS` flag set.
    #[doc(alias = "SDL_RaiseWindow")]
    pub fn raise(&self) -> Result<()> {
        to_result(unsafe { SDL_RaiseWindow(self.as_ptr()) })
    }

    /// Request that the window be made as large as possible.
    ///
    /// # Remarks
    ///
    /// Non-resizable windows can't be maximized. The window must have the
    /// `SDL_WINDOW_RESIZABLE` flag set, or this will have no effect.
    ///
    /// On some windowing systems this request is asynchronous and the new
    /// window state may not have been applied immediately upon the return of
    /// this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, an [`Event::WindowMaximized`] event
    /// will be emitted. Note that, as this is just a request, the windowing
    /// system can deny the state change.
    ///
    /// When maximizing a window, whether the constraints set via
    /// [`WindowHandle::set_max_size`] are honored depends on the policy of
    /// the window manager. Win32 and macOS enforce the constraints when
    /// maximizing, while X11 and Wayland window managers may vary.
    #[doc(alias = "SDL_MaximizeWindow")]
    pub fn maximize(&self) -> Result<()> {
        to_result(unsafe { SDL_MaximizeWindow(self.as_ptr()) })
    }

    /// Request that the window be minimized to an iconic representation.
    ///
    /// # Remarks
    ///
    /// If the window is in a fullscreen state, this request has no direct
    /// effect. It may alter the state the window is returned to when leaving
    /// fullscreen.
    ///
    /// On some windowing systems this request is asynchronous and the new
    /// window state may not have been applied immediately upon the return of
    /// this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, an [`Event::WindowMinimized`] event
    /// will be emitted. Note that, as this is just a request, the windowing
    /// system can deny the state change.
    #[doc(alias = "SDL_MinimizeWindow")]
    pub fn minimize(&self) -> Result<()> {
        to_result(unsafe { SDL_MinimizeWindow(self.as_ptr()) })
    }

    /// Request that the size and position of a minimized or maximized window
    /// be restored.
    ///
    /// # Remarks
    ///
    /// If the window is in a fullscreen state, this request has no direct
    /// effect. It may alter the state the window is returned to when leaving
    /// fullscreen.
    ///
    /// On some windowing systems this request is asynchronous and the new
    /// window state may not have been applied immediately upon the return of
    /// this function. If an immediate change is required, call
    /// [`WindowHandle::sync`] to block until the changes have taken effect.
    ///
    /// When the window state changes, an [`Event::WindowRestored`] event
    /// will be emitted. Note that, as this is just a request, the windowing
    /// system can deny the state change.
    #[doc(alias = "SDL_RestoreWindow")]
    pub fn restore(&self) -> Result<()> {
        to_result(unsafe { SDL_RestoreWindow(self.as_ptr()) })
    }

    /// Set the state of the progress bar for the given window's taskbar
    /// icon.
    ///
    /// [`ProgressState::None`] stops displaying the progress bar.
    #[doc(alias = "SDL_SetWindowProgressState")]
    pub fn set_progress_state(&self, state: ProgressState) -> Result<()> {
        type Src = ProgressState;
        type Dst = SDL_ProgressState;

        to_result(unsafe {
            SDL_SetWindowProgressState(self.as_ptr(), transmute::<Src, Dst>(state))
        })
    }

    /// Set the value of the progress bar for the given window's taskbar
    /// icon.
    ///
    /// `value` must be in the range `[0.0 - 1.0]`; values outside the valid
    /// range are clamped.
    #[doc(alias = "SDL_SetWindowProgressValue")]
    pub fn set_progress_value(&self, value: f32) -> Result<()> {
        to_result(unsafe { SDL_SetWindowProgressValue(self.as_ptr(), value) })
    }

    /// Get the properties associated with a window.
    ///
    /// Read-only properties of this window, as documented by
    /// [`SDL_GetWindowProperties`](https://wiki.libsdl.org/SDL3/SDL_GetWindowProperties).
    ///
    /// Covers the generic properties plus the Cocoa, Win32, X11 and Wayland
    /// backends. Not covered: Android, iOS/UIKit, KMS/DRM, OpenVR, QNX,
    /// Vivante, Emscripten and visionOS, as well as
    /// `SDL_PROP_WINDOW_WAYLAND_WINDOW_ID_STRING`, which sdl3-sys does not
    /// expose.
    #[doc(alias = "SDL_GetWindowProperties")]
    pub fn properties(&self) -> WindowProperties<'_> {
        unsafe {
            let id = SDL_GetWindowProperties(self.handle.as_ptr());
            let handle = PropertiesHandle::from_id(id).unwrap_unchecked();
            let r = Ref::from_handle(handle);

            WindowProperties::new(r)
        }
    }
}

impl Window {
    /// Used to indicate that the window position should be centered.
    pub const POS_CENTERED: i32 = SDL_WINDOWPOS_CENTERED;
    /// Used to indicate that the window position is undefined.
    pub const POS_UNDEFINED: i32 = SDL_WINDOWPOS_UNDEFINED;

    /// Bind the builder to an existing property group.
    ///
    /// A single [`Properties`] can be shared between the window, renderer
    /// and GPU device builders, since their creation properties
    /// (`SDL_PROP_WINDOW_CREATE_*`, `SDL_PROP_RENDERER_CREATE_*`,
    /// `SDL_PROP_GPU_DEVICE_CREATE_*`) never collide with each other.
    /// They do collide with themselves, however: creating a second window
    /// from the same group inherits any leftover window properties, so use
    /// one [`Properties`] per window.
    pub fn builder(props: Ref<Properties>) -> WindowBuilder {
        WindowBuilder::new(props)
    }

    /// Create a window with the specified dimensions and flags.
    ///
    /// `title` is the title of the window, in UTF-8 encoding; `size` is its
    /// width and height; `flags` may be zero or more [`WindowFlags`] OR'd
    /// together.
    ///
    /// # Remarks
    ///
    /// The window size is a request and may be different than expected based
    /// on the desktop layout and window manager policies. Your application
    /// should be prepared to handle a window of any size.
    ///
    /// The window will be shown if [`WindowFlags::HIDDEN`] is not set. If
    /// hidden at creation time, [`WindowHandle::show`] can be used to show
    /// it later.
    ///
    /// On Apple's macOS, you **must** set the `NSHighResolutionCapable`
    /// Info.plist property to YES, otherwise you will not receive a High-DPI
    /// OpenGL canvas.
    ///
    /// The window pixel size may differ from its window coordinate size if
    /// the window is on a high pixel density display. Use
    /// [`WindowHandle::size`] to query the client area's size in window
    /// coordinates, and [`WindowHandle::size_in_pixels`] or
    /// [`RendererHandle::output_size`] to query the drawable size in pixels.
    /// Note that the drawable size can vary after the window is created and
    /// should be queried again if you get an
    /// [`Event::WindowPixelSizeChanged`] event.
    ///
    /// If the window is created with any of the [`WindowFlags::OPENGL`] or
    /// [`WindowFlags::VULKAN`] flags, then the corresponding LoadLibrary
    /// function (`SDL_GL_LoadLibrary` or `SDL_Vulkan_LoadLibrary`) is called
    /// and the corresponding UnloadLibrary function is called when the
    /// window is destroyed.
    ///
    /// If [`WindowFlags::VULKAN`] is specified and there isn't a working
    /// Vulkan driver, creation will fail, because `SDL_Vulkan_LoadLibrary`
    /// will fail.
    ///
    /// If [`WindowFlags::METAL`] is specified on an OS that does not support
    /// Metal, creation will fail.
    ///
    /// If you intend to use this window with a renderer, you should use
    /// [`Window::with_renderer`] instead of this function, to avoid window
    /// flicker.
    #[doc(alias = "SDL_CreateWindow")]
    pub fn new(title: &CStr, size: PointI32, flags: WindowFlags) -> Result<Self> {
        Self::from_ptr(unsafe { SDL_CreateWindow(title.as_ptr(), size.x, size.y, flags.into()) })
    }

    /// Create a child popup window of the specified parent window.
    ///
    /// `offset` is the position of the popup window relative to the origin
    /// of the parent, `size` is its width and height, and `flags` **must**
    /// contain at least one of [`WindowFlags::TOOLTIP`] or
    /// [`WindowFlags::POPUP_MENU`], plus zero or more additional flags.
    ///
    /// # Remarks
    ///
    /// The window size is a request and may be different than expected based
    /// on the desktop layout and window manager policies. Your application
    /// should be prepared to handle a window of any size.
    ///
    /// The following flags are not relevant to popup window creation and
    /// will be ignored: [`WindowFlags::MINIMIZED`],
    /// [`WindowFlags::MAXIMIZED`], [`WindowFlags::FULLSCREEN`],
    /// [`WindowFlags::BORDERLESS`].
    ///
    /// The following flags are incompatible with popup window creation and
    /// will cause it to fail: [`WindowFlags::UTILITY`],
    /// [`WindowFlags::MODAL`].
    ///
    /// The parent of a popup window can be either a regular, toplevel
    /// window, or another popup window.
    ///
    /// Popup windows cannot be minimized, maximized, made fullscreen,
    /// raised, flash, be made a modal window, be the parent of a toplevel
    /// window, or grab the mouse and/or keyboard. Attempts to do so will
    /// fail.
    ///
    /// Popup windows implicitly do not have a border/decorations and do not
    /// appear on the taskbar/dock or in lists of windows such as alt-tab
    /// menus.
    ///
    /// By default, popup window positions will automatically be constrained
    /// to keep the entire window within display bounds. This can be
    /// overridden with the `SDL_PROP_WINDOW_CREATE_CONSTRAIN_POPUP_BOOLEAN`
    /// property.
    ///
    /// By default, popup menus will automatically grab keyboard focus from
    /// the parent when shown. This behavior can be overridden by setting the
    /// `SDL_WINDOW_NOT_FOCUSABLE` flag, setting the
    /// `SDL_PROP_WINDOW_CREATE_FOCUSABLE_BOOLEAN` property to false, or
    /// toggling it after creation via [`WindowHandle::set_focusable`].
    ///
    /// If a parent window is hidden or destroyed, any child popup windows
    /// will be recursively hidden or destroyed as well. Child popup windows
    /// not explicitly hidden will be restored when the parent is shown.
    #[doc(alias = "SDL_CreatePopupWindow")]
    pub fn popup(
        parent: Ref<Window>,
        offset: PointI32,
        size: PointI32,
        flags: WindowFlags,
    ) -> Result<Self> {
        Self::from_ptr(unsafe {
            SDL_CreatePopupWindow(
                parent.handle.as_ptr(),
                offset.x,
                offset.y,
                size.x,
                size.y,
                flags.into(),
            )
        })
    }

    /// Create a window and default renderer.
    ///
    /// This is equivalent to calling [`Window::new`] and creating a default
    /// renderer for it; see [`Window::new`] for the meaning of the
    /// parameters and additional remarks.
    #[doc(alias = "SDL_CreateWindowAndRenderer")]
    pub fn with_renderer(
        title: &CStr,
        size: PointI32,
        flags: WindowFlags,
    ) -> Result<(Self, Renderer)> {
        let mut ret = MaybeUninit::<(*mut SDL_Window, *mut SDL_Renderer)>::uninit();
        let ptr = ret.as_mut_ptr();

        unsafe {
            if SDL_CreateWindowAndRenderer(
                title.as_ptr(),
                size.x,
                size.y,
                flags.into(),
                &raw mut (*ptr).0,
                &raw mut (*ptr).1,
            ) {
                // SAFETY: The above function succeeds only when both
                // the window and renderer are initialized.
                let init = ret.assume_init();
                let wnd = Self::from_ptr(init.0).unwrap_unchecked();
                let rnd = Renderer::from_ptr(init.1).unwrap_unchecked();

                Ok((wnd, rnd))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get a window from a stored ID.
    ///
    /// Returns [`None`] if no window with that ID exists.
    ///
    /// # Safety
    ///
    /// The lifetime of the returned reference is inferred.
    /// In practice, it's going to be valid until the window is destroyed.
    /// It is your responsibility to only use it before that happens.
    ///
    /// # Remarks
    ///
    /// The numeric ID is what `SDL_WindowEvent` references, and is necessary
    /// to map these events to specific window objects.
    #[doc(alias = "SDL_GetWindowFromID")]
    pub unsafe fn from_id<'a>(id: WindowId) -> Option<Ref<'a, Window>> {
        NonNull::new(unsafe { SDL_GetWindowFromID(id.as_sdl()) }).map(|handle| {
            let handle = WindowHandle { handle };
            unsafe { Ref::from_handle(handle) }
        })
    }

    /// Get the numeric ID of a window.
    ///
    /// # Remarks
    ///
    /// The numeric ID is what `SDL_WindowEvent` references, and is necessary
    /// to map these events to specific window objects.
    #[doc(alias = "SDL_GetWindowID")]
    pub fn id(&self) -> WindowId {
        let id = unsafe { SDL_GetWindowID(self.inner.handle.as_ptr()) }.0;

        // SAFETY: Valid windows should always have an ID.
        unsafe { WindowId::from_raw_unchecked(id) }
    }
}
