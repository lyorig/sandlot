//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_QueryGPUFence
//! - [x] SDL_ReleaseGPUFence

use sdl3_sys::gpu::*;

use crate::{resource::Ref, resource_new_no_drop};

use super::device::Device;

resource_new_no_drop!(
    /// An opaque handle representing a fence.
    SDL_GPUFence, Fence
);
impl Fence {
    /// Release a fence obtained from command-buffer submission.
    ///
    /// `device` is the GPU device that owns the fence. This method consumes the
    /// fence; it must not be referenced after this call.
    #[doc(alias = "SDL_ReleaseGPUFence")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUFence(device.handle.as_ptr(), self.handle.as_ptr()) }
    }
}

impl FenceHandle {
    /// Check whether this fence has been signaled by the GPU.
    ///
    /// `device` is the GPU device associated with the fence. Returns `true` if
    /// the submitted work has signaled the fence, or `false` otherwise.
    #[doc(alias = "SDL_QueryGPUFence")]
    pub fn is_signaled(&self, device: Ref<Device>) -> bool {
        unsafe { SDL_QueryGPUFence(device.handle.as_ptr(), self.handle.as_ptr()) }
    }
}
