use std::{
    ffi::{CStr, c_char},
    fmt::{Debug, Display},
    mem,
    ops::Deref,
    str::Utf8Error,
};

/// A slice of bytes that combines [`&CStr`](CStr)'s promise of nul-termination,
/// and [`&str`](str)'s promise of UTF-8 contents.
///
/// Represents a non-owning view of string data coming from SDL.
pub struct Str {
    bytes: [u8],
}

impl Str {
    /// Analogous to [`CStr::from_ptr`], but with an extra UTF-8 requirement.
    ///
    /// # Safety
    ///
    /// The caller must uphold the safety contract of [`CStr::from_ptr`],
    /// and also ensure that the data pointer to by `ptr` is valid UTF-8.
    pub const unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        unsafe {
            let cs = CStr::from_ptr(ptr);
            Self::from_cstr_unchecked(cs)
        }
    }

    pub fn from_cstr(cs: &CStr) -> Result<&Self, Utf8Error> {
        cs.to_str()?;
        Ok(unsafe { Self::from_cstr_unchecked(cs) })
    }

    /// # Safety
    ///
    /// `cs` must contain nul-terminated UTF-8 data.
    pub const unsafe fn from_cstr_unchecked(cs: &CStr) -> &Self {
        unsafe { mem::transmute(cs.to_bytes_with_nul()) }
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

    pub fn as_ptr(&self) -> *const c_char {
        self.bytes.as_ptr().cast()
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl Eq for Str {}

impl PartialEq<CStr> for Str {
    fn eq(&self, other: &CStr) -> bool {
        self.as_cstr() == other
    }
}
