//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUBuffer
//! - [x] SDL_DownloadFromGPUBuffer
//! - [x] SDL_ReleaseGPUBuffer
//! - [x] SDL_SetGPUBufferName
//! - [x] SDL_UploadToGPUBuffer

use std::{ffi::CStr, marker::PhantomData};

use bitflags::bitflags;
use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result, gpu::Cycle, mod_reexport, properties::Properties, resource::Ref, resource_new_no_drop, util::impl_enum_transmute,
};

use super::{copy_pass::CopyPass, device::Device, transfer_buffer::TransferBufferLocation};

mod_reexport!(builder);

bitflags! {
    /// Specifies how a buffer is intended to be used.
    ///
    /// At least one usage flag is required. Flags can be combined, although
    /// some combinations are invalid, such as [`Self::VERTEX`] with
    /// [`Self::INDEX`]. Multiple read usages may result in more conservative
    /// memory barriers. Unlike textures, read and write storage usages can be
    /// combined for simultaneous read-write access.
    ///
    /// Storage buffers must follow `std430` layout conventions. In particular,
    /// `vec3` and `vec4` fields must be aligned to 16-byte boundaries.
    #[derive(Clone, Copy)]
    #[doc(alias = "SDL_GPUBufferUsageFlags")]
    pub struct BufferUsageFlags: u32 {
        /// The buffer is a vertex buffer.
        const VERTEX = SDL_GPUBufferUsageFlags::VERTEX.0;
        /// The buffer is an index buffer.
        const INDEX = SDL_GPUBufferUsageFlags::INDEX.0;
        /// The buffer is an indirect buffer.
        const INDIRECT = SDL_GPUBufferUsageFlags::INDIRECT.0;
        /// The buffer supports storage reads in graphics stages.
        const GRAPHICS_STORAGE_READ = SDL_GPUBufferUsageFlags::GRAPHICS_STORAGE_READ.0;
        /// The buffer supports storage reads in the compute stage.
        const COMPUTE_STORAGE_READ = SDL_GPUBufferUsageFlags::COMPUTE_STORAGE_READ.0;
        /// The buffer supports storage writes in the compute stage.
        const COMPUTE_STORAGE_WRITE = SDL_GPUBufferUsageFlags::COMPUTE_STORAGE_WRITE.0;
    }
}

impl_enum_transmute!(SDL_GPUBufferUsageFlags, BufferUsageFlags);

/// Parameters for creating a GPU buffer.
///
/// The buffer's contents are undefined until data is written to it. Usage flags
/// can be combined, but certain combinations are invalid, such as vertex and
/// index usage together. Storage buffers must follow `std430` layout conventions;
/// `vec3` and `vec4` fields must be aligned to 16-byte boundaries.
///
/// The wrapper leaves SDL's extension-property ID set to zero.
#[doc(alias = "SDL_GPUBufferCreateInfo")]
#[derive(Clone, Copy)]
pub struct BufferCreateInfo(SDL_GPUBufferCreateInfo);
impl BufferCreateInfo {
    /// Describe a buffer with the given usages and size.
    ///
    /// `usage` specifies how the buffer will be used, and `size` is its size in
    /// bytes. At least one usage flag must be set.
    pub const fn new(usage: BufferUsageFlags, size: u32) -> Self {
        let usage = SDL_GPUBufferUsageFlags::new(usage.bits());
        let inner = SDL_GPUBufferCreateInfo {
            usage,
            size,
            props: SDL_PropertiesID::new(0),
        };
        Self(inner)
    }
}

/// A region of a GPU buffer used for data transfers.
///
/// The region borrows `buffer` for `'b`. `offset` and `size` are measured in
/// bytes, and the region starts at `offset` within the buffer.
#[doc(alias = "SDL_GPUBufferRegion")]
#[derive(Clone, Copy)]
pub struct BufferRegion<'b>(SDL_GPUBufferRegion, PhantomData<Ref<'b, Buffer>>);
impl<'b> BufferRegion<'b> {
    /// Describe a region of `buffer` beginning at `offset` and extending for
    /// `size` bytes.
    pub fn new(buffer: Ref<'b, Buffer>, offset: u32, size: u32) -> Self {
        let buffer = buffer.handle.as_ptr();
        let inner = SDL_GPUBufferRegion {
            buffer,
            offset,
            size,
        };
        Self(inner, PhantomData)
    }

    /// Same as [`Self::new`], but with an offset of zero.
    pub fn whole(buffer: Ref<'b, Buffer>, size: u32) -> Self {
        Self::new(buffer, 0, size)
    }
}

/// Parameters for binding a GPU buffer.
///
/// The buffer must have been created with [`BufferUsageFlags::VERTEX`] when
/// used as a vertex buffer or [`BufferUsageFlags::INDEX`] when used as an index
/// buffer. The buffer is borrowed for `'b`; `offset` is the starting byte of the
/// data to bind.
#[doc(alias = "SDL_GPUBufferBinding")]
#[derive(Clone, Copy)]
pub struct BufferBinding<'b>(
    pub(crate) SDL_GPUBufferBinding,
    PhantomData<Ref<'b, Buffer>>,
);

impl<'b> BufferBinding<'b> {
    /// Bind `buffer` starting at `offset` bytes into the buffer.
    pub fn new(buffer: Ref<'b, Buffer>, offset: u32) -> Self {
        Self(
            SDL_GPUBufferBinding {
                buffer: buffer.handle.as_ptr(),
                offset,
            },
            PhantomData,
        )
    }
}

/// A location in a GPU buffer used when copying between buffers.
///
/// The location borrows `buffer` for `'b`, and `offset` is measured in bytes
/// from the beginning of that buffer.
#[doc(alias = "SDL_GPUBufferLocation")]
#[derive(Clone, Copy)]
pub struct BufferLocation<'b>(
    pub(crate) SDL_GPUBufferLocation,
    PhantomData<Ref<'b, Buffer>>,
);

impl<'b> BufferLocation<'b> {
    /// Refer to `buffer` at `offset` bytes from its beginning.
    pub fn new(buffer: Ref<'b, Buffer>, offset: u32) -> Self {
        Self(
            SDL_GPUBufferLocation {
                buffer: buffer.handle.as_ptr(),
                offset,
            },
            PhantomData,
        )
    }

    /// Same as [`Self::new`], with an offset of zero.
    pub fn at_start(buffer: Ref<'b, Buffer>) -> Self {
        Self::new(buffer, 0)
    }
}

/// Parameters for binding a storage buffer for read-write access in a compute pass.
///
/// `buffer` must have been created with
/// [`BufferUsageFlags::COMPUTE_STORAGE_WRITE`]. The buffer is borrowed for `'b`.
/// `cycle` controls whether SDL cycles the buffer when it is already bound.
#[doc(alias = "SDL_GPUStorageBufferReadWriteBinding")]
#[derive(Clone, Copy)]
pub struct StorageBufferReadWriteBinding<'b>(
    SDL_GPUStorageBufferReadWriteBinding,
    PhantomData<Ref<'b, Buffer>>,
);

impl<'b> StorageBufferReadWriteBinding<'b> {
    /// Bind `buffer` for read-write access, using `cycle` when it is already bound.
    pub fn new(buffer: Ref<'b, Buffer>, cycle: Cycle) -> Self {
        Self(
            SDL_GPUStorageBufferReadWriteBinding {
                buffer: buffer.handle.as_ptr(),
                cycle: cycle.into(),
                ..Default::default()
            },
            PhantomData,
        )
    }
}

resource_new_no_drop!(
    /// Represents a GPU buffer.
    /// Used for vertices, indices, indirect draw commands, and general compute data.
    SDL_GPUBuffer, Buffer
);

impl Buffer {
    /// Build a [`Buffer`] with additional parameters not available in [`BufferCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> BufferBuilder<'_> {
        BufferBuilder::new(props)
    }

    /// Create a buffer for use in graphics or compute workflows.
    ///
    /// `device` is the GPU device that owns the buffer, and `create_info`
    /// describes its usage flags and size. The buffer's contents are undefined
    /// until data is written to it.
    ///
    /// Returns [`Err`] if the buffer cannot be created or its usage combination
    /// is invalid.
    #[doc(alias = "SDL_CreateGPUBuffer")]
    pub fn new(device: Ref<Device>, create_info: &BufferCreateInfo) -> Result<Self> {
        let handle =
            unsafe { SDL_CreateGPUBuffer(device.handle.as_ptr(), &raw const create_info.0) };

        Self::from_ptr(handle)
    }

    /// Release a buffer as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the buffer. This method consumes the
    /// buffer; it must not be referenced after this call. Unlike ordinary RAII
    /// resources, a buffer created with this module has no automatic destructor,
    /// so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUBuffer")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUBuffer(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}

impl BufferHandle {
    /// Upload data from a transfer buffer to this buffer on the GPU timeline.
    ///
    /// * `copy_pass` is the copy pass that records the upload.
    /// * `src` identifies the source transfer buffer and offset.
    /// * `dst` identifies the destination buffer region and size.
    /// * `cycle` controls whether SDL cycles the destination buffer if it is
    ///   already bound; otherwise the existing data is overwritten.
    ///
    /// Subsequent commands can assume that the upload has finished.
    #[doc(alias = "SDL_UploadToGPUBuffer")]
    pub fn upload(
        &self,
        copy_pass: Ref<CopyPass>,
        src: &TransferBufferLocation,
        dst: &BufferRegion,
        cycle: Cycle,
    ) {
        unsafe {
            SDL_UploadToGPUBuffer(
                copy_pass.handle.as_ptr(),
                &raw const src.0,
                &raw const dst.0,
                cycle.into(),
            );
        }
    }

    /// Copy data from this buffer to a transfer buffer on the GPU timeline.
    ///
    /// * `copy_pass` is the copy pass that records the download.
    /// * `src` identifies the source region of this buffer.
    /// * `dst` identifies the destination transfer buffer and offset.
    ///
    /// The data is not guaranteed to have been copied until the command-buffer
    /// fence is signaled.
    #[doc(alias = "SDL_DownloadFromGPUBuffer")]
    pub fn download(
        &self,
        copy_pass: Ref<CopyPass>,
        src: &BufferRegion,
        dst: &TransferBufferLocation,
    ) {
        unsafe {
            SDL_DownloadFromGPUBuffer(
                copy_pass.handle.as_ptr(),
                &raw const src.0,
                &raw const dst.0,
            );
        };
    }

    /// Attach a UTF-8 label to this buffer.
    ///
    /// `device` is the GPU device that owns the buffer, and `name` is the label
    /// used by debugging tools. To name a buffer at creation time, prefer the
    /// [`BufferBuilder`] name property when constructing it.
    #[doc(alias = "SDL_SetGPUBufferName")]
    pub fn set_name(&self, device: Ref<Device>, name: &CStr) {
        unsafe {
            SDL_SetGPUBufferName(device.handle.as_ptr(), self.handle.as_ptr(), name.as_ptr());
        };
    }
}
