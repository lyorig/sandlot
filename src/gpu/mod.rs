//! Low-level interface to GPU APIs such as DirectX, Vulkan and Metal.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)).
//! Only free functions and functionality that fits no submodule belong here;
//! everything else is covered by the submodule checklists.
//! - [x] SDL_CalculateGPUTextureFormatSize
//! - [ ] SDL_GDKResumeGPU — GDK (Xbox) only, makes no sense to implement here
//! - [ ] SDL_GDKSuspendGPU — GDK (Xbox) only, makes no sense to implement here
//! - [x] SDL_GetGPUDriver
//! - [x] SDL_GetGPUTextureFormatFromPixelFormat
//! - [x] SDL_GetNumGPUDrivers
//! - [x] SDL_GetPixelFormatFromGPUTextureFormat
//! - [x] SDL_GPUSupportsProperties
//! - [x] SDL_GPUSupportsShaderFormats
//! - [x] SDL_GPUTextureFormatTexelBlockSize

mod_reexport!(buffer);
mod_reexport!(command_buffer);
mod_reexport!(compute_pass);
mod_reexport!(compute_pipeline);
mod_reexport!(copy_pass);
mod_reexport!(device);
mod_reexport!(enums);
mod_reexport!(fence);
mod_reexport!(graphics_pipeline);
mod_reexport!(pipeline_state);
mod_reexport!(render_pass);
mod_reexport!(render_state);
mod_reexport!(sampler);
mod_reexport!(shader);
mod_reexport!(texture);
mod_reexport!(transfer_buffer);

use bitflags::bitflags;
use sdl3_sys::{gpu::*, pixels::SDL_PixelFormat};

use crate::{
    impl_enum_transmute, mod_reexport, pixels::PixelFormat, properties::Properties, resource::Ref,
    util::c_ptr_to_str,
};

/// Non-bitmask variant of `SDL_GPUShaderFormat`.
/// A shader-code format accepted by a specific GPU backend.
#[repr(u32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUShaderFormat")]
pub enum ShaderFormat {
    /// SPIR-V shaders for Vulkan.
    SpirV = SDL_GPUShaderFormat::SPIRV.0,
    /// DXBC Shader Model 5.1 shaders for D3D12.
    Dxbc = SDL_GPUShaderFormat::DXBC.0,
    /// DXIL Shader Model 6.0 shaders for D3D12.
    Dxil = SDL_GPUShaderFormat::DXIL.0,
    /// MSL shaders for Metal.
    Msl = SDL_GPUShaderFormat::MSL.0,
    /// Precompiled Metal library shaders for Metal.
    Metallib = SDL_GPUShaderFormat::METALLIB.0,
}

impl ShaderFormat {
    /// Returns [`ShaderFormats`] with only this bit set.
    pub const fn as_mask(self) -> ShaderFormats {
        ShaderFormats::from_bits_retain(self as u32)
    }
}

bitflags! {
    /// Shader-code formats that an application can provide to SDL.
    #[derive(Clone, Copy)]
    #[doc(alias = "SDL_GPUShaderFormat")]
    pub struct ShaderFormats: u32 {
        /// SPIR-V shaders for Vulkan.
        const SPIRV = SDL_GPUShaderFormat::SPIRV.0;
        /// DXBC Shader Model 5.1 shaders for D3D12.
        const DXBC = SDL_GPUShaderFormat::DXBC.0;
        /// DXIL Shader Model 6.0 shaders for D3D12.
        const DXIL = SDL_GPUShaderFormat::DXIL.0;
        /// MSL shaders for Metal.
        const MSL = SDL_GPUShaderFormat::MSL.0;
        /// Precompiled Metal library shaders for Metal.
        const METALLIB = SDL_GPUShaderFormat::METALLIB.0;
    }
}

impl_enum_transmute!(SDL_GPUShaderFormat, ShaderFormats);

impl From<ShaderFormat> for ShaderFormats {
    fn from(value: ShaderFormat) -> Self {
        value.as_mask()
    }
}

/// Check whether a GPU runtime supports the requested shader formats.
///
/// `fmts` lists the shader formats the application can provide. SDL lets the
/// runtime choose the optimal driver; use [`Device::builder`](device::Device::builder)
/// when a preferred driver or additional device properties are needed.
///
/// Returns `true` if a compatible GPU runtime is available.
#[doc(alias = "SDL_GPUSupportsShaderFormats")]
pub fn are_formats_supported(fmts: ShaderFormats) -> bool {
    let fmts = SDL_GPUShaderFormat::new(fmts.bits());
    unsafe { SDL_GPUSupportsShaderFormats(fmts, std::ptr::null()) }
}

/// Return the number of GPU drivers compiled into SDL.
#[doc(alias = "SDL_GetNumGPUDrivers")]
pub fn num_drivers() -> i32 {
    unsafe { SDL_GetNumGPUDrivers() }
}

/// Return the name of a built-in GPU driver by index.
///
/// `i` is the driver index, in the order SDL normally checks drivers during
/// initialization. Returns `None` when the index is out of bounds. Driver names
/// are low-ASCII identifiers such as `"vulkan"`, `"metal"`, and `"direct3d12"`.
#[doc(alias = "SDL_GetGPUDriver")]
pub fn driver(i: i32) -> Option<&'static str> {
    let ptr = unsafe { SDL_GetGPUDriver(i) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { c_ptr_to_str(ptr) })
    }
}

/// Calculate the size in bytes of a texture format with dimensions.
///
/// `format` is the texture format, `width` and `height` are measured in pixels,
/// and `depth_or_layer_count` is the depth for a 3D texture or the layer count
/// for other texture types.
#[doc(alias = "SDL_CalculateGPUTextureFormatSize")]
pub fn calculate_texture_format_size(
    format: TextureFormat,
    width: u32,
    height: u32,
    depth_or_layer_count: u32,
) -> u32 {
    unsafe {
        SDL_CalculateGPUTextureFormatSize(
            SDL_GPUTextureFormat::new(format as _),
            width,
            height,
            depth_or_layer_count,
        )
    }
}

/// Return the texel-block size of a texture format in bytes.
///
/// `format` is the texture format to inspect. This is useful when aligning
/// texture upload data.
#[doc(alias = "SDL_GPUTextureFormatTexelBlockSize")]
pub fn texture_format_texel_block_size(format: TextureFormat) -> u32 {
    unsafe { SDL_GPUTextureFormatTexelBlockSize(SDL_GPUTextureFormat::new(format as _)) }
}

/// Convert an SDL pixel format to the corresponding GPU texture format.
///
/// `pixel_format` is the SDL pixel format to convert. Returns
/// [`None`] when no corresponding GPU format exists.
#[doc(alias = "SDL_GetGPUTextureFormatFromPixelFormat")]
pub fn texture_format_from_pixel_format(pixel_format: SDL_PixelFormat) -> Option<TextureFormat> {
    let fmt = SDL_GetGPUTextureFormatFromPixelFormat(pixel_format);
    if fmt == SDL_GPUTextureFormat::INVALID {
        None
    } else {
        Some(fmt.into())
    }
}

/// Convert a GPU texture format to the corresponding SDL pixel format.
///
/// `format` is the GPU texture format to convert. Returns
/// [`None`] when no corresponding SDL pixel format exists.
#[doc(alias = "SDL_GetPixelFormatFromGPUTextureFormat")]
pub fn pixel_format_from_texture_format(format: TextureFormat) -> Option<PixelFormat> {
    let fmt = SDL_GetPixelFormatFromGPUTextureFormat(SDL_GPUTextureFormat::new(format as _));
    if fmt == SDL_PixelFormat::UNKNOWN {
        None
    } else {
        Some(fmt.into())
    }
}

/// Check whether a GPU runtime supports a set of device properties.
///
/// `props` is the property group to use. Returns `true` if the configuration
/// is supported.
#[doc(alias = "SDL_GPUSupportsProperties")]
pub fn supports_properties(props: Ref<Properties>) -> bool {
    unsafe { SDL_GPUSupportsProperties(props.id()) }
}
