use std::ffi::c_char;

use sdl3_sys::gpu::*;

use crate::{Result, gpu::Device, properties::Properties, resource::Ref, str::Str};

use super::{ComputePipeline, ComputePipelineCreateInfo};

const CREATE_PROPERTIES: [*const c_char; 1] = [SDL_PROP_GPU_COMPUTEPIPELINE_CREATE_NAME_STRING];

/// Builder for [`ComputePipelineCreateInfo`] properties.
#[derive(Clone, Copy)]
pub struct ComputePipelineBuilder<'p> {
    props: Ref<'p, Properties>,
}

impl<'p> ComputePipelineBuilder<'p> {
    pub(super) fn new(props: Ref<'p, Properties>) -> Self {
        Self { props }
    }

    /// A name for the compute pipeline, used for debugging.
    #[doc(alias = "SDL_PROP_GPU_COMPUTEPIPELINE_CREATE_NAME_STRING")]
    pub fn name(self, value: Str) -> Self {
        _ = unsafe {
            self.props.set_string(
                SDL_PROP_GPU_COMPUTEPIPELINE_CREATE_NAME_STRING,
                value.as_ptr(),
            )
        };
        self
    }

    /// Clear all GPU compute pipeline creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    pub fn build<'ctx, 'vid, 'dev>(
        self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        mut create_info: ComputePipelineCreateInfo<'_, '_>,
    ) -> Result<ComputePipeline<'ctx, 'vid, 'dev>> {
        create_info.0.props = self.props.id();
        ComputePipeline::new(device, &create_info)
    }

    /// Creates a [`ComputePipeline`] using [`ComputePipelineCreateInfo`],
    /// then removes all compute pipeline creation properties from the attached property group.
    pub fn build_cleanup<'ctx, 'vid, 'dev>(
        self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        create_info: ComputePipelineCreateInfo,
    ) -> Result<ComputePipeline<'ctx, 'vid, 'dev>> {
        let res = self.build(device, create_info);
        Self::clear_from(self.props);
        res
    }
}
