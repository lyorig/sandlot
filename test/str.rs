use std::ffi::CStr;

use rustest::test;
use sandlot::str::Str;

#[test]
fn str_from_cstr_exposes_all_views() {
    let cstr = c"hello";
    let value = unsafe { Str::from_cstr_unchecked(cstr) };

    assert_eq!(value.as_bytes(), b"hello");
    assert_eq!(value.as_bytes_with_nul(), b"hello\0");
    assert_eq!(value.as_str(), "hello");
    assert_eq!(value.as_cstr(), cstr);
}

#[test]
fn str_supports_empty_strings() {
    let value = unsafe { Str::from_cstr_unchecked(c"") };

    assert!(value.as_bytes().is_empty());
    assert_eq!(value.as_bytes_with_nul(), b"\0");
    assert_eq!(value.as_str(), "");
    assert_eq!(value.as_cstr(), c"");
}

#[test]
fn str_preserves_utf8_without_loss() {
    let text = "Sandlot — 日本語 🦀";
    let bytes = [text.as_bytes(), b"\0"].concat();
    let cstr = CStr::from_bytes_with_nul(&bytes).unwrap();
    let value = unsafe { Str::from_cstr_unchecked(cstr) };

    assert_eq!(value.as_str(), text);
    assert_eq!(value.as_bytes(), text.as_bytes());
    assert_eq!(value.as_cstr().to_bytes(), text.as_bytes());
}

#[test]
fn str_from_ptr_matches_from_cstr() {
    let cstr = c"pointer input";
    let from_cstr = unsafe { Str::from_cstr_unchecked(cstr) };
    let from_ptr = unsafe { Str::from_ptr(cstr.as_ptr()) };

    assert_eq!(from_ptr.as_bytes_with_nul(), from_cstr.as_bytes_with_nul());
    assert_eq!(from_ptr.as_str(), from_cstr.as_str());
}

#[test]
fn str_display_trait() {
    let text = c"Sandlot — 日本語 🦀";
    let ucs = unsafe { Str::from_cstr_unchecked(text) };
    let disp = ucs.to_string();

    assert_eq!(&text.to_string_lossy(), &disp);
}
