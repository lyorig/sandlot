use std::marker::PhantomData;

use sdl3_sys::{gpu::*, render::*, surface::*, video::*};
use sdl3_ttf_sys::ttf::*;

use crate::ttf::Context;

macro_rules! resource_new_impl {
    ($sdl:ident, $owned:ident) => {
        paste::paste! {
            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>] {
                pub(crate) handle: std::ptr::NonNull<$sdl>,
            }

            impl [<$owned Handle>] {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    std::ptr::NonNull::new(handle).map(|handle| Self { handle })
                }

                pub(crate) fn as_ptr(&self) -> *mut $sdl {
                    self.handle.as_ptr()
                }
            }

            impl $owned {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> $crate::Result<Self> {
                    match std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] { handle },
                        }),
                        None => Err($crate::error::Error::current()),
                    }
                }
            }

            impl std::ops::Deref for $owned {
                type Target = [<$owned Handle>];
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl std::ops::DerefMut for $owned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl $crate::resource::Handle for [<$owned Handle>] {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl $crate::resource::Resource for $owned {
                type Handle = [<$owned Handle>];

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }
        }
    };
}

macro_rules! resource_new_no_drop {
    ($(#[$meta:meta])* $sdl:ty, $owned:ident) => {
        paste::paste! {
            $(#[$meta])*
            ///
            /// BEWARE: This struct has no automatic destructor, and must be manually dropped or otherwise consumed!
            #[must_use = "This struct has to be manually dropped via an associated `drop()` method."]
            #[doc(alias = "" $sdl "")]
            pub struct $owned {
                pub(crate) inner: [<$owned Handle>],
            }

            resource_new_impl!($sdl, $owned);
        }
    };
}

/// Define a resource and implement shared traits and member functions.
macro_rules! resource_new {
    ($(#[$meta:meta])* $sdl:ident, $owned:ident, $dtor:ident) => {
        paste::paste! {
            $(#[$meta])*
            #[doc(alias = "" $sdl "")]
            pub struct $owned {
                pub(crate) inner: [<$owned Handle>],
            }
        }

        paste::paste! {
            resource_new_impl!($sdl, $owned);

            impl Drop for $owned {
                #[doc(alias = "" $sdl "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    };
}

macro_rules! resource_new_tied {
    ($(#[$meta:meta])* $sdl:ident, $owned:ident, $dtor:ident, $tied:ident) => {
        paste::paste! {
            $(#[$meta])*
            #[doc(alias = "" $sdl "")]
            pub struct $owned<'a> {
                pub(crate) inner: [<$owned Handle>],
                pub(crate) marker: PhantomData<&'a $tied>,
            }

            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>] {
                pub(crate) handle: ::std::ptr::NonNull<$sdl>,
            }

            impl [<$owned Handle>] {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    std::ptr::NonNull::new(handle).map(|handle| Self { handle })
                }

                /// Convenience method to directly access the underlying pointer.
                pub(crate) fn as_ptr(&self) -> *mut $sdl {
                    self.handle.as_ptr()
                }
            }

            impl<'a> $owned<'a> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> $crate::Result<Self> {
                    match std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] { handle },
                            marker: PhantomData,
                        }),
                        None => Err($crate::error::Error::current()),
                    }
                }
            }

            impl std::ops::Deref for $owned<'_> {
                type Target = [<$owned Handle>];
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl std::ops::DerefMut for $owned<'_> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl $crate::resource::Handle for [<$owned Handle>] {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl $crate::resource::Resource for $owned<'_> {
                type Handle = [<$owned Handle>];

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }

            impl Drop for $owned<'_> {
                #[doc(alias = "" $dtor "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    };
}

resource_new_no_drop!(
    /// Represents a GPU buffer.
    /// Used for vertices, indices, indirect draw commands, and general compute data.
    SDL_GPUBuffer, Buffer
);

resource_new_no_drop!(
    /// An opaque handle representing a command buffer.
    /// Most state is managed via command buffers, and is local to each one.
    SDL_GPUCommandBuffer, CommandBuffer
);

resource_new_no_drop!(
    /// An opaque handle representing a fence.
    SDL_GPUFence, Fence
);

resource_new!(
    /// An opaque handle representing a compute pass.
    /// Transient and invalid once the pass ends.
    SDL_GPUComputePass, ComputePass, SDL_EndGPUComputePass
);

resource_new!(
    /// An opaque handle representing the SDL_GPU context.
    SDL_GPUDevice, Device, SDL_DestroyGPUDevice
);

resource_new_no_drop!(
    /// An opaque handle representing a compute pipeline.
    /// Used during compute passes.
    SDL_GPUComputePipeline, ComputePipeline
);

resource_new!(
    /// An opaque handle representing a copy pass.
    /// Transient; invalid once the pass ends.
    SDL_GPUCopyPass, CopyPass, SDL_EndGPUCopyPass
);

resource_new_no_drop!(
    /// An opaque handle representing a graphics pipeline.
    /// Used during render passes.
    SDL_GPUGraphicsPipeline, GraphicsPipeline
);

resource_new!(
    /// An opaque handle representing a render pass.
    /// Transient; invalid once the pass ends.
    SDL_GPURenderPass, RenderPass, SDL_EndGPURenderPass
);

resource_new!(
    /// A custom GPU render state.
    SDL_GPURenderState, RenderState, SDL_DestroyGPURenderState
);

resource_new_no_drop!(
    /// An opaque handle representing a sampler.
    SDL_GPUSampler, Sampler
);

resource_new_no_drop!(
    /// An opaque handle representing a compiled shader object.
    SDL_GPUShader, Shader
);

resource_new_no_drop!(
    /// An opaque handle representing a texture.
    SDL_GPUTexture, GpuTexture
);

resource_new_no_drop!(
    /// An opaque handle representing a transfer buffer.
    /// Used for transferring data to and from the GPU.
    SDL_GPUTransferBuffer, TransferBuffer
);

resource_new!(
    /// Represents rendering state.
    SDL_Renderer, Renderer, SDL_DestroyRenderer
);

resource_new!(
    /// A collection of pixels used in software blitting.
    ///
    /// # Remarks
    ///
    /// Pixels are arranged in memory in rows, with the top row first.
    /// Each row occupies an amount of memory given by the pitch (sometimes known as the row stride in non-SDL APIs).
    ///
    /// Within each row, pixels are arranged from left to right until the width is reached.
    /// Each pixel occupies a number of bits appropriate for its format, with most formats representing
    /// each pixel as one or more whole bytes (in some indexed formats, instead multiple pixels are packed into each byte),
    /// and a byte order given by the format. After encoding all pixels, any remaining bytes to reach the pitch are used
    /// as padding to reach a desired alignment, and have undefined contents.
    ///
    /// When a surface holds YUV format data, the planes are assumed to be contiguous without padding between them,
    /// e.g. a 32x32 surface in NV12 format with a pitch of 32 would consist of 32x32 bytes of Y plane followed by 32x16 bytes of UV plane.
    ///
    /// When a surface holds MJPG format data, pixels points at the compressed JPEG image and pitch is the length of that data.
    SDL_Surface, Surface, SDL_DestroySurface
);

resource_new!(
    /// A GPU texture residing in VRAM.
    SDL_Texture, Texture, SDL_DestroyTexture
);

resource_new!(
    /// Represents a window on the screen.
    SDL_Window, Window, SDL_DestroyWindow
);

resource_new!(
    /// A text engine that draws text objects with the SDL GPU API.
    TTF_TextEngine, GpuEngine, TTF_DestroyGPUTextEngine
);

resource_new!(
    /// A text engine that draws text objects to a [`Surface`](crate::Surface).
    TTF_TextEngine, SurfaceEngine, TTF_DestroySurfaceTextEngine
);

resource_new!(
    /// A text engine that draws text objects with an SDL 2D renderer.
    TTF_TextEngine,
    RendererEngine,
    TTF_DestroyRendererTextEngine
);

resource_new_tied!(
    /// A font with which you can render text in various ways.
    TTF_Font, Font, TTF_CloseFont, Context
);

resource_new!(
    /// Represents a text object that can be rendered with a [`Font`](crate::ttf::Font).
    TTF_Text, Text, TTF_DestroyText
);
