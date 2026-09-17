use std::{
    ffi::{CStr, c_char},
    fmt::Display,
    mem,
};

/// A slice of bytes that is both nul-terminated, and UTF-8.
pub struct Utf8CStr {
    bytes: [u8],
}

impl Utf8CStr {
    /// # Safety
    ///
    /// The caller must uphold the safety contract of [`CStr::from_ptr`],
    /// and also ensure that the data pointer to by `ptr` is valid UTF-8.
    pub const unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        unsafe {
            let cs = CStr::from_ptr(ptr);
            Self::from_cstr(cs)
        }
    }

    /// # Safety
    ///
    /// `cs` must contain nul-terminated UTF-8 data.
    pub const unsafe fn from_cstr(cs: &CStr) -> &Self {
        unsafe { mem::transmute(cs) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let bytes = self.as_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    pub const fn as_bytes_with_nul(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl Display for Utf8CStr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}
