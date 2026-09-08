//! [`String`](std::string::String), but using SDL's memory allocation API.

use std::{
    ffi::{CStr, c_char},
    fmt::Display,
};

use crate::{Result, boxed::Box};

/// An SDL-allocated string.
///
/// Unlike [`std::string::String`] which wraps a [`Vec<u8>`], [`String`] wraps a [`Box<c_char>`],
/// as SDL always provides a null-terminated string pointer. This makes it borrow some [`CStr`]
/// semantics (i.e. [`Self::count_bytes`]). However, SDL also often makes UTF-8 guarantees about string contents,
/// so certain conversion methods become infallible (such as [`String::into_boxed_str`]).
pub struct String {
    handle: Box<c_char>,
}

impl String {
    /// # Safety
    /// See the safety requirements of [`Box::from_raw`].
    pub(crate) unsafe fn from_raw(handle: *mut c_char) -> Self {
        let handle = unsafe { Box::from_raw(handle) };
        Self { handle }
    }

    /// # Safety
    /// See the safety requirements of [`Box::from_raw_nullck`].
    pub(crate) unsafe fn from_raw_nullck(handle: *mut c_char) -> Result<Self> {
        unsafe { Box::from_raw_nullck(handle) }.map(|handle| Self { handle })
    }

    /// Convert this SDL string to a byte slice.
    /// This involves calculating the length via [`Self::count_bytes`].
    pub fn to_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.handle.as_ptr().cast(), self.count_bytes()) }
    }

    /// Convert this SDL string to a string slice. This can be done,
    /// since all strings originating from SDL are guaranteed UTF-8.
    /// This involves calculating its length via [`Self::count_bytes`].
    pub fn to_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.to_bytes()) }
    }

    /// Analogous to [`CStr::count_bytes`].
    pub fn count_bytes(&self) -> usize {
        let cs = unsafe { CStr::from_ptr(self.handle.as_ptr()) };
        cs.count_bytes()
    }

    /// Transforms `self` into a boxed `str`.
    /// This involves calculating the length via [`Self::count_bytes`].
    pub fn into_boxed_str(self) -> Box<str> {
        let len = self.count_bytes();
        let ptr = self.handle.into_raw_non_null().cast::<u8>();
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr(), len) };
        let slice_ptr = std::ptr::from_mut(slice);

        unsafe { Box::from_raw(slice_ptr as *mut str) }
    }
}

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = self.to_str();
        <str as Display>::fmt(str, f)
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.to_str() == other.to_str()
    }
}
