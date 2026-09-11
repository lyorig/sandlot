use std::ffi::{CStr, c_char};

use crate::{Result, error::Error};

/// Defines a private submodule and publicly re-exports everything within.
/// Use when you want to compartmentalize things in a module, but also have
/// everything available at the module level.
macro_rules! mod_reexport {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

pub(crate) use mod_reexport;

/// Define an enum with two variants, `No` and `Yes`.
///
/// Interconvertible with `bool` via `From`.
macro_rules! boolenum {
    ($(#[$meta:meta])* $name:ident) => {
        #[repr(u8)]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        pub enum $name {
            No = false as _,
            Yes = true as _,
        }

        impl From<$name> for ::core::primitive::bool {
            fn from(value: $name) -> Self {
                match value {
                    $name::No => false,
                    $name::Yes => true,
                }
            }
        }

        impl From<::core::primitive::bool> for $name {
            fn from(value: ::core::primitive::bool) -> Self {
                match value {
                    false => Self::No,
                    true => Self::Yes,
                }
            }
        }
    };
}

pub(crate) use boolenum;

/// Implement bidirectional [`From`] for two enums using [`std::mem::transmute`].
///
/// Intended for use with Sandlot's SDL enum wrappers.
///
/// Defines two `const fn`s:
/// - `$wrap::from_sdl(self)`, which is `unsafe` since SDL often has "invalid" enum variants (i.e. `SDL_SCALEMODE_INVALID`)
///   for marking error states in functions etc. Sandlot leaves these out, instead opting to use [`None`] or [`Err`] to indicate errors.
/// - `$wrap::to_sdl($sdl)`, which is infallible since Sandlot enums are always a subset of their SDL counterparts.
///
/// The macro ensures the following pre-requisites at compile-time:
/// - the two types have equal size (via [`std::mem::size_of`])
/// - both types implement [`Copy`]
///
/// It is otherwise your responsibility to ensure transmuting between both types is sound.
macro_rules! impl_enum_transmute {
    ($sdl:ident, $wrap:ident) => {
        const _: () = assert!(::std::mem::size_of::<$sdl>() == ::std::mem::size_of::<$wrap>());

        impl $crate::util::IsCopy for $sdl {}
        impl $crate::util::IsCopy for $wrap {}

        impl $wrap {
            /// Construct this enum from its SDL equivalent.
            ///
            /// # Safety
            /// The caller must ensure that `value` is a valid enum variant of [`Self`],
            /// e.g. `ScaleMode` cannot be constructed from `SDL_SCALEMODE_INVALID`.
            pub const unsafe fn from_sdl_unchecked(value: $sdl) -> Self {
                unsafe { ::std::mem::transmute(value) }
            }

            /// Convert this enum to its SDL equivalent.
            /// This is the inverse of [`Self::from_sdl_unchecked`], and is infallible, owing to [`Self`] being a subset
            /// of the enum it's wrapping.
            pub const fn to_sdl(self) -> $sdl {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl From<$wrap> for $sdl {
            fn from(value: $wrap) -> Self {
                value.to_sdl()
            }
        }
    };

    ($sdl:ident, $wrap:ident, $invalid:ident) => {
        $crate::util::impl_enum_transmute!($sdl, $wrap);

        impl $wrap {
            /// Construct this enum from its SDL equivalent.
            ///
            /// Returns [`None`] if `value` is the "invalid" sentinel value
            /// (e.g. `SDL_PixelFormat::UNKNOWN`).
            pub fn from_sdl(value: $sdl) -> Option<Self> {
                if value == $sdl::$invalid {
                    None
                } else {
                    Some(unsafe { Self::from_sdl_unchecked(value) })
                }
            }
        }
    };
}

pub(crate) use impl_enum_transmute;

/// Converts an [`Option`] holding a reference to a pointer.
/// As you would expect, [`None`] produces [`std::ptr::null`], while
/// [`Some`] returns `&T` as a pointer.
///
/// This function's purpose is to facilitate interfacing with C FFI libraries.
pub(crate) fn opt2ptr<T>(opt: Option<&T>) -> *const T {
    opt.map_or(std::ptr::null(), |s| s)
}

/// Analogous to [`opt2ptr`], but for mutable references.
pub(crate) fn opt2ptr_mut<T>(opt: Option<&mut T>) -> *mut T {
    opt.map_or(std::ptr::null_mut(), |s| s)
}

/// Convenience function that converts an [`Option<T>`] to
/// a [`Result`], getting the current error if it is [`None`].
pub(crate) fn opt2res<T>(opt: Option<T>) -> Result<T> {
    match opt {
        Some(s) => Ok(s),
        None => Err(Error::current()),
    }
}

/// Convenience function that converts an [`Option<T>`] to
/// a [`Result<U>`], getting the current error if it is [`None`].
pub(crate) fn opt2res_map<T, U, F: FnOnce(T) -> U>(opt: Option<T>, f: F) -> Result<U> {
    match opt {
        Some(s) => Ok(f(s)),
        None => Err(Error::current()),
    }
}

/// Returns [`Ok`] if `result`, otherwise [`Err`] with [`Error::current`].
pub(crate) fn to_result(result: bool) -> Result<()> {
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
pub(crate) unsafe fn c_ptr_to_str<'a>(ptr: *const c_char) -> &'a str {
    unsafe { str::from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()) }
}

/// Marker trait for asserting that a type is [`Copy`].
pub(crate) trait IsCopy: Copy {}
