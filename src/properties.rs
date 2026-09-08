//! SDL properties API wrapper.
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
//! sandlot exposes an intuitive builder for each such object via the associated `builder()` function.
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
//! These contain the required creation fields, while a resource builder attaches a property group
//! for setting further options (SDL calls them "extensions"). GPU builders are created from the
//! object being built, rather than from its `CreateInfo` struct. Two build options are provided:
//!
//! ```rust,ignore
//! fn build(&self, device: Ref<Device>, ci: FooCreateInfo) -> Result<Foo>;
//! fn build_cleanup(&self, device: Ref<Device>, ci: FooCreateInfo) -> Result<Foo>;
//! ```
//!
//! First create a `CreateInfo` with its associated `new` function, then pass it to the resource
//! builder. `build` attaches the builder's properties and creates the object. `build_cleanup` does
//! the same, then clears the creation properties from the builder's property group.
//!
//! ```rust,ignore
//! let create_info = TextureCreateInfo::new(/* required fields */);
//! let texture = Texture::builder(props.as_ref()).build(device.as_ref(), create_info)?;
//! ```
//!
//! The `CreateInfo` value is passed by value because the builder adds the property group just before
//! creation. Its lifetime parameters therefore describe only other borrowed creation data, not the
//! properties group.
//!
//! # API checklist ([source](https://wiki.libsdl.org/SDL3/CategoryProperties))
//! - [x] SDL_ClearProperty
//! - [x] SDL_CopyProperties
//! - [x] SDL_CreateProperties
//! - [x] SDL_DestroyProperties
//! - [x] SDL_EnumerateProperties
//! - [x] SDL_GetBooleanProperty
//! - [x] SDL_GetFloatProperty
//! - [x] SDL_GetGlobalProperties
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
    resource::{Handle, Ref, Resource},
    util::{opt2res_map, to_result},
};

/// An ID that represents a properties set.
///
/// While this looks like an integer to the application, SDL properties are
/// actually key/value stores that can manage sets of information with
/// multiple datatypes.
#[derive(Clone, Copy)]
#[doc(alias = "SDL_PropertiesID")]
pub struct PropertiesHandle {
    pub(crate) handle: NonZero<u32>,
}

impl PropertiesHandle {
    pub(crate) fn from_id(handle: SDL_PropertiesID) -> Option<Self> {
        NonZero::new(handle.0).map(|handle| Self { handle })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetNumberProperty")]
    pub unsafe fn number(self, key: *const c_char, default: i64) -> i64 {
        unsafe { SDL_GetNumberProperty(self.id(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetFloatProperty")]
    pub unsafe fn float(self, key: *const c_char, default: f32) -> f32 {
        unsafe { SDL_GetFloatProperty(self.id(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetPointerProperty")]
    pub unsafe fn pointer(self, key: *const c_char, default: *mut c_void) -> *mut c_void {
        unsafe { SDL_GetPointerProperty(self.id(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetStringProperty")]
    pub unsafe fn string(self, key: *const c_char, default: *const c_char) -> *const c_char {
        unsafe { SDL_GetStringProperty(self.id(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_GetBooleanProperty")]
    pub unsafe fn bool(self, key: *const c_char, default: bool) -> bool {
        unsafe { SDL_GetBooleanProperty(self.id(), key, default) }
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetNumberProperty")]
    pub unsafe fn set_number(self, key: *const c_char, value: i64) -> Result<()> {
        to_result(unsafe { SDL_SetNumberProperty(self.id(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetFloatProperty")]
    pub unsafe fn set_float(self, key: *const c_char, value: f32) -> Result<()> {
        to_result(unsafe { SDL_SetFloatProperty(self.id(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetPointerProperty")]
    pub unsafe fn set_pointer(self, key: *const c_char, value: *mut c_void) -> Result<()> {
        to_result(unsafe { SDL_SetPointerProperty(self.id(), key, value) })
    }

    /// # Safety
    /// `key` and `value` must be valid, null-terminated C strings.
    #[doc(alias = "SDL_SetStringProperty")]
    pub unsafe fn set_string(self, key: *const c_char, value: *const c_char) -> Result<()> {
        to_result(unsafe { SDL_SetStringProperty(self.id(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_SetBooleanProperty")]
    pub unsafe fn set_bool(self, key: *const c_char, value: bool) -> Result<()> {
        to_result(unsafe { SDL_SetBooleanProperty(self.id(), key, value) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_ClearProperty")]
    pub unsafe fn clear(self, key: *const c_char) -> Result<()> {
        to_result(unsafe { SDL_ClearProperty(self.id(), key) })
    }

    /// Copy a group of properties.
    ///
    /// Copies all the properties from one group of properties to another,
    /// with the exception of properties requiring cleanup, which will not be
    /// copied. Any property that already exists on `dst` will be
    /// overwritten.
    #[doc(alias = "SDL_CopyProperties")]
    fn copy_to(self, dst: Ref<Properties>) -> Result<()> {
        to_result(unsafe { SDL_CopyProperties(self.id(), dst.id()) })
    }

    /// # Safety
    /// `key` must be a valid, null-terminated C string.
    #[doc(alias = "SDL_HasProperty")]
    pub unsafe fn has(self, key: *const c_char) -> bool {
        unsafe { SDL_HasProperty(self.id(), key) }
    }

    /// Get the type of a property in a group of properties.
    ///
    /// Returns the type of the property, or invalid if it is not set.
    #[doc(alias = "SDL_GetPropertyType")]
    unsafe fn type_of(self, key: *const c_char) -> SDL_PropertyType {
        unsafe { SDL_GetPropertyType(self.id(), key) }
    }

    pub(crate) fn id(self) -> SDL_PropertiesID {
        SDL_PropertiesID::new(self.handle.get())
    }
}

/// An ID that represents a properties set.
///
/// While this looks like an integer to the application, SDL properties are
/// actually key/value stores that can manage sets of information with
/// multiple datatypes.
#[doc(alias = "SDL_PropertiesID")]
pub struct Properties {
    pub(crate) inner: PropertiesHandle,
}

impl Properties {
    pub(crate) fn from_id(handle: SDL_PropertiesID) -> Result<Self> {
        opt2res_map(NonZero::new(handle.0), |handle| Self {
            inner: PropertiesHandle { handle },
        })
    }

    /// Create a group of properties.
    ///
    /// # Remarks
    ///
    /// All properties are automatically destroyed when SDL quits.
    #[doc(alias = "SDL_CreateProperties")]
    pub fn new() -> Result<Self> {
        let id = unsafe { SDL_CreateProperties() };
        match PropertiesHandle::from_id(id) {
            Some(inner) => Ok(Self { inner }),
            None => Err(Error::current()),
        }
    }

    /// Get the global SDL properties.
    #[doc(alias = "SDL_GetGlobalProperties")]
    pub fn global() -> Result<Ref<'static, Properties>> {
        let id = unsafe { SDL_GetGlobalProperties() };
        match PropertiesHandle::from_id(id) {
            Some(p) => Ok(unsafe { Ref::from_handle(p) }),
            None => Err(Error::current()),
        }
    }
}

impl std::ops::Deref for Properties {
    type Target = PropertiesHandle;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Properties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Handle for PropertiesHandle {
    type Raw = u32;
    type Inner = NonZero<Self::Raw>;

    fn as_raw(&self) -> Self::Raw {
        self.handle.get()
    }

    fn as_inner(&self) -> Self::Inner {
        self.handle
    }
}

impl Resource for Properties {
    type Handle = PropertiesHandle;
    unsafe fn as_handle(&self) -> Self::Handle {
        self.inner
    }
}

impl Drop for Properties {
    /// Destroy a group of properties.
    ///
    /// All properties are deleted and their cleanup functions will be
    /// called, if any.
    #[doc(alias = "SDL_DestroyProperties")]
    fn drop(&mut self) {
        unsafe { SDL_DestroyProperties(self.id()) }
    }
}
