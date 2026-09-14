use std::ffi::{CStr, c_char};

use sdl3_sys::gpu::*;

use crate::{
    properties::{Properties, PropertiesHandle},
    resource::Ref,
};

#[derive(Clone, Copy)]
pub struct DeviceProperties<'a> {
    inner: Ref<'a, Properties>,
}

impl<'a> DeviceProperties<'a> {
    pub(super) fn new(inner: Ref<'a, Properties>) -> Self {
        Self { inner }
    }

    fn get(&self, key: *const c_char) -> Option<&str> {
        let s = unsafe { self.inner.string(key, std::ptr::null()) };
        if s.is_null() {
            None
        } else {
            unsafe {
                let cstr = CStr::from_ptr(s);
                Some(str::from_utf8_unchecked(cstr.to_bytes()))
            }
        }
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_NAME_STRING")]
    pub fn device_name(&self) -> Option<&str> {
        self.get(SDL_PROP_GPU_DEVICE_NAME_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_NAME_STRING")]
    pub fn driver_name(&self) -> Option<&str> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_NAME_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_VERSION_STRING")]
    pub fn driver_version(&self) -> Option<&str> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_VERSION_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING")]
    pub fn driver_info(&self) -> Option<&str> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING)
    }
}

impl std::ops::Deref for DeviceProperties<'_> {
    type Target = PropertiesHandle;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
