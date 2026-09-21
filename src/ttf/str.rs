use std::ffi::c_char;

/// A string that certain TTF functions can safely read.
///
/// Many TTF functions take a pointer and a byte length.
/// A length of zero means that the pointer refers to a nul-terminated string.
/// As Rust strings do not have a nul terminator, an empty [`&str`](str) would make the
/// function read past the end of the string. This struct prevents that error.
///
/// Construct a "TTF-ready" string from a `&str`:
/// - [`TtfStr::new`] checks for an empty string. In that case, the pointer it set to `c""`.
/// - [`TtfStr::new_unchecked`] skips the check. Use when you know that the string is not empty.
#[derive(Clone, Copy)]
pub struct TtfStr<'a> {
    first_byte: &'a c_char,
    len: usize,
}

impl<'a> TtfStr<'a> {
    pub const fn new(s: &'a str) -> Self {
        let len = s.len();
        let first_byte = if len == 0 {
            // SAFETY: Guaranteed to be valid for exactly one (nul) byte.
            unsafe { c"".as_ptr().as_ref_unchecked() }
        } else {
            // SAFETY: Guaranteed to be valid for at least `len` bytes.
            unsafe { s.as_ptr().cast::<c_char>().as_ref_unchecked() }
        };

        Self { first_byte, len }
    }

    /// # Safety
    ///
    /// `s` must not be empty.
    pub const unsafe fn new_unchecked(s: &'a str) -> Self {
        unsafe { std::mem::transmute(s) }
    }

    pub const fn as_ptr(&self) -> *const c_char {
        std::ptr::from_ref(self.first_byte).cast()
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}
