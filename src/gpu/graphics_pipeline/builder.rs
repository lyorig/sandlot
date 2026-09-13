use crate::{Result, gpu::Device, properties::Properties, resource::Ref};

use super::{GraphicsPipeline, GraphicsPipelineCreateInfo};

use sdl3_sys::gpu::*;
use std::ffi::{CStr, c_char};

const CREATE_PROPERTIES: [*const c_char; 1] = [SDL_PROP_GPU_GRAPHICSPIPELINE_CREATE_NAME_STRING];

/// Builder for [`GraphicsPipelineCreateInfo`] properties.
#[derive(Clone, Copy)]
pub struct GraphicsPipelineBuilder<'p> {
    props: Ref<'p, Properties>,
}

impl<'p> GraphicsPipelineBuilder<'p> {
    pub(super) fn new(props: Ref<'p, Properties>) -> Self {
        Self { props }
    }

    /// A name for the graphics pipeline, used for debugging.
    #[doc(alias = "SDL_PROP_GPU_GRAPHICSPIPELINE_CREATE_NAME_STRING")]
    pub fn name(&mut self, value: &CStr) -> &mut Self {
        _ = unsafe {
            self.props.set_string(
                SDL_PROP_GPU_GRAPHICSPIPELINE_CREATE_NAME_STRING,
                value.as_ptr(),
            )
        };
        self
    }

    /// Clear all GPU graphics pipeline creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    pub fn build<'ctx, 'vid, 'dev>(
        &self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        mut create_info: GraphicsPipelineCreateInfo,
    ) -> Result<GraphicsPipeline<'ctx, 'vid, 'dev>> {
        create_info.0.props = self.props.id();
        GraphicsPipeline::new(device, &create_info)
    }

    /// Creates a [`GraphicsPipeline`] using [`GraphicsPipelineCreateInfo`],
    /// then removes all graphics pipeline creation properties from the attached property group.
    pub fn build_cleanup<'ctx, 'vid, 'dev>(
        &self,
        device: Ref<'dev, Device<'ctx, 'vid>>,
        create_info: GraphicsPipelineCreateInfo,
    ) -> Result<GraphicsPipeline<'ctx, 'vid, 'dev>> {
        let res = self.build(device, create_info);
        Self::clear_from(self.props);
        res
    }
}
