//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_BeginGPUCopyPass
//! - [x] SDL_CopyGPUBufferToBuffer
//! - [x] SDL_CopyGPUTextureToTexture
//! - [x] SDL_EndGPUCopyPass

use sdl3_sys::gpu::*;

use crate::{
    Result,
    gpu::Cycle,
    resource::{Ref, Resource},
};

use super::{buffer::BufferLocation, command_buffer::CommandBuffer, texture::TextureLocation};

pub use crate::generated::{CopyPass, CopyPassHandle};
impl CopyPass {
    /// Begin a copy pass on a command buffer.
    ///
    /// `cmdbuf` is the command buffer that records the pass. All operations
    /// that copy to or from buffers or textures must occur inside a copy pass.
    /// Another copy, render, or compute pass cannot begin until this pass ends.
    ///
    /// Returns [`Err`] if SDL cannot begin the pass.
    #[doc(alias = "SDL_BeginGPUCopyPass")]
    pub fn new(cmdbuf: Ref<CommandBuffer>) -> Result<Self> {
        let handle = unsafe { SDL_BeginGPUCopyPass(cmdbuf.handle.as_ptr()) };
        Self::from_ptr(handle)
    }

    /// Convenience function that creates a [`CopyPass`], does some work on it,
    /// then ends (drops) it.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`CopyPass::new`]
    /// - `op`
    pub fn run<F: FnOnce(Ref<Self>) -> Result<()>>(
        cmdbuf: Ref<CommandBuffer>,
        op: F,
    ) -> Result<()> {
        let pass = Self::new(cmdbuf)?;
        op(pass.as_ref())
    }
}

impl CopyPassHandle {
    /// Copy a region from one texture to another on the GPU timeline.
    ///
    /// * `source` identifies the source texture and location.
    /// * `destination` identifies the destination texture and location.
    /// * `(w, h, d)` is the width, height, and depth of the region to copy.
    /// * `cycle` controls whether SDL cycles the destination texture if it is
    ///   already bound; otherwise existing data is overwritten.
    ///
    /// Subsequent commands can assume that the copy has finished. Direct copies
    /// between depth and color textures are not supported; copy through a buffer
    /// instead.
    #[doc(alias = "SDL_CopyGPUTextureToTexture")]
    pub fn copy_texture_to_texture(
        &self,
        source: &TextureLocation,
        destination: &TextureLocation,
        (w, h, d): (u32, u32, u32),
        cycle: Cycle,
    ) {
        unsafe {
            SDL_CopyGPUTextureToTexture(
                self.handle.as_ptr(),
                &raw const source.0,
                &raw const destination.0,
                w,
                h,
                d,
                cycle.into(),
            );
        }
    }

    /// Copy a region from one buffer to another on the GPU timeline.
    ///
    /// * `source` identifies the source buffer and byte offset.
    /// * `destination` identifies the destination buffer and byte offset.
    /// * `size` is the number of bytes to copy.
    /// * `cycle` controls whether SDL cycles the destination buffer if it is
    ///   already bound; otherwise existing data is overwritten.
    ///
    /// Subsequent commands can assume that the copy has finished.
    #[doc(alias = "SDL_CopyGPUBufferToBuffer")]
    pub fn copy_buffer_to_buffer(
        &self,
        source: &BufferLocation,
        destination: &BufferLocation,
        size: u32,
        cycle: Cycle,
    ) {
        unsafe {
            SDL_CopyGPUBufferToBuffer(
                self.handle.as_ptr(),
                &raw const source.0,
                &raw const destination.0,
                size,
                cycle.into(),
            );
        }
    }
}
