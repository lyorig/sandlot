//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_QueryGPUFence
//! - [x] SDL_ReleaseGPUFence

use sdl3_sys::gpu::*;

use crate::{resource::Ref, resource::resource_new};

use super::device::Device;

resource_new! {
    /// An opaque handle representing a fence.
    pub struct Fence<'ctx, 'vid, 'dev> : SDL_GPUFence {
        marker: PhantomData<(Ref<'dev, Device<'ctx, 'vid>>)>,
    }
}

impl<'ctx, 'vid, 'dev> Fence<'ctx, 'vid, 'dev> {
    /// Release a fence obtained from command-buffer submission.
    ///
    /// `device` is the GPU device that owns the fence. This method consumes the
    /// fence; it must not be referenced after this call.
    #[doc(alias = "SDL_ReleaseGPUFence")]
    pub fn drop(self, device: Ref<'dev, Device<'ctx, 'vid>>) {
        unsafe { SDL_ReleaseGPUFence(device.as_raw(), self.as_raw()) }
    }
}

impl<'ctx, 'vid, 'dev> FenceHandle<'ctx, 'vid, 'dev> {
    /// Check whether this fence has been signaled by the GPU.
    ///
    /// `device` is the GPU device associated with the fence. Returns `true` if
    /// the submitted work has signaled the fence, or `false` otherwise.
    #[doc(alias = "SDL_QueryGPUFence")]
    pub fn is_signaled(self, device: Ref<'dev, Device<'ctx, 'vid>>) -> bool {
        unsafe { SDL_QueryGPUFence(device.as_raw(), self.as_raw()) }
    }
}
