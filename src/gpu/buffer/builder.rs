use std::ffi::{CStr, c_char};

use sdl3_sys::gpu::*;

use crate::{Result, gpu::Device, properties::Properties, resource::Ref};

use super::{Buffer, BufferCreateInfo};

const CREATE_PROPERTIES: [*const c_char; 1] = [SDL_PROP_GPU_BUFFER_CREATE_NAME_STRING];

/// Builder for [`BufferCreateInfo`] properties.
#[derive(Clone, Copy)]
pub struct BufferBuilder<'p> {
    props: Ref<'p, Properties>,
}

impl<'p> BufferBuilder<'p> {
    pub(super) fn new(props: Ref<'p, Properties>) -> Self {
        Self { props }
    }

    /// A name for the buffer, used for debugging.
    #[doc(alias = "SDL_PROP_GPU_BUFFER_CREATE_NAME_STRING")]
    pub fn name(self, value: &CStr) -> Self {
        _ = unsafe {
            self.props
                .set_string(SDL_PROP_GPU_BUFFER_CREATE_NAME_STRING, value.as_ptr())
        };
        self
    }

    /// Clear all GPU buffer creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    pub fn build<'ctx, 'vid, 'dev>(
        self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        mut create_info: BufferCreateInfo,
    ) -> Result<Buffer<'ctx, 'vid, 'dev>> {
        create_info.0.props = self.props.id();
        Buffer::new(device, &create_info)
    }

    /// Creates a [`Buffer`] using [`BufferCreateInfo`],
    /// then removes all buffer creation properties from the attached property group.
    pub fn build_cleanup<'ctx, 'vid, 'dev>(
        self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        create_info: BufferCreateInfo,
    ) -> Result<Buffer<'ctx, 'vid, 'dev>> {
        let res = self.build(device, create_info);
        Self::clear_from(self.props);
        res
    }
}
