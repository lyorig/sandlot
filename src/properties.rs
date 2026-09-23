//! Property groups, used for configuring SDL objects.
//!
//! # What are properties?
//!
//! A property group ([`Properties`]) is essentially a map, where (in Rust terms):
//! - the key is a [`&CStr`](std::ffi::CStr)
//! - the value is one of {[`CString`](std::ffi::CString), [`i64`], [`f32`], [`bool`], [`*mut c_void`](std::ffi::c_void)}.
//!
//! SDL has begun using this API in release 3.2.0, and many of its objects are built
//! by setting certain values on a property group, then calling `SDL_Create*WithProperties()`.
//! This enables extensibility, and is an interesting case for wrapping in an intuitive API.
//!
//! # Builders
//! Since each [`Properties`]-constructible SDL object has a finite well-documented set of properties,
//! Sandlot exposes an intuitive builder for each such object via the associated `builder()` function.
//! Each builder "attaches" to an existing property group, enabling efficient memory usage.
//!
//! For example:
//!
//! ```rust
//! use sandlot::{window::Window, rect::Point, resource::Resource, properties::Properties};
//!
//! // you can also obtain a 'static reference to an existing
//! // global property group via `Properties::global()`
//! let props = Properties::new().unwrap();
//! let wnd = Window::builder(props.as_ref())
//!     .title(c"My Super Amazing Window")
//!     .size(Point::new(640, 480))
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Build-with-cleanup
//! Alongside the usual `.build()` method, builders also expose `build_cleanup()`, which
//! additionally removes all relevant properties from the property group it is attached to.
//!
//! This is useful when using a longer-lived property group, specifically:
//! - you don't want to keep the builder properties in memory, since they won't be used anymore
//! - you intend to re-use it to build something else, and don't want the earlier configuration
//!   to influence future builds
//!
//! # GPU object builders
//!
//! Many objects in the GPU submodule use a separate structure in place of constructor arguments,
//! e.g. [`Texture`](crate::gpu::Texture) uses [`TextureCreateInfo`](crate::gpu::TextureCreateInfo).
//! These internally contain the property ID field to construct with (SDL calls them "extensions"),
//! so some shenanigans had to be done in order to do maintain the "typical" builder API you'd see
//! for non-GPU structs.
//!
//! That is, the `CreateInfo` value is passed by value because the builder adds the property group
//! just before creation.
//!
//! ```rust,ignore
//! let create_info = TextureCreateInfo::new(/* required fields */);
//! let texture = Texture::builder(props.as_ref())
//!     .build(device.as_ref(), create_info)?;
//! ```
//!
//! # API checklist ([source](https://wiki.libsdl.org/SDL3/CategoryProperties))
//! - [x] SDL_ClearProperty
//! - [x] SDL_CopyProperties
//! - [x] SDL_CreateProperties
//! - [x] SDL_DestroyProperties
//! - [x] SDL_EnumerateProperties
//! - [x] SDL_GetBooleanProperty
//! - [x] SDL_GetFloatProperty
//! - [x] SDL_GetGlobalProperties (impl'd as [`ContextHandle::global_properties`])
//! - [x] SDL_GetNumberProperty
//! - [x] SDL_GetPointerProperty
//! - [x] SDL_GetPropertyType
//! - [x] SDL_GetStringProperty
//! - [x] SDL_HasProperty
//! - [x] SDL_SetBooleanProperty
//! - [x] SDL_SetFloatProperty
//! - [x] SDL_SetNumberProperty
//! - [x] SDL_SetPointerProperty
//! - [x] SDL_SetStringProperty
//!
//! Not planned/unavailable for implementation:
//! - SDL_GetNumProperties (since SDL 3.6.0)
//! - SDL_LockProperties
//! - SDL_SetPointerPropertyWithCleanup
//! - SDL_UnlockProperties

use std::{
    ffi::{c_char, c_void},
    num::NonZero,
};

use sdl3_sys::properties::*;

use crate::{
    Result,
    error::Error,
    resource::{Ref, resource_new},
    util::to_result,
};

#[expect(unused_imports)]
use crate::init::ContextHandle;

resource_new! {
    /// An ID that represents a properties set.
    ///
    /// While this looks like an integer to the application, SDL properties are
    /// actually key/value stores that can manage sets of information with
    /// multiple datatypes.
    pub struct Properties<> : SDL_PropertiesID {
        raw: SDL_PropertiesID,
        inner: NonZero<u32>,
        marker: PhantomData<()>,
    }

    /// Destroy a group of properties.
    ///
    /// All properties are deleted and their cleanup functions will be
    /// called, if any.
    ~SDL_DestroyProperties
}

impl PropertiesHandle {
    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetNumberProperty")]
    pub unsafe fn number(self, key: *const c_char, default: i64) -> i64 {
        unsafe { SDL_GetNumberProperty(self.as_raw(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetFloatProperty")]
    pub unsafe fn float(self, key: *const c_char, default: f32) -> f32 {
        unsafe { SDL_GetFloatProperty(self.as_raw(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetPointerProperty")]
    pub unsafe fn pointer(self, key: *const c_char, default: *mut c_void) -> *mut c_void {
        unsafe { SDL_GetPointerProperty(self.as_raw(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetStringProperty")]
    pub unsafe fn string(self, key: *const c_char, default: *const c_char) -> *const c_char {
        unsafe { SDL_GetStringProperty(self.as_raw(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetBooleanProperty")]
    pub unsafe fn bool(self, key: *const c_char, default: bool) -> bool {
        unsafe { SDL_GetBooleanProperty(self.as_raw(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetNumberProperty")]
    pub unsafe fn set_number(self, key: *const c_char, value: i64) -> Result<()> {
        to_result(unsafe { SDL_SetNumberProperty(self.as_raw(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetFloatProperty")]
    pub unsafe fn set_float(self, key: *const c_char, value: f32) -> Result<()> {
        to_result(unsafe { SDL_SetFloatProperty(self.as_raw(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetPointerProperty")]
    pub unsafe fn set_pointer(self, key: *const c_char, value: *mut c_void) -> Result<()> {
        to_result(unsafe { SDL_SetPointerProperty(self.as_raw(), key, value) })
    }

    /// # Safety
    /// `key` and `value` must be valid, null-terminated C strings.
    #[doc(alias = "SDL_SetStringProperty")]
    pub unsafe fn set_string(self, key: *const c_char, value: *const c_char) -> Result<()> {
        to_result(unsafe { SDL_SetStringProperty(self.as_raw(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetBooleanProperty")]
    pub unsafe fn set_bool(self, key: *const c_char, value: bool) -> Result<()> {
        to_result(unsafe { SDL_SetBooleanProperty(self.as_raw(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_ClearProperty")]
    pub unsafe fn clear(self, key: *const c_char) -> Result<()> {
        to_result(unsafe { SDL_ClearProperty(self.as_raw(), key) })
    }

    /// Copy a group of properties.
    ///
    /// Copies all the properties from one group of properties to another,
    /// with the exception of properties requiring cleanup, which will not be
    /// copied. Any property that already exists on `dst` will be
    /// overwritten.
    #[doc(alias = "SDL_CopyProperties")]
    fn copy_to(self, dst: Ref<Properties>) -> Result<()> {
        to_result(unsafe { SDL_CopyProperties(self.as_raw(), dst.as_raw()) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_HasProperty")]
    pub unsafe fn has(self, key: *const c_char) -> bool {
        unsafe { SDL_HasProperty(self.as_raw(), key) }
    }

    /// Get the type of a property in a group of properties.
    ///
    /// Returns the type of the property, or invalid if it is not set.
    #[doc(alias = "SDL_GetPropertyType")]
    unsafe fn type_of(self, key: *const c_char) -> SDL_PropertyType {
        unsafe { SDL_GetPropertyType(self.as_raw(), key) }
    }
}

impl Properties {
    /// Create a group of properties.
    ///
    /// # Remarks
    ///
    /// All properties are automatically destroyed when SDL quits.
    #[doc(alias = "SDL_CreateProperties")]
    pub fn new() -> Result<Self> {
        let id = unsafe { SDL_CreateProperties() };
        match PropertiesHandle::from_raw(id) {
            Some(inner) => Ok(Self { inner }),
            None => Err(Error::current()),
        }
    }
}
