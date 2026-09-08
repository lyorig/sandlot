//! Logging facilities.
//!
//! Enables logging with various priorities and categories. Sends output via the OS'
//! native logging API, ensuring output happens, for example, even on Windows when not using the `CONSOLE` subsystem.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryLog)):
//! - [x] SDL_GetLogPriority
//! - [x] SDL_Log
//! - [x] SDL_LogCritical
//! - [x] SDL_LogDebug
//! - [x] SDL_LogError
//! - [x] SDL_LogInfo
//! - [ ] SDL_LogMessage
//! - [x] SDL_LogTrace
//! - [x] SDL_LogVerbose
//! - [x] SDL_LogWarn
//! - [x] SDL_ResetLogPriorities
//! - [x] SDL_SetLogPriorities
//! - [x] SDL_SetLogPriority
//! - [ ] SDL_SetLogPriorityPrefix
//!
//! Not planned for implementation:
//! - SDL_GetDefaultLogOutputFunction
//! - SDL_GetLogOutputFunction
//! - SDL_LogMessageV
//! - SDL_SetLogOutputFunction

use std::{
    ffi::{CString, c_char},
    fmt::Arguments,
};

use sdl3_sys::log::*;

use crate::impl_enum_transmute;

#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_LogPriority")]
/// The predefined log priorities.
pub enum Priority {
    Trace = SDL_LogPriority::TRACE.0,
    Verbose = SDL_LogPriority::VERBOSE.0,
    Debug = SDL_LogPriority::DEBUG.0,
    Info = SDL_LogPriority::INFO.0,
    Warn = SDL_LogPriority::WARN.0,
    Error = SDL_LogPriority::ERROR.0,
    Critical = SDL_LogPriority::CRITICAL.0,
}

#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_LogCategory")]
/// The predefined log categories.
///
/// # Remarks
///
/// By default the application and gpu categories are enabled at the INFO
/// level, the assert category is enabled at the WARN level, test is enabled
/// at the VERBOSE level and all other categories are enabled at the ERROR
/// level.
pub enum Category {
    Application = SDL_LogCategory::APPLICATION.0,
    Error = SDL_LogCategory::ERROR.0,
    Assert = SDL_LogCategory::ASSERT.0,
    System = SDL_LogCategory::SYSTEM.0,
    Audio = SDL_LogCategory::AUDIO.0,
    Video = SDL_LogCategory::VIDEO.0,
    Render = SDL_LogCategory::RENDER.0,
    Input = SDL_LogCategory::INPUT.0,
    Test = SDL_LogCategory::TEST.0,
    Gpu = SDL_LogCategory::GPU.0,
}

impl_enum_transmute!(SDL_LogPriority, Priority);
impl_enum_transmute!(SDL_LogCategory, Category);

fn args2cstr(args: Arguments) -> CString {
    let s = args.to_string();
    unsafe { CString::from_vec_unchecked(s.into_bytes()) }
}

const FMT: *const c_char = c"%s".as_ptr();

macro_rules! log_for_priority {
    ($name:ident, $sdl:ident, $alias:literal) => {
        #[doc(alias = $alias)]
        pub fn $name(category: Category, args: Arguments) {
            let cs = args2cstr(args);
            unsafe { $sdl(category as _, FMT, cs.as_ptr()) };
        }
    };
}

log_for_priority!(trace, SDL_LogTrace, "SDL_LogTrace");
log_for_priority!(verbose, SDL_LogVerbose, "SDL_LogVerbose");
log_for_priority!(debug, SDL_LogDebug, "SDL_LogDebug");
log_for_priority!(info, SDL_LogInfo, "SDL_LogInfo");
log_for_priority!(warn, SDL_LogWarn, "SDL_LogWarn");
log_for_priority!(error, SDL_LogError, "SDL_LogError");
log_for_priority!(critical, SDL_LogCritical, "SDL_LogCritical");

#[doc(alias = "SDL_Log")]
pub fn log(args: Arguments) {
    let cs = args2cstr(args);
    unsafe { SDL_Log(FMT, cs.as_ptr()) };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::log(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_trace {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::trace(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::trace($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_verbose {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::verbose(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::verbose($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::debug(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::debug($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::info(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::info($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::warn(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::warn($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::error(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::error($cat, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_critical {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        $crate::log::critical(
            $crate::log::Category::Application,
            format_args!($fmt $(, $($arg)*)?),
        );
    };
    ($cat:expr, $($arg:tt)*) => {
        $crate::log::critical($cat, format_args!($($arg)*));
    };
}

/// Set the priority of a particular log category.
#[doc(alias = "SDL_SetLogPriority")]
pub fn set_priority(category: Category, priority: Priority) {
    unsafe { SDL_SetLogPriority(category as _, priority.into()) }
}

/// Set the priority of all log categories.
#[doc(alias = "SDL_SetLogPriorities")]
pub fn set_priorities(priority: Priority) {
    unsafe { SDL_SetLogPriorities(priority.into()) }
}

/// Get the priority of a particular log category.
#[doc(alias = "SDL_GetLogPriority")]
pub fn priority(category: Category) -> Priority {
    unsafe { SDL_GetLogPriority(category as _) }.into()
}

/// Reset all priorities to default.
///
/// This is called by SDL's quit function.
#[doc(alias = "SDL_ResetLogPriorities")]
pub fn reset_priorities() {
    unsafe { SDL_ResetLogPriorities() }
}
