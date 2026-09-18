use std::{ffi::c_char, marker::PhantomData};

use sdl3_sys::gpu::*;

use crate::{gpu::Device, properties::Properties, resource::Ref, str::Str};

#[derive(Clone, Copy)]
pub struct DeviceProperties<'ctx, 'vid, 'dev> {
    inner: Ref<'dev, Properties>,
    marker: PhantomData<Ref<'dev, Device<'ctx, 'vid>>>,
}

impl<'ctx, 'vid, 'dev> DeviceProperties<'ctx, 'vid, 'dev> {
    pub(super) fn new(inner: Ref<'dev, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    fn get(self, key: *const c_char) -> Option<Str<'dev>> {
        let s = unsafe { self.inner.string(key, std::ptr::null()) };
        unsafe { Str::from_ptr(s) }
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_NAME_STRING")]
    pub fn device_name(self) -> Option<Str<'dev>> {
        self.get(SDL_PROP_GPU_DEVICE_NAME_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_NAME_STRING")]
    pub fn driver_name(self) -> Option<Str<'dev>> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_NAME_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_VERSION_STRING")]
    pub fn driver_version(self) -> Option<Str<'dev>> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_VERSION_STRING)
    }

    #[doc(alias = "SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING")]
    pub fn driver_info(self) -> Option<Str<'dev>> {
        self.get(SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING)
    }
}
