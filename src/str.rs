use std::{
    ffi::{CStr, c_char},
    fmt::{Debug, Display},
    marker::PhantomData,
    ptr::NonNull,
};

/// Holds a pointer to a nul-terminated string which is guaranteed UTF-8.
///
/// This enables passing pointers to and from SDL functions without having to
/// store the length. At the same time, infallible conversion methods to
/// [`str`] and [`CStr`] exist, owing to the aforementioned guarantees.
///
/// This struct is meant to be bound to some object with lifetime `'a`.
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
    ptr: NonNull<c_char>,
    marker: PhantomData<&'a ()>,
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
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr_unchecked(ptr) })
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
        unsafe { Self::from_nonnull(ptr) }
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
    pub const unsafe fn from_nonnull(ptr: NonNull<c_char>) -> Self {
        Self {
            ptr,
            marker: PhantomData,
        }
    }

    pub const fn as_nonnull(self) -> NonNull<c_char> {
        self.ptr
    }

    pub const fn as_ptr(self) -> *const c_char {
        self.ptr.as_ptr()
    }

    pub const fn to_cstr(self) -> &'a CStr {
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    pub const fn to_bytes(self) -> &'a [u8] {
        self.to_cstr().to_bytes()
    }

    pub fn to_bytes_with_nul(self) -> &'a [u8] {
        self.to_cstr().to_bytes_with_nul()
    }

    pub const fn to_str(self) -> &'a str {
        unsafe { str::from_utf8_unchecked(self.to_bytes()) }
    }

    /// Analogous to [`CStr::count_bytes`].
    pub const fn count_bytes(self) -> usize {
        self.to_cstr().count_bytes()
    }

    pub const fn is_empty(self) -> bool {
        // First byte is the nul terminator? Then the string is empty.
        // SAFETY: `ptr` is guaranteed to not be null and be readable for at least a single byte.
        (unsafe { self.ptr.read() }) == 0
    }
}

impl<'a> TryFrom<&'a [u8]> for Str<'a> {
    type Error = ();

    /// Requires `value` to represent UTF-8 data and contain exactly one nul byte at the end.
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value
            .iter()
            .position(|&b| b == 0)
            .is_some_and(|p| p == value.len())
        {
            Ok(unsafe { Self::from_ptr_unchecked(value.as_ptr().cast()) })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<&'a str> for Str<'a> {
    type Error = ();

    /// Requires `value` to contain exactly one nul byte at the end.
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value
            .bytes()
            .position(|b| b == b'\0')
            .is_some_and(|p| p == value.len())
        {
            Ok(unsafe { Self::from_ptr_unchecked(value.as_ptr().cast()) })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<&'a CStr> for Str<'a> {
    type Error = ();

    /// Requires `value` to be UTF-8.
    fn try_from(value: &'a CStr) -> Result<Self, Self::Error> {
        match str::from_utf8(value.to_bytes()) {
            Ok(_) => Ok(unsafe { Self::from_ptr_unchecked(value.as_ptr().cast()) }),
            Err(_) => Err(()),
        }
    }
}

impl PartialEq for Str<'_> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.to_str(), other.to_str())
    }
}

impl Eq for Str<'_> {}

impl PartialEq<&[u8]> for Str<'_> {
    fn eq(&self, other: &&[u8]) -> bool {
        PartialEq::eq(self.to_bytes(), *other)
    }
}

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.to_str(), f)
    }
}

impl Debug for Str<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.to_str(), f)
    }
}

/// Used in the [`s!`](crate::s!) macro to validate the passed string literal.
pub const fn has_nul_byte(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
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
        const _: () = ::core::assert!(!$crate::str::has_nul_byte($s));
        unsafe {
            $crate::str::Str::<'static>::from_ptr_unchecked(concat!($s, "\0").as_ptr().cast())
        }
    }};
}
