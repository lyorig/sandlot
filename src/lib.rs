#![allow(dead_code)]

use std::ptr::NonNull;

use sdl3_sys::{
    filesystem::{SDL_GetBasePath, SDL_GetUserFolder},
    init::{SDL_IsMainThread, SDL_Quit},
    platform::SDL_GetPlatform,
    timer::{SDL_GetTicks, SDL_GetTicksNS},
};

use crate::{
    error::Error,
    fs::Folder,
    util::{c_ptr_to_str, opt2res_map},
};

pub mod boxed;

pub mod clipboard;
pub mod color;
pub mod cpu;
pub mod display;
pub mod error;
pub mod event;
pub mod fs;
pub mod gpu;
pub mod init;
pub mod keyboard;
pub mod log;
pub mod msgbox;
pub mod pixels;
pub mod properties;
pub mod rect;
pub mod renderer;
pub mod resource;
pub mod string;
pub mod surface;
pub mod texture;
pub mod traits;
pub mod ttf;
pub mod util;
pub mod window;

/// A zero-sized type that only exists to call [`SDL_Quit`].
/// As such, think of it as a guard that creates a scope for
/// the initialization of subsystems, ensuring they're properly
/// quit once it goes out of scope.
pub struct Context;

impl Context {
    /// Like [`Self::new`], without the safety checks.
    ///
    /// # Safety
    /// Only call this on the main thread.
    pub unsafe fn new_unchecked() -> Self {
        Self {}
    }

    /// Create a new context, enabling you to initialize individual subsystems.
    ///
    /// # Panics
    /// Panics if this function is not called on the main thread.
    ///
    /// # Why doesn't this return a [`Result`] instead?
    /// TL;DR: It's less error-prone.
    /// Contexts are sometimes left unused, i.e.
    /// ```
    /// use sandlot::Context;
    ///
    /// let _ctx = Context::new();
    /// ```
    /// If [`Self::new`] returned [`Err`], this snippet would silently skip
    /// the destructor and not quit SDL in case of an error. Not running on
    /// the main thread isn't really something that can happen by chance and you
    /// can recover from. If necessary, check yourself via [`crate::is_main_thread`].
    ///
    /// In addition, [`Result`] is only intended to originate from SDL API calls.
    /// Since [`Context`] is a ZST providing an abstraction over SDL initialization,
    /// this would newly require a way to create a "custom" error.
    pub fn new() -> Self {
        assert!(crate::is_main_thread(), "Context not on main thread");
        Self {}
    }

    /// Get the directory where the application was run from.
    /// [`Err`] is returned on error or when the platform does not implement this functionality.
    ///
    /// Returns an absolute, UTF-8 path to the application data directory. The
    /// path is guaranteed to end with a path separator (`\\` on Windows and `/`
    /// on most other platforms).
    ///
    /// On macOS and iOS, an application inside a `.app` bundle returns the
    /// bundle's resource directory by default. This can be changed with the
    /// `SDL_FILESYSTEM_BASE_DIR_TYPE` property in `Info.plist`: `resource`
    /// selects the resource directory, `bundle` selects the bundle directory,
    /// and `parent` selects the directory containing the bundle. On Android,
    /// this returns `./`, which allows filesystem operations to use internal
    /// storage and the asset system. On Nintendo 3DS, this returns the
    /// application's `romfs` directory, which is not writable.
    ///
    /// SDL caches the result, but the first call may be slow.
    ///
    /// The cached path is freed when this [`Context`] is dropped, so its lifetime
    /// is tied to `&self`.
    #[doc(alias = "SDL_GetBasePath")]
    pub fn base_path(&self) -> Result<&str> {
        let ptr = unsafe { SDL_GetBasePath() };
        if ptr.is_null() {
            Err(Error::current())
        } else {
            // SAFETY: The string returned by `SDL_GetBasePath()`
            // is guaranteed to be valid UTF-8.
            Ok(unsafe { c_ptr_to_str(ptr) })
        }
    }

    /// Find the most suitable user folder for a specific purpose.
    ///
    /// `folder` selects the type of folder to find, such as
    /// [`Folder::Documents`] or [`Folder::Downloads`]. These are user folders
    /// intended for the user to access and manage. For application-specific data,
    /// use [`crate::fs::pref_path`] instead.
    ///
    /// Returns [`Err`] if the requested folder is unsupported or cannot be found.
    /// The returned path is guaranteed to end with a path separator (`\\` on
    /// Windows and `/` on most other platforms).
    ///
    /// SDL caches the result. The cached path is freed when this [`Context`] is
    /// dropped, so its lifetime is tied to `&self`.
    #[doc(alias = "SDL_GetUserFolder")]
    pub fn user_folder(&self, folder: Folder) -> Result<&str> {
        let ptr = unsafe { SDL_GetUserFolder(folder.into()) };

        // SAFETY: SDL guarantees the path is valid UTF-8.
        unsafe {
            opt2res_map(NonNull::new(ptr.cast_mut()), |ptr| {
                c_ptr_to_str(ptr.as_ptr())
            })
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Context {
    /// Deinitialize SDL, quitting all subsystems.
    #[doc(alias = "SDL_Quit")]
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
    }
}

/// Convenience alias for [`std::result::Result<T, Error>`].
/// Used as the return type throughout this crate.
pub type Result<T> = std::result::Result<T, error::Error>;

/// Get the name of the platform.
///
/// If the correct platform name is not available, returns a string beginning
/// with the text "Unknown".
///
/// # Remarks
///
/// Here are the names returned for some (but not all) supported platforms:
/// - "Windows"
/// - "macOS"
/// - "Linux"
/// - "iOS"
/// - "Android"
#[doc(alias = "SDL_GetPlatform")]
pub fn platform() -> &'static str {
    // SAFETY: All SDL3 platform strings are UTF-8,
    // and are stored statically.
    unsafe { c_ptr_to_str(SDL_GetPlatform()) }
}

/// Return whether this is the main thread.
///
/// # Remarks
///
/// On Apple platforms, the main thread is the thread that runs your
/// program's `main()` entry point. On other platforms, the main thread is
/// the one that calls `SDL_Init(SDL_INIT_VIDEO)`, which should usually be
/// the one that runs your program's `main()` entry point. If you are using
/// the main callbacks, `SDL_AppInit`, `SDL_AppIterate`, and `SDL_AppQuit`
/// are all called on the main thread.
#[doc(alias = "SDL_IsMainThread")]
pub fn is_main_thread() -> bool {
    unsafe { SDL_IsMainThread() }
}

/// Get the number of milliseconds that have elapsed since the SDL library
/// initialization.
#[doc(alias = "SDL_GetTicks")]
pub fn ticks() -> u64 {
    unsafe { SDL_GetTicks() }
}

/// Get the number of nanoseconds since SDL library initialization.
#[doc(alias = "SDL_GetTicksNS")]
pub fn ticks_ns() -> u64 {
    unsafe { SDL_GetTicksNS() }
}
