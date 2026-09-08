use rustest::test;
use sandlot::{Result, error::Error};
use sdl3_sys::error::{SDL_ClearError, SDL_SetError};

/// [`Error::current()`] reads the current SDL error string.
#[test]
fn error_current() {
    unsafe { SDL_SetError(c"failed to frobnicate".as_ptr()) };

    let err = Error::current();
    assert_eq!(err.as_str(), "failed to frobnicate");
}

/// [`Error`] owns its string, so it isn't affected by later SDL errors.
#[test]
fn error_snapshot() {
    unsafe { SDL_SetError(c"first error".as_ptr()) };
    let err = Error::current();

    unsafe { SDL_SetError(c"second error".as_ptr()) };

    assert_eq!(err.as_str(), "first error");
    assert_eq!(Error::current().as_str(), "second error");
}

/// After [`SDL_ClearError()`], [`Error::current()`] is empty.
#[test]
fn error_empty_after_clear() {
    SDL_ClearError();
    assert_eq!(Error::current().as_str(), "");
}

/// The [`Display`] implementation forwards to the error string.
#[test]
fn error_display() {
    unsafe { SDL_SetError(c"a displayable error".as_ptr()) };
    assert_eq!(Error::current().to_string(), "a displayable error");
}

/// [`Error::into_cstring()`] yields a nul-terminated copy of the string.
#[test]
fn error_into_cstring() {
    unsafe { SDL_SetError(c"an error for C".as_ptr()) };

    let cstr = Error::current().into_cstring();
    assert_eq!(cstr, c"an error for C");
    assert_eq!(cstr.to_bytes_with_nul(), b"an error for C\0");

    // The SDL error itself is unaffected.
    assert_eq!(Error::current().as_str(), "an error for C");
}

/// [`Error::current()`] handles non-ASCII UTF-8 messages.
#[test]
fn error_utf8() {
    unsafe { SDL_SetError(c"blåbær 日本語 🦀".as_ptr()) };
    assert_eq!(Error::current().as_str(), "blåbær 日本語 🦀");
}

/// [`Error`] implements [`std::error::Error`], so it works with `?` and `Box<dyn Error>`.
#[test]
fn error_std_error() {
    fn propagate() -> Result<()> {
        unsafe { SDL_SetError(c"propagated".as_ptr()) };
        Err(Error::current())
    }

    let err = propagate().unwrap_err();
    assert_eq!(err.as_str(), "propagated");

    let boxed: Box<dyn std::error::Error> = Box::new(Error::current());
    assert_eq!(boxed.to_string(), "propagated");
}
