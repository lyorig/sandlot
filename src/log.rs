//! Simple log messages with priorities and categories.
//!
//! A message's [`Priority`] signifies how important the message is.
//! A message's [`Category`] signifies from what domain it belongs to.
//! Every category has a minimum priority specified: when a message belongs to that category,
//! it will only be sent out if it has that minimum priority or higher.
//!
//! SDL's own logs are sent below the default priority threshold, so they are
//! quiet by default.
//!
//! You can change the log verbosity programmatically using [`set_priority`],
//! with `SDL_SetHint`(`SDL_HINT_LOGGING`, ...), or with the `SDL_LOGGING`
//! environment variable. This variable is a comma separated set of
//! category=level tokens that define the default logging levels for SDL
//! applications.
//!
//! The category can be a numeric category, one of "app", "error", "assert",
//! "system", "audio", "video", "render", "input", "test", or `*` for any
//! unspecified category.
//!
//! The level can be a numeric level, one of "trace", "verbose", "debug",
//! "info", "warn", "error", "critical", or "quiet" to disable that category.
//!
//! You can omit the category if you want to set the logging level for all
//! categories.
//!
//! If this hint isn't set, the default log levels are equivalent to:
//!
//! `app=info,assert=warn,test=verbose,*=error`
//!
//! Here's where the messages go on different platforms:
//!
//! - Windows: debug output stream
//! - Android: log output
//! - Others: standard error output (stderr)
//!
//! You don't need to have a newline (`\n`) on the end of messages, the
//! functions will do that for you. For consistent behavior cross-platform,
//! you shouldn't have any newlines in messages, such as to log multiple
//! lines in one call; unusual platform-specific behavior can be observed in
//! such usage. Do one log call per line instead, with no newlines in messages.
//!
//! Each log call is atomic, so you won't see log messages cut off one another
//! when logging from multiple threads.
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
        #[doc = concat!("Delegated to by [`log_", stringify!($name), "!`](crate::log_", stringify!($name), "!)")]
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

/// Logs a formatted string with [`Priority::Info`] and [`Category::Application`].
///
/// You should use [`log!`](crate::log!) instead, which delegates to this function.
#[doc(alias = "SDL_Log")]
pub fn log(args: Arguments) {
    let cs = args2cstr(args);
    unsafe { SDL_Log(FMT, cs.as_ptr()) };
}

/// Logs a formatted string with [`Priority::Info`] and [`Category::Application`].
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::log(format_args!($($arg)*));
    };
}

/// Logs a formatted string with [`Priority::Trace`] and an optional category.
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

/// Logs a formatted string with [`Priority::Verbose`] and an optional category.
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

/// Logs a formatted string with [`Priority::Debug`] and an optional category.
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

/// Logs a formatted string with [`Priority::Info`] and an optional category.
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

/// Logs a formatted string with [`Priority::Warn`] and an optional category.
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

/// Logs a formatted string with [`Priority::Error`] and an optional category.
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

/// Logs a formatted string with [`Priority::Critical`] and an optional category.
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
