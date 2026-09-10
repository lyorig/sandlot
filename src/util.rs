use std::ffi::{CStr, c_char};

use crate::{Result, error::Error};

/// Analogous to C's `sizeof`. Can be used with a type or expression,
/// delegating to [`size_of`] or [`size_of_val`] appropriately.
#[macro_export]
macro_rules! size_of {
    ($t:ty) => {
        ::core::mem::size_of::<$t>()
    };
    ($e:expr) => {
        ::core::mem::size_of_val(&$e)
    }
}

/// Defines a private submodule and publicly re-exports everything within.
/// Use when you want to compartmentalize things in a module, but also have
/// everything available at the module level.
#[macro_export]
macro_rules! mod_reexport {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

/// Define an enum with two variants, `No` and `Yes`.
///
/// Interconvertible with `bool` via `From`.
#[macro_export]
macro_rules! boolenum {
    ($(#[$meta:meta])* $name:ident) => {
        #[repr(u8)]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        pub enum $name {
            No = false as _,
            Yes = true as _,
        }

        impl From<$name> for bool {
            fn from(value: $name) -> Self {
                match value {
                    $name::No => false,
                    $name::Yes => true,
                }
            }
        }

        impl From<bool> for $name {
            fn from(value: bool) -> Self {
                match value {
                    false => Self::No,
                    true => Self::Yes,
                }
            }
        }
    };
}

/// Implement bidirectional [`From`] for two enums.
///
/// Also defines two `const fn`s:
/// - `$wrap::from_sdl($sdl)`
/// - `$wrap::to_sdl(self)`
///
/// The macro ensures the following pre-requisites at compile-time:
/// - the two types have equal size ([`std::mem::size_of`])
/// - both types implement [`Copy`]
///
/// The conversion is done via [`std::mem::transmute`].
/// It is your responsibility to ensure its use for converting between both enums is sound.
#[macro_export]
macro_rules! impl_enum_transmute {
    ($sdl:ident, $wrap:ident) => {
        const _: () = assert!(::std::mem::size_of::<$sdl>() == ::std::mem::size_of::<$wrap>());

        impl $crate::util::IsCopy for $sdl {}
        impl $crate::util::IsCopy for $wrap {}

        impl $wrap {
            const fn from_sdl(value: $sdl) -> Self {
                unsafe { ::std::mem::transmute(value) }
            }

            const fn to_sdl(self) -> $sdl {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl From<$sdl> for $wrap {
            fn from(value: $sdl) -> Self {
                Self::from_sdl(value)
            }
        }

        impl From<$wrap> for $sdl {
            fn from(value: $wrap) -> Self {
                value.to_sdl()
            }
        }
    };
}

/// Converts an [`Option`] holding a reference to a pointer.
/// As you would expect, `None` produces [`std::ptr::null`], while
/// `Some` returns `&T` as a pointer.
///
/// This function's purpose is to facilitate interfacing with C FFI libraries.
pub fn opt2ptr<T>(opt: Option<&T>) -> *const T {
    opt.map_or(std::ptr::null(), |s| s)
}

/// Analogous to [`opt2ptr`], but for mutable references.
pub fn opt2ptr_mut<T>(opt: Option<&mut T>) -> *mut T {
    opt.map_or(std::ptr::null_mut(), |s| s)
}

/// Convenience function that converts an `Option<T>` to
/// a `Result`, getting the current error if it is `None`.
pub fn opt2res<T>(opt: Option<T>) -> Result<T> {
    match opt {
        Some(s) => Ok(s),
        None => Err(Error::current()),
    }
}

/// Convenience function that converts an `Option<T>` to
/// a `Result<U>`, getting the current error if it is `None`.
pub fn opt2res_map<T, U, F: FnOnce(T) -> U>(opt: Option<T>, f: F) -> Result<U> {
    match opt {
        Some(s) => Ok(f(s)),
        None => Err(Error::current()),
    }
}

/// Returns `Ok(())` if `result`, otherwise `Err(Error::current())`.
pub fn to_result(result: bool) -> Result<()> {
    if result {
        Ok(())
    } else {
        Err(Error::current())
    }
}

/// Convert a `NonNull<c_char>` (commonly used in FFI) to a `&str`.
///
/// # Safety
/// This function is VERY unsafe, a non-exhaustive list of assumptions:
/// - `ptr` points to a valid null-terminated C string
/// - the string pointed to by `ptr` is valid UTF-8
///
/// The returned value's lifetime is inferred from its usage (see [`CStr::from_ptr`]).
pub unsafe fn c_ptr_to_str<'a>(ptr: *const c_char) -> &'a str {
    unsafe { str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()) }
}

/// Marker trait for asserting that a type is `Copy`.
pub(crate) trait IsCopy: Copy {}
