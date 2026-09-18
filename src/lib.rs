#![allow(dead_code)]

use std::ffi::CStr;

use sdl3_sys::{
    init::SDL_IsMainThread,
    misc::SDL_OpenURL,
    platform::SDL_GetPlatform,
    timer::{SDL_GetTicks, SDL_GetTicksNS},
};

use crate::util::{c_ptr_to_str, to_result};

pub mod boxed;

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
pub mod str;
pub mod string;
pub mod surface;
pub mod texture;
pub mod traits;
pub mod ttf;
pub mod util;
pub mod window;

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
/// the one that calls [`Video::init`](init::Video::init), which should usually be
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

/// Open a URL/URI in the browser or other appropriate external application.
///
/// Open a URL in a separate, system-provided application. How this works will vary wildly depending on the platform.
/// This will likely launch what makes sense to handle a specific URL’s protocol (a web browser for http://, etc),
/// but it might also be able to launch file managers for directories and other things.
///
/// What happens when you open a URL varies wildly as well: your game window may lose focus (and may or may not
/// lose focus if your game was fullscreen or grabbing input at the time). On mobile devices, your app will likely
/// move to the background or your process might be paused. Any given platform may or may not handle a given URL.
///
/// If this is unimplemented (or simply unavailable) for a platform, this will fail with an error.
/// A successful result does not mean the URL loaded, just that we launched something to handle it
/// (or at least believe we did).
///
/// All this to say: this function can be useful, but you should definitely test it on every platform you target.
///
/// # Parameters
///
/// * `url`: a valid URL/URI to open. Use `file:///full/path/to/file` for local files, if supported.
#[doc(alias = "SDL_OpenURL")]
pub fn open_url(url: &CStr) -> Result<()> {
    to_result(unsafe { SDL_OpenURL(url.as_ptr()) })
}
