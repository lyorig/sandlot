//! Wrapper for [`SDL_GetError`], suitable for usage in [`Result`].

use std::ffi::CString;

use sdl3_sys::error::{SDL_ClearError, SDL_GetError, SDL_SetError};

use crate::str::Str;

#[derive(Debug)]
pub struct Error {
    reason: String,
}

impl Error {
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
        // SAFETY: SDL's error strings are UTF-8.
        let s = unsafe { Str::from_ptr(SDL_GetError()) };

        // Speculatively reserve capacity for a null byte,
        // in case `Self::into_cstring()` is called.
        let mut reason = String::with_capacity(s.len() + 1);
        reason.push_str(s);

        Self { reason }
    }

    /// Sets the error message for the current thread, then returns [`Error::current`].
    ///
    /// Calling this function will replace any previous error message that was set.
    #[doc(alias = "SDL_SetError")]
    pub fn set(reason: &Str) -> Self {
        unsafe { SDL_SetError(c"%s".as_ptr(), reason.as_ptr()) };
        Self::current()
    }

    /// Clear any previous error message for this thread.
    #[doc(alias = "SDL_ClearError")]
    pub fn clear() {
        SDL_ClearError();
    }

    pub fn as_str(&self) -> &str {
        self.reason.as_str()
    }

    /// Consume the [`Error`], turning it into a [`CString`].
    /// This is useful when interfacing with C APIs which
    /// expect nul-terminated strings.
    pub fn into_cstring(self) -> CString {
        // SAFETY: The stored SDL string contains no nul bytes.
        let mut vec = self.reason.into_bytes();
        vec.push(b'\0');

        unsafe { CString::from_vec_with_nul_unchecked(vec) }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for Error {}
