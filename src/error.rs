//! Wrapper for [`SDL_GetError`], suitable for usage in [`Result`].

use std::fmt::{Debug, Display};

use sdl3_sys::{
    error::{SDL_ClearError, SDL_GetError, SDL_SetError},
    stdinc::SDL_strdup,
};

use crate::{str::Str, string::String};

/// Lightweight wrapper around the string returned by SDL in case of errors.
///
/// This is a pointer-sized struct which owns a duplicate of the error message
/// present at the time [`Error::current`] is called.
pub struct Error {
    reason: String,
}

impl Error {
    /// # Sandlot-specific
    ///
    /// Duplicates the SDL error string and stores it in a [`Box`] so as to avoid
    /// the message being silently overwritten by subsequent [`SDL_SetError`] calls
    /// from inside SDL.
    ///
    /// # SDL documentation
    ///
    /// Retrieve a message about the last error that occurred on the current
    /// thread.
    ///
    /// Returns an empty string if there hasn't been an error message set.
    ///
    /// # Remarks
    ///
    /// It is possible for multiple errors to occur before calling this
    /// function. Only the last error is returned.
    ///
    /// The message is only applicable when an SDL function has signaled an
    /// error. You must check the return values of SDL function calls to
    /// determine when to appropriately call this function. You should *not*
    /// use the results of [`Error::current`] to decide if an error has
    /// occurred! Sometimes SDL will set an error string even when reporting
    /// success.
    ///
    /// SDL will *not* clear the error string for successful API calls. You
    /// *must* check return values for failure cases before you can assume
    /// the error string applies.
    ///
    /// Error strings are set per-thread, so an error set in a different
    /// thread will not interfere with the current thread's operation.
    #[doc(alias = "SDL_GetError")]
    pub fn current() -> Self {
        let dup = unsafe { SDL_strdup(SDL_GetError()) };
        let reason = unsafe { String::from_ptr_unchecked(dup) };

        Self { reason }
    }

    /// Sets the error message for the current thread, then returns [`Error::current`].
    ///
    /// Calling this function will replace any previous error message that was set.
    #[doc(alias = "SDL_SetError")]
    pub fn set(reason: Str) -> Self {
        unsafe { SDL_SetError(c"%s".as_ptr(), reason.as_ptr()) };
        Self::current()
    }

    /// Clear any previous error message for this thread.
    #[doc(alias = "SDL_ClearError")]
    pub fn clear() {
        SDL_ClearError();
    }

    pub fn as_str(&self) -> Str<'_> {
        // SAFETY: Error messages coming from SDL are UTF-8,
        // and `Error::set` also enforces UTF-8.
        unsafe { Str::from_non_null(self.reason.as_non_null()) }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.as_str(), f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(&self.reason, f)
    }
}

impl std::error::Error for Error {}
