use std::ffi::CStr;

use rustest::test;
use sandlot::{s, str::Str};

#[test]
fn str_from_cstr_exposes_all_views() {
    let value = s!("hello");

    assert_eq!(value.to_bytes(), b"hello");
    assert_eq!(value.to_bytes_with_nul(), b"hello\0");
    assert_eq!(value, "hello");
}

#[test]
fn str_supports_empty_strings() {
    let value = s!("");

    assert!(value.to_bytes().is_empty());
    assert_eq!(value.to_bytes_with_nul(), b"\0");
    assert_eq!(value, "");
    assert_eq!(value, c"");
}

#[test]
fn str_preserves_utf8_without_loss() {
    let text = s!("Sandlot — 日本語 🦀");
    let bytes = [text.to_bytes(), b"\0"].concat();
    let cstr = CStr::from_bytes_with_nul(&bytes).unwrap();
    let value = Str::try_from(cstr).unwrap();

    assert_eq!(value, text);
    assert_eq!(value.to_bytes(), text.to_bytes());
    assert_eq!(value.to_bytes(), text.to_bytes());
}

#[test]
fn str_from_ptr_matches_from_cstr() {
    let cstr = c"pointer input";
    let from_cstr = Str::try_from(cstr).unwrap();
    let from_ptr = unsafe { Str::from_ptr(cstr.as_ptr()).unwrap() };

    assert_eq!(from_ptr.to_bytes_with_nul(), from_cstr.to_bytes_with_nul());
    assert_eq!(from_ptr.to_str(), from_cstr.to_str());
}

#[test]
fn str_display_trait() {
    let text = c"Sandlot — 日本語 🦀";
    let ucs = Str::try_from(text).unwrap();
    let disp = ucs.to_string();

    assert_eq!(&text.to_string_lossy(), &disp);
}

#[test]
fn str_try_from() {
    assert!(Str::try_from(c"Test").is_ok());
    assert!(Str::try_from(c"\x05\x99\xFF").is_err());
    assert!(Str::try_from("No nul terminator :(").is_err());
    assert!(Str::try_from("Nul terminator :D\0").is_ok());
    assert!(Str::try_from("Bad nul \0terminator :(\0").is_err());

    let bytes_good = b"Hello\0";
    assert!(Str::try_from(bytes_good.as_slice()).is_ok());

    let bytes_bad = b"Hello\0World\0";
    assert!(Str::try_from(bytes_bad.as_slice()).is_err());
}
