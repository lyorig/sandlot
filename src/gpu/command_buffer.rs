//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_AcquireGPUCommandBuffer
//! - [x] SDL_AcquireGPUSwapchainTexture
//! - [x] SDL_SubmitGPUCommandBuffer
//! - [x] SDL_SubmitGPUCommandBufferAndAcquireFence
//! - [x] SDL_WaitAndAcquireGPUSwapchainTexture
//! - [x] SDL_CancelGPUCommandBuffer
//! - [x] SDL_BlitGPUTexture
//! - [x] SDL_GenerateMipmapsForGPUTexture
//! - [x] SDL_InsertGPUDebugLabel
//! - [x] SDL_PopGPUDebugGroup
//! - [x] SDL_PushGPUDebugGroup
//! - [x] SDL_PushGPUComputeUniformData
//! - [x] SDL_PushGPUFragmentUniformData
//! - [x] SDL_PushGPUVertexUniformData

use std::{ffi::CStr, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use sdl3_sys::{gpu::*, surface::SDL_FlipMode};

use crate::{
    Result,
    color::RgbaF32,
    gpu::Cycle,
    impl_enum_transmute,
    resource::{Ref, Resource},
    resource_new_no_drop,
    util::{opt2ptr_mut, to_result},
    window::Window,
};

use super::{
    device::Device,
    fence::Fence,
    render_pass::LoadOp,
    sampler::Filter,
    texture::{BlitRegion, Texture, TextureHandle},
};

/// Converts a raw swapchain texture pointer into a reference.
/// A null pointer (e.g. too many frames in flight) yields `None`.
fn swapchain_texture<'a>(ptr: *mut SDL_GPUTexture) -> Option<Ref<'a, Texture>> {
    let handle = NonNull::new(ptr)?;
    let inner = TextureHandle { handle };
    Some(unsafe { Ref::from_handle(inner) })
}

/// The flip applied to a source region during a blit.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_FlipMode")]
pub enum FlipMode {
    /// Do not flip the source region.
    None = SDL_FlipMode::NONE.0,
    /// Flip the source region horizontally.
    Horizontal = SDL_FlipMode::HORIZONTAL.0,
    /// Flip the source region vertically.
    Vertical = SDL_FlipMode::VERTICAL.0,
    /// Flip the source region horizontally and vertically.
    HorizontalAndVertical = SDL_FlipMode::HORIZONTAL_AND_VERTICAL.0,
}

impl_enum_transmute!(SDL_FlipMode, FlipMode);

/// Parameters for a texture blit command.
///
/// The source and destination textures are borrowed for `'s` and `'d`,
/// respectively. `load_op` controls the destination before the blit;
/// `clear_color` is used only when it clears. `flip_mode` transforms the source,
/// `filter` selects the blit filter, and `cycle` controls whether SDL cycles an
/// already-bound destination texture.
#[doc(alias = "SDL_GPUBlitInfo")]
#[derive(Clone, Copy)]
pub struct BlitInfo<'s, 'd>(
    SDL_GPUBlitInfo,
    PhantomData<Ref<'s, Texture>>,
    PhantomData<Ref<'d, Texture>>,
);

impl<'s, 'd> BlitInfo<'s, 'd> {
    /// Describe a blit from `source` to `destination` with the given load,
    /// clear, flip, filter, and cycling behavior.
    pub fn new(
        source: BlitRegion<'s>,
        destination: BlitRegion<'d>,
        load_op: LoadOp,
        clear_color: RgbaF32,
        flip_mode: FlipMode,
        filter: Filter,
        cycle: Cycle,
    ) -> Self {
        Self(
            SDL_GPUBlitInfo {
                source: source.0,
                destination: destination.0,
                load_op: SDL_GPULoadOp::new(load_op as _),
                clear_color: clear_color.into(),
                flip_mode: SDL_FlipMode::new(flip_mode as _),
                filter: SDL_GPUFilter::new(filter as _),
                cycle: cycle.into(),
                ..Default::default()
            },
            PhantomData,
            PhantomData,
        )
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a command buffer.
    /// Most state is managed via command buffers, and is local to each one.
    SDL_GPUCommandBuffer, CommandBuffer
);
impl CommandBuffer {
    /// Acquire a command buffer from a GPU device.
    ///
    /// `device` is the GPU device from which to acquire the buffer. The command
    /// buffer is managed by SDL and must be submitted or cancelled; it must not
    /// be freed directly.
    ///
    /// Returns [`Err`] if a command buffer cannot be acquired.
    #[doc(alias = "SDL_AcquireGPUCommandBuffer")]
    pub fn new(device: Ref<Device>) -> Result<Self> {
        let handle = unsafe { SDL_AcquireGPUCommandBuffer(device.handle.as_ptr()) };
        Self::from_ptr(handle)
    }

    /// Creates a new [`CommandBuffer`], performs some operations on it, then submits it.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`CommandBuffer::new`]
    /// - `op`
    /// - [`CommandBuffer::submit`]
    pub fn run<F: FnOnce(Ref<Self>) -> Result<()>>(device: Ref<Device>, op: F) -> Result<()> {
        let cmdbuf = Self::new(device)?;
        op(cmdbuf.as_ref())?;
        cmdbuf.submit()
    }

    /// Creates a new [`CommandBuffer`], performs some operations on it, then submits it, returning a fence.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`CommandBuffer::new`]
    /// - `op`
    /// - [`CommandBuffer::submit_fence`]
    pub fn run_fence<F: FnOnce(Ref<Self>) -> Result<()>>(
        device: Ref<Device>,
        op: F,
    ) -> Result<Fence> {
        let cmdbuf = Self::new(device)?;
        op(cmdbuf.as_ref())?;
        cmdbuf.submit_fence()
    }

    /// Submit this command buffer for execution on the GPU.
    ///
    /// This consumes the command buffer; it must not be used after submission.
    /// Commands in this submission begin executing before commands in a later
    /// submission.
    ///
    /// Returns [`Err`] if submission fails.
    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<()> {
        to_result(unsafe { SDL_SubmitGPUCommandBuffer(self.handle.as_ptr()) })
    }

    /// Submit this command buffer and acquire a fence associated with it.
    ///
    /// This consumes the command buffer; it must not be used after submission.
    /// The returned fence must be released when it is no longer needed.
    ///
    /// Returns [`Err`] if submission or fence acquisition fails.
    #[doc(alias = "SDL_SubmitGPUCommandBufferAndAcquireFence")]
    pub fn submit_fence(self) -> Result<Fence> {
        let fence = unsafe { SDL_SubmitGPUCommandBufferAndAcquireFence(self.handle.as_ptr()) };
        Fence::from_ptr(fence)
    }

    /// Cancel this command buffer without executing its enqueued commands.
    ///
    /// This consumes the command buffer; it must not be used after cancellation.
    /// A command buffer cannot be cancelled after a swapchain texture has been
    /// acquired.
    ///
    /// Returns [`Err`] if cancellation fails.
    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(self) -> Result<()> {
        to_result(unsafe { SDL_CancelGPUCommandBuffer(self.handle.as_ptr()) })
    }
}

impl CommandBufferHandle {
    /// Acquire a texture for presentation from a claimed window.
    ///
    /// `wnd` is the claimed window. `tex_x` and `tex_y`, when present, receive
    /// the swapchain texture dimensions. The returned texture is managed by SDL,
    /// must not be freed, and is valid only for the command buffer that acquired
    /// it. It is write-only and cannot be sampled or read by another operation.
    ///
    /// Returns `Ok(None)` when too many frames are in flight and no texture is
    /// available; this is not an error and should be treated as a reason to wait.
    /// In the default VSync mode, this call may block while waiting for vblank.
    ///
    /// Returns [`Err`] on an SDL failure.
    #[doc(alias = "SDL_AcquireGPUSwapchainTexture")]
    pub fn acquire_swapchain_texture(
        &self,
        wnd: Ref<Window>,
        (tex_x, tex_y): (Option<&mut u32>, Option<&mut u32>),
    ) -> Result<Option<Ref<'_, Texture>>> {
        let mut tex = MaybeUninit::uninit();
        let res = unsafe {
            SDL_AcquireGPUSwapchainTexture(
                self.handle.as_ptr(),
                wnd.handle.as_ptr(),
                tex.as_mut_ptr(),
                opt2ptr_mut(tex_x),
                opt2ptr_mut(tex_y),
            )
        };

        to_result(res).map(|()| swapchain_texture(unsafe { tex.assume_init() }))
    }

    /// Block until a swapchain texture is available, then acquire it.
    ///
    /// `wnd` is the claimed window. `tex_x` and `tex_y`, when present, receive
    /// the swapchain texture dimensions. The returned texture is managed by SDL,
    /// must not be freed, and is valid only for the command buffer that acquired
    /// it. It is write-only and cannot be sampled or read by another operation.
    ///
    /// Returns `Ok(None)` when no texture is available, for example because
    /// the window is minimized; this is not an error. Acquiring a texture causes
    /// it to be presented when the command buffer is submitted, and the command
    /// buffer cannot be cancelled afterward.
    ///
    /// Returns [`Err`] on an SDL failure.
    #[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
    pub fn wait_for_swapchain_texture(
        &self,
        wnd: Ref<Window>,
        (tex_x, tex_y): (Option<&mut u32>, Option<&mut u32>),
    ) -> Result<Option<Ref<'_, Texture>>> {
        let mut tex = MaybeUninit::uninit();
        let res = unsafe {
            SDL_WaitAndAcquireGPUSwapchainTexture(
                self.handle.as_ptr(),
                wnd.handle.as_ptr(),
                tex.as_mut_ptr(),
                opt2ptr_mut(tex_x),
                opt2ptr_mut(tex_y),
            )
        };

        to_result(res).map(|()| swapchain_texture(unsafe { tex.assume_init() }))
    }

    /// Generate mipmaps for a texture with more than one mip level.
    ///
    /// This operation must be recorded outside any render, compute, or copy pass.
    #[doc(alias = "SDL_GenerateMipmapsForGPUTexture")]
    pub fn generate_mipmaps(&self, texture: Ref<Texture>) {
        unsafe { SDL_GenerateMipmapsForGPUTexture(self.handle.as_ptr(), texture.handle.as_ptr()) }
    }

    /// Insert a UTF-8 label into the command-buffer call stream for debugging.
    ///
    /// `text` is the label to insert. On Direct3D 12, this requires
    /// `WinPixEventRuntime.dll` to be available in `PATH` or beside the executable.
    #[doc(alias = "SDL_InsertGPUDebugLabel")]
    pub fn insert_debug_label(&self, text: &CStr) {
        unsafe { SDL_InsertGPUDebugLabel(self.handle.as_ptr(), text.as_ptr()) }
    }

    /// Begin a named debug group in the command-buffer call stream.
    ///
    /// `name` is a UTF-8 group name. Each call should be paired with
    /// [`Self::pop_debug_group`]. On Direct3D 12, this requires
    /// `WinPixEventRuntime.dll` to be available in `PATH` or beside the executable.
    /// On some backends, groups recorded inside a pass are scoped to that native
    /// pass, so they should be popped in the same pass.
    #[doc(alias = "SDL_PushGPUDebugGroup")]
    pub fn push_debug_group(&self, name: &CStr) {
        unsafe { SDL_PushGPUDebugGroup(self.handle.as_ptr(), name.as_ptr()) }
    }

    /// End the most recently pushed debug group.
    ///
    /// On Direct3D 12, this requires `WinPixEventRuntime.dll` to be available in
    /// `PATH` or beside the executable.
    #[doc(alias = "SDL_PopGPUDebugGroup")]
    pub fn pop_debug_group(&self) {
        unsafe { SDL_PopGPUDebugGroup(self.handle.as_ptr()) }
    }

    /// Push data to a vertex uniform slot on this command buffer.
    ///
    /// `slot_index` selects the uniform slot, and `data` contains the bytes to
    /// write. Subsequent draw calls in this command buffer use the data. The
    /// data must follow `std140` layout rules; `vec3` and `vec4` fields must be
    /// aligned to 16-byte boundaries.
    #[doc(alias = "SDL_PushGPUVertexUniformData")]
    pub fn push_vertex_uniform_data(&self, slot_index: u32, data: &[u8]) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.handle.as_ptr(),
                slot_index,
                data.as_ptr().cast(),
                data.len() as _,
            );
        }
    }

    /// Push data to a fragment uniform slot on this command buffer.
    ///
    /// `slot_index` selects the uniform slot, and `data` contains the bytes to
    /// write. Subsequent draw calls in this command buffer use the data. The
    /// data must follow `std140` layout rules; `vec3` and `vec4` fields must be
    /// aligned to 16-byte boundaries.
    #[doc(alias = "SDL_PushGPUFragmentUniformData")]
    pub fn push_fragment_uniform_data(&self, slot_index: u32, data: &[u8]) {
        unsafe {
            SDL_PushGPUFragmentUniformData(
                self.handle.as_ptr(),
                slot_index,
                data.as_ptr().cast(),
                data.len() as _,
            );
        }
    }

    /// Push data to a compute uniform slot on this command buffer.
    ///
    /// `slot_index` selects the uniform slot, and `data` contains the bytes to
    /// write. The data must follow `std140` layout rules; `vec3` and `vec4`
    /// fields must be aligned to 16-byte boundaries.
    #[doc(alias = "SDL_PushGPUComputeUniformData")]
    pub fn push_compute_uniform_data(&self, slot_index: u32, data: &[u8]) {
        unsafe {
            SDL_PushGPUComputeUniformData(
                self.handle.as_ptr(),
                slot_index,
                data.as_ptr().cast(),
                data.len() as _,
            );
        }
    }

    /// Blit a source texture region to a destination texture region.
    ///
    /// `info` contains the source, destination, load, clear, flip, filter, and
    /// cycling parameters. This operation must be recorded outside any pass.
    #[doc(alias = "SDL_BlitGPUTexture")]
    pub fn blit(&self, info: &BlitInfo) {
        unsafe { SDL_BlitGPUTexture(self.handle.as_ptr(), &raw const info.0) }
    }
}
