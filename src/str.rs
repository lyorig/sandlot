use std::{
    ffi::{CStr, FromBytesWithNulError, c_char},
    fmt::{self, Debug, Display},
    ptr::NonNull,
    str::Utf8Error,
};

use sdl3_sys::stdinc::SDL_strdup;

use crate::string::String;

/// Refers to an immutable nul-terminated string in UTF-8 format.
///
/// This enables passing pointers to and from SDL functions without having to
/// calculate/store the length. In addition, infallible conversion methods to
/// [`str`] and [`CStr`] exist, owing to the aforementioned guarantees. Create
/// a new [`Str`] from a string literal using the [`s!`](crate::s) macro.
///
/// # When to use this
///
/// Two prerequisites:
///
/// 1. The function only requires a nul-terminated pointer, not a length
///     - If not, use [`str`]
/// 2. The function expects UTF-8 data
///     - If not, use [`CStr`]
///
/// For example, [`SDL_SetWindowTitle`](sdl3_sys::video::SDL_SetWindowTitle)
/// explicitly states that the title is "expected to be in UTF-8 encoding".
/// And since the function only takes a pointer, [`Str`] is a prime canididate
/// for wrapping the argument in something more Rust-like.
///
/// On the other hand, [`SDL_CreateRenderer`](sdl3_sys::render::SDL_CreateRenderer)
/// only describes the `name` parameter as the "the name of the rendering driver to initialize".
/// Since the string is only going to be compared against an internal list of available renderers
/// via `strcmp` or something similar, guaranteeing UTF-8 doesn't bring anything to the table.
///
/// # A note on performance
///
/// [`Str`] implements many traits to make it usable just like a [`str`], such as [`PartialEq`]
/// for `&[u8]`, `&str`, and `&CStr`. However, many of these operations involve performing
/// a `strlen` internally, and so it might be beneficial to first convert it to a type with
/// a precalculated length such as `&str`, if you're going to be doing more length-related operations.
#[derive(Clone, Copy)]
pub struct Str<'a> {
    first_byte: &'a c_char,
}

impl<'a> Str<'a> {
    /// Analogous to [`CStr::from_ptr`], but with an extra UTF-8 requirement.
    ///
    /// Returns [`None`] if `ptr` is a null pointer.
    ///
    /// # Safety
    ///
    /// The caller must uphold the safety contract of [`CStr::from_ptr`]
    /// (barring the pointer being null, since this function checks that),
    /// and also ensure that the data pointer to by `ptr` is valid UTF-8.
    ///
    /// The lifetime is inferred; functions building on this one should tie
    /// it to something sensible.
    pub const unsafe fn from_ptr(ptr: *const c_char) -> Option<Self> {
        // `Option::map` isn't const, so match instead.
        match NonNull::new(ptr.cast_mut()) {
            Some(nn) => Some(unsafe { Self::from_non_null(nn) }),
            None => None,
        }
    }

    /// Analogous to [`CStr::from_ptr`], but with an extra UTF-8 requirement.
    ///
    /// # Safety
    ///
    /// The caller must uphold the safety contract of [`CStr::from_ptr`],
    /// and also ensure that the data pointer to by `ptr` is valid UTF-8.
    ///
    /// The lifetime is inferred; functions building on this one should tie
    /// it to something sensible.
    pub const unsafe fn from_ptr_unchecked(ptr: *const c_char) -> Self {
        let ptr = unsafe { NonNull::new(ptr.cast_mut()).unwrap_unchecked() };
        unsafe { Self::from_non_null(ptr) }
    }

    /// Analogous to [`CStr::from_ptr`], but with an extra UTF-8 requirement.
    ///
    /// # Safety
    ///
    /// The caller must uphold the safety contract of [`CStr::from_ptr`]
    /// (barring the pointer being null, since [`NonNull`] ensures that),
    /// and also ensure that the data pointer to by `ptr` is valid UTF-8.
    ///
    /// The lifetime is inferred; functions building on this one should tie
    /// it to something sensible.
    pub const unsafe fn from_non_null(ptr: NonNull<c_char>) -> Self {
        Self {
            first_byte: unsafe { ptr.cast().as_ref() },
        }
    }

    pub const fn as_non_null(self) -> NonNull<c_char> {
        NonNull::from_ref(self.first_byte)
    }

    pub const fn as_ptr(self) -> *const c_char {
        std::ptr::from_ref(self.first_byte)
    }

    /// Convert to a [`CStr`].
    ///
    /// For the time being, this involves a length calculation,
    /// although Rust plans to make [`CStr`] only store the pointer,
    /// and perform the length calculation on demand. As such, it's
    /// using the `as_*` naming in advance.
    pub const fn as_c_str(self) -> &'a CStr {
        // SAFETY: We're pointing to a nul-terminated string.
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    pub const fn to_bytes(self) -> &'a [u8] {
        self.as_c_str().to_bytes()
    }

    pub fn to_bytes_with_nul(self) -> &'a [u8] {
        self.as_c_str().to_bytes_with_nul()
    }

    /// Convert to a [`CStr`].
    ///
    /// This involves a length calculation.
    pub const fn to_str(self) -> &'a str {
        unsafe { str::from_utf8_unchecked(self.to_bytes()) }
    }

    /// Analogous to [`CStr::count_bytes`].
    pub const fn count_bytes(self) -> usize {
        self.as_c_str().count_bytes()
    }

    pub const fn is_empty(self) -> bool {
        // First byte is the nul terminator? Then the string is empty.
        *self.first_byte == 0
    }

    /// Create an owned Sandlot [`String`] from this [`Str`].
    ///
    /// This is a hack to work around [`ToOwned`] not being implementable
    /// for this type.
    pub fn to_owned(self) -> String {
        let dup = unsafe { SDL_strdup(self.as_ptr()) };

        // SAFETY: `SDL_strdup` can only fail in an OOM scenario.
        // If that happens, you've got bigger fish to fry.
        unsafe { String::from_ptr_unchecked(dup) }
    }
}

/// An enumeration of things that can go wrong when trying to convert
/// a slice of bytes to a nul-terminated UTF-8 [`Str`].
#[derive(Clone, Copy)]
pub enum FromUtf8BytesWithNulError {
    /// [`str::from_utf8`] failed.
    Utf8Error(Utf8Error),
    /// [`CStr::from_bytes_with_nul`] failed.
    FromBytesWithNulError(FromBytesWithNulError),
}

impl Display for FromUtf8BytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Utf8Error(u) => Display::fmt(u, f),
            Self::FromBytesWithNulError(n) => Display::fmt(n, f),
        }
    }
}

impl Debug for FromUtf8BytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Utf8Error(u) => Debug::fmt(u, f),
            Self::FromBytesWithNulError(n) => Debug::fmt(n, f),
        }
    }
}

impl std::error::Error for FromUtf8BytesWithNulError {}

impl<'a> TryFrom<&'a [u8]> for Str<'a> {
    type Error = FromUtf8BytesWithNulError;

    /// Requires `value` to represent UTF-8 data and contain exactly one nul byte at the end.
    ///
    /// The UTF-8 check is performed first, then the nul check. If both pass, [`Ok`] is returned
    /// with a [`Str`] pointing to the bytes of `value`.
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if let Err(e) = str::from_utf8(value) {
            Err(FromUtf8BytesWithNulError::Utf8Error(e))
        } else if let Err(e) = CStr::from_bytes_with_nul(value) {
            Err(FromUtf8BytesWithNulError::FromBytesWithNulError(e))
        } else {
            Ok(unsafe { Self::from_ptr_unchecked(value.as_ptr().cast()) })
        }
    }
}

impl<'a> TryFrom<&'a str> for Str<'a> {
    type Error = FromBytesWithNulError;

    /// Requires `value` to contain exactly one nul byte at the end.
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        CStr::from_bytes_with_nul(value.as_bytes()).map(|_| unsafe {
            // SAFETY: `value` is not empty, thus its pointer won't be null.
            Self::from_ptr_unchecked(value.as_ptr().cast())
        })
    }
}

impl<'a> TryFrom<&'a CStr> for Str<'a> {
    type Error = Utf8Error;

    /// Requires `value` to be UTF-8.
    fn try_from(value: &'a CStr) -> Result<Self, Self::Error> {
        str::from_utf8(value.to_bytes())
            .map(|s| unsafe { Self::from_ptr_unchecked(s.as_ptr().cast()) })
    }
}

impl PartialEq for Str<'_> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.to_str(), other.to_str())
    }
}

impl Eq for Str<'_> {}

// Intentionally not comparable with `&[u8]`, since we don't know whether
// your slice also has the nul byte.

impl PartialEq<&str> for Str<'_> {
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.to_str(), *other)
    }
}

impl PartialEq<&CStr> for Str<'_> {
    fn eq(&self, other: &&CStr) -> bool {
        PartialEq::eq(self.to_bytes(), other.to_bytes())
    }
}

impl Display for Str<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.to_str(), f)
    }
}

impl Debug for Str<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.to_str(), f)
    }
}

/// Returns whether a byte slice has any nul bytes apart from the end.
///
/// Only intended for use in the [`s!`](crate::s!) macro.
#[doc(hidden)]
pub const fn has_interior_nul(bytes: &[u8]) -> bool {
    let mut i = 0;

    while i < bytes.len() - 1 {
        if bytes[i] == b'\0' {
            return true;
        }

        i += 1;
    }

    false
}

/// Safely create a static-lifetime [`Str`] from a string literal.
///
/// Ensures, at compile time, that the passed string literal has no
/// interior nul bytes.
///
/// This is a hack to reduce typing when passing literals to places
/// which expect [`Str`], inspired by [`CStr`] literal syntax.
/// Where you'd write `c"Hello World!"`to create a [`CStr`], you'd
/// instead write `s!("Hello World!")` to create a [`Str`].
#[macro_export]
macro_rules! s {
    ($s:literal) => {{
        const LIT: &str = ::core::concat!($s, "\0");
        const _: () = ::core::assert!(!$crate::str::has_interior_nul(LIT.as_bytes()));

        unsafe { $crate::str::Str::from_ptr_unchecked(LIT.as_ptr().cast()) }
    }};
}
