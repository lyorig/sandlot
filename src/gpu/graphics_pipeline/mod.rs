//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_BindGPUGraphicsPipeline
//! - [x] SDL_CreateGPUGraphicsPipeline
//! - [x] SDL_ReleaseGPUGraphicsPipeline

use std::marker::PhantomData;

use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result,
    gpu::{ColorTargetDescription, VertexAttribute, VertexBufferDescription},
    properties::Properties,
    resource::{Ref, Resource},
    util::impl_enum_transmute,
    util::mod_reexport,
    util::resource_new_no_drop,
};

use super::{
    device::Device,
    pipeline_state::{
        DepthStencilState, GraphicsPipelineTargetInfo, MultisampleState, RasterizerState,
        VertexInputState,
    },
    render_pass::RenderPass,
    shader::Shader,
};

mod_reexport!(builder);

/// The primitive topology of a graphics pipeline.
///
/// When using [`Self::PointList`], the vertex shader must output a point size:
/// HLSL targeting SPIR-V uses `[[vk::builtin("PointSize")]]`, GLSL uses
/// `gl_PointSize`, and MSL uses `[[point_size]]`. Sized points are not supported
/// by D3D12; point sizes other than 1 are ignored.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUPrimitiveType")]
pub enum PrimitiveType {
    /// A series of separate triangles.
    TriangleList = SDL_GPUPrimitiveType::TRIANGLELIST.0,
    /// A series of connected triangles.
    TriangleStrip = SDL_GPUPrimitiveType::TRIANGLESTRIP.0,
    /// A series of separate lines.
    LineList = SDL_GPUPrimitiveType::LINELIST.0,
    /// A series of connected lines.
    LineStrip = SDL_GPUPrimitiveType::LINESTRIP.0,
    /// A series of separate points.
    PointList = SDL_GPUPrimitiveType::POINTLIST.0,
}

impl_enum_transmute!(SDL_GPUPrimitiveType, PrimitiveType);

/// Parameters for creating a graphics pipeline state.
///
/// The create info borrows the vertex and fragment shaders, vertex-buffer
/// descriptions, vertex attributes, and color-target descriptions for the
/// lifetimes encoded in its type. Those resources and slices must remain valid
/// while the create info is used to create the pipeline.
///
/// The wrapper sets SDL's extension-property ID to zero because extensions are
/// not exposed by this constructor.
#[doc(alias = "SDL_GPUGraphicsPipelineCreateInfo")]
#[derive(Clone, Copy)]
pub struct GraphicsPipelineCreateInfo<'vs, 'fs, 'vbd, 'va, 'ctd>(
    SDL_GPUGraphicsPipelineCreateInfo,
    PhantomData<Ref<'vs, Shader>>,
    PhantomData<Ref<'fs, Shader>>,
    PhantomData<&'vbd [VertexBufferDescription]>,
    PhantomData<&'va [VertexAttribute]>,
    PhantomData<&'ctd [ColorTargetDescription]>,
);
impl<'vs, 'fs, 'vbd, 'va, 'ctd> GraphicsPipelineCreateInfo<'vs, 'fs, 'vbd, 'va, 'ctd> {
    /// Describe the shaders and fixed-function state of a graphics pipeline.
    ///
    /// * `vertex_shader` and `fragment_shader` are the shaders used by the pipeline.
    /// * `vertex_input_state` describes the vertex layout.
    /// * `primitive_type` specifies the primitive topology.
    /// * `rasterizer_state` specifies rasterization behavior.
    /// * `multisample_state` specifies multisampling behavior.
    /// * `depth_stencil_state` specifies depth and stencil behavior.
    /// * `target_info` specifies render-target formats and blend modes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vertex_shader: Ref<'vs, Shader>,
        fragment_shader: Ref<'fs, Shader>,
        vertex_input_state: VertexInputState<'vbd, 'va>,
        primitive_type: PrimitiveType,
        rasterizer_state: RasterizerState,
        multisample_state: MultisampleState,
        depth_stencil_state: DepthStencilState,
        target_info: GraphicsPipelineTargetInfo<'ctd>,
    ) -> Self {
        Self(
            SDL_GPUGraphicsPipelineCreateInfo {
                vertex_shader: vertex_shader.handle.as_ptr(),
                fragment_shader: fragment_shader.handle.as_ptr(),
                vertex_input_state: vertex_input_state.0,
                primitive_type: SDL_GPUPrimitiveType::new(primitive_type as _),
                rasterizer_state: rasterizer_state.0,
                multisample_state: multisample_state.0,
                depth_stencil_state: depth_stencil_state.0,
                target_info: target_info.0,
                props: SDL_PropertiesID::new(0),
            },
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a graphics pipeline.
    /// Used during render passes.
    SDL_GPUGraphicsPipeline, GraphicsPipeline
);
impl GraphicsPipeline {
    /// Build a [`GraphicsPipeline`] with additional parameters not available in [`GraphicsPipelineCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> GraphicsPipelineBuilder<'_> {
        GraphicsPipelineBuilder::new(props)
    }

    /// Create a pipeline object for a graphics workflow.
    ///
    /// `device` is the GPU device that owns the pipeline, and `create_info`
    /// describes the shaders, vertex layout, primitive topology, fixed-function
    /// state, and render-target configuration.
    ///
    /// Returns [`Err`] if the graphics pipeline cannot be created.
    #[doc(alias = "SDL_CreateGPUGraphicsPipeline")]
    pub fn new(device: Ref<Device>, create_info: &GraphicsPipelineCreateInfo) -> Result<Self> {
        let handle = unsafe {
            SDL_CreateGPUGraphicsPipeline(device.handle.as_ptr(), &raw const create_info.0)
        };

        Self::from_ptr(handle)
    }

    /// Convenience function that creates a [`GraphicsPipeline`],
    /// does some work on it, then drops it.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`GraphicsPipeline::new`]
    /// - `op`
    pub fn with<F: FnOnce(Ref<Self>) -> Result<()>>(
        device: Ref<Device>,
        create_info: &GraphicsPipelineCreateInfo,
        f: F,
    ) -> Result<()> {
        let gp = Self::new(device, create_info)?;
        f(gp.as_ref())?;
        gp.drop(device);

        Ok(())
    }

    /// Release a graphics pipeline as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the pipeline. This method consumes
    /// the pipeline; it must not be referenced after this call. Unlike ordinary
    /// RAII resources, a graphics pipeline created with this module has no
    /// automatic destructor, so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUGraphicsPipeline")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUGraphicsPipeline(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}

impl GraphicsPipelineHandle {
    /// Bind this graphics pipeline to a render pass for rendering.
    ///
    /// `render_pass` is the render pass that will use the pipeline. A graphics
    /// pipeline must be bound before making draw calls.
    #[doc(alias = "SDL_BindGPUGraphicsPipeline")]
    pub fn bind(&self, render_pass: Ref<RenderPass>) {
        unsafe { SDL_BindGPUGraphicsPipeline(render_pass.handle.as_ptr(), self.handle.as_ptr()) };
    }
}
