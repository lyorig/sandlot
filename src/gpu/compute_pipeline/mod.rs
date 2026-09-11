//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUComputePipeline
//! - [x] SDL_ReleaseGPUComputePipeline

use std::{ffi::CStr, marker::PhantomData};

use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result, properties::Properties, resource::Ref, util::mod_reexport, util::resource_new_no_drop,
};

use super::{ShaderFormat, device::Device};

mod_reexport!(builder);

/// Parameters for creating a compute pipeline state.
///
/// The create info borrows the compute shader `code` and the UTF-8 shader
/// `entrypoint` name. Those values must remain valid while this create info is
/// used to create the pipeline. The wrapper sets SDL's extension-property ID to
/// zero because extensions are not exposed by this constructor.
#[doc(alias = "SDL_GPUComputePipelineCreateInfo")]
#[derive(Clone, Copy)]
pub struct ComputePipelineCreateInfo<'bc, 'ep>(
    SDL_GPUComputePipelineCreateInfo,
    PhantomData<&'bc [u8]>,
    PhantomData<&'ep CStr>,
);

impl<'bc, 'ep> ComputePipelineCreateInfo<'bc, 'ep> {
    /// Describe a compute pipeline's shader and resource layout.
    ///
    /// * `code` is the compute shader code.
    /// * `entrypoint` is the null-terminated UTF-8 entry-point function name.
    /// * `fmt` is the format of the shader code.
    /// * `sampler_count` is the number of samplers defined in the shader.
    /// * `uniform_buffer_count` is the number of uniform buffers defined in the shader.
    /// * The four storage counts are, in order, read-only storage textures,
    ///   read-only storage buffers, read-write storage textures, and read-write
    ///   storage buffers.
    /// * `threadcount_xyz` contains the number of threads in the X, Y, and Z
    ///   dimensions. These values should match the shader.
    ///
    /// The shader resource bindings must follow the convention for `fmt`:
    ///
    /// - SPIR-V: set 0 contains samplers, then read-only storage textures, then
    ///   read-only storage buffers; set 1 contains read-write storage textures,
    ///   then read-write storage buffers; set 2 contains uniform buffers. Each
    ///   set starts at binding 0 and has no gaps, with resources ordered as they
    ///   are bound through SDL.
    /// - DXBC/DXIL: samplers use `s[n]` in space 0; sampled textures, read-only
    ///   storage textures, and read-only storage buffers use consecutive `t[n]`
    ///   registers in space 0; read-write storage textures and buffers use
    ///   consecutive `u[n]` registers in space 1; uniform buffers use `b[n]` in
    ///   space 2. Each register set starts at index 0 and has no gaps.
    /// - MSL: samplers use the `sampler` table; sampled textures, then
    ///   read-only storage textures, then read-write storage textures use the
    ///   `texture` table; uniform buffers, then read-only storage buffers, then
    ///   read-write storage buffers use the `buffer` table. Each table starts at
    ///   index 0 and has no gaps, with resources ordered as they are bound.
    pub const fn new(
        code: &'bc [u8],
        entrypoint: &'ep CStr,
        fmt: ShaderFormat,
        sampler_count: u32,
        uniform_buffer_count: u32,
        (
            readonly_storage_texture_count,
            readonly_storage_buffer_count,
            readwrite_storage_texture_count,
            readwrite_storage_buffer_count,
        ): (u32, u32, u32, u32),
        threadcount_xyz: (u32, u32, u32),
    ) -> Self {
        let inner = SDL_GPUComputePipelineCreateInfo {
            code_size: code.len(),
            code: code.as_ptr(),
            entrypoint: entrypoint.as_ptr(),
            format: SDL_GPUShaderFormat::new(fmt as _),
            num_samplers: sampler_count,
            num_readonly_storage_textures: readonly_storage_texture_count,
            num_readonly_storage_buffers: readonly_storage_buffer_count,
            num_readwrite_storage_textures: readwrite_storage_texture_count,
            num_readwrite_storage_buffers: readwrite_storage_buffer_count,
            num_uniform_buffers: uniform_buffer_count,
            threadcount_x: threadcount_xyz.0,
            threadcount_y: threadcount_xyz.1,
            threadcount_z: threadcount_xyz.2,
            props: SDL_PropertiesID::new(0),
        };

        Self(inner, PhantomData, PhantomData)
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a compute pipeline.
    /// Used during compute passes.
    SDL_GPUComputePipeline, ComputePipeline
);
impl ComputePipeline {
    /// Build a [`ComputePipeline`] with additional parameters not available in [`ComputePipelineCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> ComputePipelineBuilder<'_> {
        ComputePipelineBuilder::new(props)
    }

    /// Create a pipeline object for a compute workflow.
    ///
    /// `device` is the GPU device that owns the pipeline, and `create_info`
    /// describes the compute shader, resource counts, and thread dimensions.
    /// Shader resource bindings must follow the convention for the shader format;
    /// see [`ComputePipelineCreateInfo::new`] for the required layouts.
    ///
    /// Returns [`Err`] if the pipeline cannot be created.
    #[doc(alias = "SDL_CreateGPUComputePipeline")]
    pub fn new(device: Ref<Device>, create_info: &ComputePipelineCreateInfo) -> Result<Self> {
        let handle = unsafe {
            SDL_CreateGPUComputePipeline(device.handle.as_ptr(), &raw const create_info.0)
        };

        Self::from_ptr(handle)
    }

    /// Release a compute pipeline as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the pipeline. This method consumes
    /// the pipeline; it must not be referenced after this call. Unlike ordinary
    /// RAII resources, a compute pipeline created with this module has no
    /// automatic destructor, so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUComputePipeline")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUComputePipeline(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}
