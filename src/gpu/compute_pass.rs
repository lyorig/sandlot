//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_BeginGPUComputePass
//! - [x] SDL_BindGPUComputePipeline
//! - [x] SDL_BindGPUComputeSamplers
//! - [x] SDL_BindGPUComputeStorageBuffers
//! - [x] SDL_BindGPUComputeStorageTextures
//! - [x] SDL_DispatchGPUCompute
//! - [x] SDL_DispatchGPUComputeIndirect
//! - [x] SDL_EndGPUComputePass

use sdl3_sys::gpu::*;

use crate::{
    Result,
    resource::{Ref, Resource},
    util::resource_new,
};

use super::{
    buffer::{Buffer, StorageBufferReadWriteBinding},
    command_buffer::CommandBuffer,
    compute_pipeline::ComputePipeline,
    texture::{StorageTextureReadWriteBinding, Texture, TextureSamplerBinding},
};

// doc-only
#[expect(unused_imports)]
use super::texture::TextureUsageFlags;

resource_new!(
    /// An opaque handle representing a compute pass.
    /// Transient; invalid once the pass ends.
    SDL_GPUComputePass, ComputePass, SDL_EndGPUComputePass
);

/// Parameters of an indirect dispatch command.
///
/// Commands of this type are read by
/// [`ComputePassHandle::dispatch_indirect`] from a buffer at the given byte
/// offset, so they must be written there with a matching layout.
#[doc(alias = "SDL_GPUIndirectDispatchCommand")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndirectDispatchCommand(SDL_GPUIndirectDispatchCommand);

impl IndirectDispatchCommand {
    /// Describe the number of local workgroups to dispatch in the X, Y and Z
    /// dimensions.
    pub const fn new((x, y, z): (u32, u32, u32)) -> Self {
        Self(SDL_GPUIndirectDispatchCommand {
            groupcount_x: x,
            groupcount_y: y,
            groupcount_z: z,
        })
    }

    /// The number of local workgroups to dispatch in each dimension.
    pub const fn groupcounts(&self) -> (u32, u32, u32) {
        let c = &self.0;
        (c.groupcount_x, c.groupcount_y, c.groupcount_z)
    }
}
impl ComputePass {
    /// Begin a compute pass on a command buffer.
    ///
    /// `cmdbuf` is the command buffer that records the pass. The storage
    /// texture and buffer bindings declare the resources that compute pipelines
    /// may write during the pass. They must use textures created with [`TextureUsageFlags::COMPUTE_STORAGE_WRITE`]
    /// or [`TextureUsageFlags::COMPUTE_STORAGE_READ_WRITE`] and buffers created with [`TextureUsageFlags::COMPUTE_STORAGE_WRITE`].
    ///
    /// All compute operations must occur inside a compute pass, and no other
    /// compute, render, or copy pass may begin until this pass ends. Reads and
    /// writes within a pass are not implicitly synchronized: end the pass and
    /// begin another one before depending on the output of a previous dispatch.
    ///
    /// Returns [`Err`] if SDL cannot begin the pass.
    #[doc(alias = "SDL_BeginGPUComputePass")]
    pub fn new(
        cmdbuf: Ref<CommandBuffer>,
        storage_texture_bindings: &[StorageTextureReadWriteBinding],
        storage_buffer_bindings: &[StorageBufferReadWriteBinding],
    ) -> Result<Self> {
        let handle = unsafe {
            SDL_BeginGPUComputePass(
                cmdbuf.handle.as_ptr(),
                storage_texture_bindings.as_ptr().cast(),
                storage_texture_bindings.len() as _,
                storage_buffer_bindings.as_ptr().cast(),
                storage_buffer_bindings.len() as _,
            )
        };
        Self::from_ptr(handle)
    }

    /// Convenience function that creates a [`ComputePass`], does some work on it,
    /// then ends (drops) it.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`ComputePass::new`]
    /// - `op`
    pub fn run<F: FnOnce(Ref<Self>) -> Result<()>>(
        cmdbuf: Ref<CommandBuffer>,
        storage_texture_bindings: &[StorageTextureReadWriteBinding],
        storage_buffer_bindings: &[StorageBufferReadWriteBinding],
        op: F,
    ) -> Result<()> {
        let pass = Self::new(cmdbuf, storage_texture_bindings, storage_buffer_bindings)?;
        op(pass.as_ref())
    }
}

impl ComputePassHandle {
    /// Bind a compute pipeline for dispatches in this pass.
    ///
    /// `pipeline` is the compute pipeline to bind. A pipeline must be bound
    /// before dispatching compute work.
    #[doc(alias = "SDL_BindGPUComputePipeline")]
    pub fn bind(&self, pipeline: Ref<ComputePipeline>) {
        unsafe { SDL_BindGPUComputePipeline(self.handle.as_ptr(), pipeline.handle.as_ptr()) };
    }

    /// Bind texture-sampler pairs for use by the compute shader.
    ///
    /// `first_slot` is the first compute sampler slot, and `bindings` supplies
    /// consecutive slots from there. The textures must have been created with
    /// [`crate::gpu::texture::TextureUsageFlags::SAMPLER`].
    #[doc(alias = "SDL_BindGPUComputeSamplers")]
    pub fn bind_samplers(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUComputeSamplers(
                self.handle.as_ptr(),
                first_slot,
                bindings.as_ptr().cast(),
                bindings.len() as _,
            );
        }
    }

    /// Bind read-only storage textures for use by the compute shader.
    ///
    /// `first_slot` is the first compute storage-texture slot, and `textures`
    /// supplies consecutive slots from there. Each texture must have been
    /// created with [`crate::gpu::texture::TextureUsageFlags::COMPUTE_STORAGE_READ`].
    #[doc(alias = "SDL_BindGPUComputeStorageTextures")]
    pub fn bind_storage_textures(&self, first_slot: u32, textures: &[Ref<Texture>]) {
        unsafe {
            SDL_BindGPUComputeStorageTextures(
                self.handle.as_ptr(),
                first_slot,
                textures.as_ptr().cast(),
                textures.len() as _,
            );
        }
    }

    /// Bind read-only storage buffers for use by the compute shader.
    ///
    /// `first_slot` is the first compute storage-buffer slot, and `buffers`
    /// supplies consecutive slots from there. Each buffer must have been
    /// created with [`crate::gpu::buffer::BufferUsageFlags::COMPUTE_STORAGE_READ`].
    #[doc(alias = "SDL_BindGPUComputeStorageBuffers")]
    pub fn bind_storage_buffers(&self, first_slot: u32, buffers: &[Ref<Buffer>]) {
        unsafe {
            SDL_BindGPUComputeStorageBuffers(
                self.handle.as_ptr(),
                first_slot,
                buffers.as_ptr().cast(),
                buffers.len() as _,
            );
        }
    }

    /// Dispatch compute work.
    ///
    /// `(x, y, z)` is the number of local workgroups to dispatch in each
    /// dimension. A compute pipeline must be bound first. Multiple dispatches
    /// writing the same resource region have no guaranteed write order; end the
    /// pass and begin another one when ordering is required.
    #[doc(alias = "SDL_DispatchGPUCompute")]
    pub fn dispatch(&self, (x, y, z): (u32, u32, u32)) {
        unsafe { SDL_DispatchGPUCompute(self.handle.as_ptr(), x, y, z) }
    }

    /// Dispatch compute work using parameters read from a buffer.
    ///
    /// `buffer` contains [`IndirectDispatchCommand`] parameters, and `offset`
    /// is the byte offset at which to read them. A compute pipeline must be
    /// bound first. Multiple dispatches writing the same resource region have
    /// no guaranteed write order; end the pass and begin another one when
    /// ordering is required.
    #[doc(alias = "SDL_DispatchGPUComputeIndirect")]
    pub fn dispatch_indirect(&self, buffer: Ref<Buffer>, offset: u32) {
        unsafe {
            SDL_DispatchGPUComputeIndirect(self.handle.as_ptr(), buffer.handle.as_ptr(), offset);
        }
    }
}
