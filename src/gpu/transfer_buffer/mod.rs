//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUTransferBuffer
//! - [x] SDL_MapGPUTransferBuffer
//! - [x] SDL_ReleaseGPUTransferBuffer
//! - [x] SDL_UnmapGPUTransferBuffer

use std::{marker::PhantomData, ptr::NonNull};

use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result, error::Error, gpu::Cycle, properties::Properties, resource::Ref,
    util::impl_enum_transmute, util::mod_reexport, util::resource_new_no_drop,
};

use super::device::Device;

mod_reexport!(builder);

/// A location in a transfer buffer used for GPU data transfers.
///
/// The transfer buffer is borrowed for `'tb`, and `offset` is the starting byte
/// of the data in that buffer.
#[doc(alias = "SDL_GPUTransferBufferLocation")]
#[derive(Clone, Copy)]
pub struct TransferBufferLocation<'tb>(
    pub(crate) SDL_GPUTransferBufferLocation,
    PhantomData<&'tb TransferBuffer>,
);

impl<'tb> TransferBufferLocation<'tb> {
    /// Refer to `tb` at `offset` bytes from the beginning of the buffer.
    pub fn new(tb: Ref<'tb, TransferBuffer>, offset: u32) -> Self {
        let transfer_buffer = tb.handle.as_ptr();
        let inner = SDL_GPUTransferBufferLocation {
            transfer_buffer,
            offset,
        };

        Self(inner, PhantomData)
    }

    /// Like [`Self::new`] with an offset of zero.
    pub fn whole(tb: Ref<'tb, TransferBuffer>) -> Self {
        Self::new(tb, 0)
    }
}

/// The direction in which a transfer buffer is intended to be used.
///
/// Mapping and copying from an upload buffer, or copying to a download buffer,
/// is invalid.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUTransferBufferUsage")]
pub enum TransferBufferUsage {
    /// The buffer is used as the source of uploads to GPU resources.
    Upload = SDL_GPUTransferBufferUsage::UPLOAD.0,
    /// The buffer is used as the destination of downloads from GPU resources.
    Download = SDL_GPUTransferBufferUsage::DOWNLOAD.0,
}

impl_enum_transmute!(SDL_GPUTransferBufferUsage, TransferBufferUsage);

/// Parameters for creating a transfer buffer.
///
/// The wrapper leaves SDL's extension-property ID set to zero.
#[doc(alias = "SDL_GPUTransferBufferCreateInfo")]
#[derive(Clone, Copy)]
pub struct TransferBufferCreateInfo(SDL_GPUTransferBufferCreateInfo);
impl TransferBufferCreateInfo {
    /// Describe a transfer buffer with the given direction and size.
    ///
    /// `usage` selects whether the buffer uploads to or downloads from GPU
    /// resources, and `size` is the buffer size in bytes.
    pub const fn new(usage: TransferBufferUsage, size: u32) -> Self {
        Self(SDL_GPUTransferBufferCreateInfo {
            usage: SDL_GPUTransferBufferUsage::new(usage as _),
            size,
            props: SDL_PropertiesID::new(0),
        })
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a transfer buffer.
    /// Used for transferring data to and from the GPU.
    SDL_GPUTransferBuffer, TransferBuffer
);
impl TransferBuffer {
    /// Build a [`TransferBuffer`] with additional parameters not available in [`TransferBufferCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> TransferBufferBuilder<'_> {
        TransferBufferBuilder::new(props)
    }

    /// Create a transfer buffer for uploading to or downloading from GPU resources.
    ///
    /// This does not map or write the buffer. `device` is the GPU device that
    /// owns it, and `create_info` describes its direction and size. Download
    /// buffers can be expensive to create, so reuse them when downloading data
    /// regularly.
    ///
    /// # Important
    /// [`TransferBuffer::new_with`] should be preferred, as it is safer and more
    /// ergonomic, unless you have a particular reason to use these methods
    /// directly.
    ///
    /// Returns [`Err`] if the transfer buffer cannot be created.
    #[doc(alias = "SDL_CreateGPUTransferBuffer")]
    pub fn new(device: Ref<Device>, create_info: &TransferBufferCreateInfo) -> Result<Self> {
        let handle = unsafe {
            SDL_CreateGPUTransferBuffer(device.handle.as_ptr(), &raw const create_info.0)
        };
        Self::from_ptr(handle)
    }

    /// Creates a new [`TransferBuffer`], maps it, calls `write` with the mapped data, then unmaps.
    /// Basically calls the following functions/methods, in order:
    /// 1. [`TransferBuffer::new`]
    /// 2. [`TransferBufferHandle::map`]
    /// 3. `write`
    /// 4. [`TransferBufferHandle::unmap`]
    ///
    /// Any errors are propagated.
    /// The buffer must still be dropped manually.
    ///
    /// This should be your preferred way to use this struct.
    /// Although the aforementioned methods this constructor uses are public,
    /// they are mainly provided to cover the API surface, and enable advanced uses.
    pub fn new_with<F: FnOnce(&mut [u8])>(
        device: Ref<Device>,
        create_info: &TransferBufferCreateInfo,
        cycle: Cycle,
        write: F,
    ) -> Result<Self> {
        let tb = Self::new(device, create_info)?;
        let ptr = tb.map(device, cycle)?;
        let slice = unsafe {
            std::slice::from_raw_parts_mut(ptr.as_ptr().cast::<u8>(), create_info.0.size as _)
        };

        write(slice);

        tb.unmap(device);

        Ok(tb)
    }

    /// Release a transfer buffer as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the transfer buffer. This method
    /// consumes the buffer; it must not be referenced after this call. Unlike
    /// ordinary RAII resources, a transfer buffer created with this module has
    /// no automatic destructor, so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUTransferBuffer")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUTransferBuffer(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}

impl TransferBufferHandle {
    /// Map this transfer buffer into application address space.
    ///
    /// `device` is the GPU device that owns the buffer. `cycle` controls
    /// whether SDL cycles the buffer if it is already bound. The returned
    /// pointer refers to memory owned by the graphics driver; it must not be
    /// freed by the caller.
    ///
    /// Returns [`Err`] if the buffer cannot be mapped. Call [`Self::unmap`]
    /// before encoding upload commands that use the buffer.
    #[doc(alias = "SDL_MapGPUTransferBuffer")]
    pub fn map(&self, device: Ref<Device>, cycle: Cycle) -> Result<NonNull<u8>> {
        let ptr = unsafe {
            SDL_MapGPUTransferBuffer(device.handle.as_ptr(), self.handle.as_ptr(), cycle.into())
        };
        NonNull::new(ptr.cast()).ok_or_else(Error::current)
    }

    /// Unmap a previously mapped transfer buffer.
    ///
    /// `device` is the GPU device that owns the buffer. The mapped memory must
    /// be unmapped before encoding upload commands that use it.
    #[doc(alias = "SDL_UnmapGPUTransferBuffer")]
    pub fn unmap(&self, device: Ref<Device>) {
        unsafe { SDL_UnmapGPUTransferBuffer(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}
