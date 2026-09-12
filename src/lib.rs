#![allow(dead_code)]

use sdl3_sys::{
    init::SDL_IsMainThread,
    platform::SDL_GetPlatform,
    timer::{SDL_GetTicks, SDL_GetTicksNS},
};

use crate::util::c_ptr_to_str;

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
