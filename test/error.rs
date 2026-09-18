use std::ffi::CStr;

use rustest::test;
use sandlot::{Result, error::Error, str::Str};

fn s(c: &CStr) -> &Str {
    unsafe { Str::from_cstr_unchecked(c) }
}

/// [`Error::current()`] reads the current SDL error string.
#[test]
fn error_current() {
    let err = Error::set(s(c"failed to frobnicate"));
    assert_eq!(err.as_str(), "failed to frobnicate");
}

/// [`Error`] owns its string, so it isn't affected by later SDL errors.
#[test]
fn error_snapshot() {
    assert_eq!(Error::set(s(c"first error")).as_str(), "first error");
    assert_eq!(Error::set(s(c"second error")).as_str(), "second error");
}

/// After [`SDL_ClearError()`], [`Error::current()`] is empty.
#[test]
fn error_empty_after_clear() {
    Error::clear();
    assert_eq!(Error::current().as_str(), "");
}

/// The [`Display`] implementation forwards to the error string.
#[test]
fn error_display() {
    assert_eq!(
        Error::set(s(c"a displayable error")).to_string(),
        "a displayable error"
    );
}

/// [`Error::into_cstring()`] yields a nul-terminated copy of the string.
#[test]
fn error_into_cstring() {
    let cstr = Error::set(s(c"an error for C")).into_cstring();

    assert_eq!(cstr, c"an error for C");
    assert_eq!(cstr.to_bytes_with_nul(), b"an error for C\0");

    // The SDL error itself is unaffected.
    assert_eq!(Error::current().as_str(), "an error for C");
}

/// [`Error::current()`] handles non-ASCII UTF-8 messages.
#[test]
fn error_utf8() {
    assert_eq!(
        Error::set(s(c"blåbær 日本語 🦀")).as_str(),
        "blåbær 日本語 🦀"
    );
}

/// [`Error`] implements [`std::error::Error`], so it works with `?` and `Box<dyn Error>`.
#[test]
fn error_std_error() {
    fn propagate() -> Result<()> {
        Err(Error::set(s(c"propagated")))
    }

    let err = propagate().unwrap_err();
    assert_eq!(err.as_str(), "propagated");

    let boxed: Box<dyn std::error::Error> = Box::new(Error::current());
    assert_eq!(boxed.to_string(), "propagated");
}
