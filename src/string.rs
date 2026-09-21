//! [`String`](std::string::String), but using SDL's memory allocation API.

use std::{
    ffi::{CStr, c_char},
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::{Result, boxed::Box, error::Error};

/// An SDL-allocated nul-terminated UTF-8 string.
///
/// Unlike [`std::string::String`] which wraps a [`Vec<u8>`], [`String`] wraps a [`Box<c_char>`],
/// as SDL always provides a null-terminated string pointer. This makes it borrow some [`CStr`]
/// semantics (i.e. [`String::count_bytes`]). However, SDL also often makes UTF-8 guarantees about string contents,
/// so certain conversion methods become infallible (such as [`String::into_boxed_str`]).
pub struct String {
    handle: Box<c_char>,
}

impl String {
    /// # Safety
    ///
    /// See the safety requirements of [`Box::from_ptr_unchecked`]
    ///
    /// TL;DR: `handle` points to an UTF-8 nul-terminated string allocated
    /// with SDL's allocator, and it's okay to take ownership of it.
    pub(crate) unsafe fn from_ptr_unchecked(handle: *mut c_char) -> Self {
        let handle = unsafe { Box::from_ptr_unchecked(handle) };
        Self { handle }
    }

    /// # Safety
    ///
    /// See the safety requirements of [`Box::from_ptr`].
    pub(crate) unsafe fn from_ptr(handle: *mut c_char) -> Result<Self> {
        unsafe { Box::from_ptr(handle) }
            .map(|handle| Self { handle })
            .ok_or_else(Error::current)
    }

    /// Convert this string to a byte slice.
    ///
    /// This involves calculating the length via [`String::count_bytes`].
    pub fn to_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.handle.as_ptr().cast(), self.count_bytes()) }
    }

    /// Convert this string to a string slice.
    ///
    /// This can be done since the data we point to is guaranteed UTF-8.
    ///
    /// This involves calculating its length via [`String::count_bytes`].
    pub fn to_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.to_bytes()) }
    }

    /// Convert this string to a C string slice.
    ///
    /// This can be done since the data we point to is guaranteed to be nul-terminated.
    ///
    /// For the time being, this involves a length calculation,
    /// although Rust plans to make [`CStr`] only store the pointer,
    /// and perform the length calculation on demand. As such, it's
    /// using the `as_*` naming in advance.
    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.handle.as_ptr()) }
    }

    /// Analogous to [`CStr::count_bytes`].
    pub fn count_bytes(&self) -> usize {
        let cs = unsafe { CStr::from_ptr(self.handle.as_ptr()) };
        cs.count_bytes()
    }

    /// Transforms `self` into a boxed [`str`].
    ///
    /// This involves calculating the length via [`String::count_bytes`].
    pub fn into_boxed_str(self) -> Box<str> {
        let len = self.count_bytes();
        let ptr = self.handle.into_non_null().cast::<u8>();
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr(), len) };
        let slice_ptr = std::ptr::from_mut(slice);

        unsafe { Box::from_ptr_unchecked(slice_ptr as *mut str) }
    }
}

impl Deref for String {
    type Target = Box<c_char>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self.to_str(), f)
    }
}

impl Debug for String {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(self.to_str(), f)
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.to_str() == other.to_str()
    }
}
