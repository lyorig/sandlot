//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUTexture
//! - [x] SDL_DownloadFromGPUTexture
//! - [x] SDL_ReleaseGPUTexture
//! - [x] SDL_SetGPUTextureName
//! - [x] SDL_UploadToGPUTexture

use std::{ffi::CStr, marker::PhantomData};

use bitflags::bitflags;
use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result, gpu::Cycle, properties::Properties, rect::Point, resource::Ref,
    resource::resource_new_no_drop, util::impl_enum_transmute, util::mod_reexport,
};

use super::{
    copy_pass::CopyPass, device::Device, sampler::Sampler, transfer_buffer::TransferBuffer,
};

mod_reexport!(builder);

/// The dimensionality of a texture.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUTextureType")]
pub enum TextureType {
    /// A two-dimensional image.
    _2d = SDL_GPUTextureType::_2D.0,
    /// An array of two-dimensional images.
    _2dArray = SDL_GPUTextureType::_2D_ARRAY.0,
    /// A three-dimensional image.
    _3d = SDL_GPUTextureType::_3D.0,
    /// A cube image.
    Cube = SDL_GPUTextureType::CUBE.0,
    /// An array of cube images.
    CubeArray = SDL_GPUTextureType::CUBE_ARRAY.0,
}

impl_enum_transmute!(SDL_GPUTextureType, TextureType);

bitflags! {
    /// Specifies how a texture is intended to be used.
    ///
    /// At least one usage flag is required. [`Self::SAMPLER`] cannot be combined
    /// with storage-read flags. Compute storage read and write usages allow one
    /// shader to read and another to write; [`Self::COMPUTE_STORAGE_READ_WRITE`]
    /// additionally allows reads and writes in the same shader or compute pass.
    /// The simultaneous-read-write mode has no synchronization within a pass,
    /// so callers must avoid data races, and it is supported only by some formats.
    #[derive(Clone, Copy)]
    #[doc(alias = "SDL_GPUTextureUsageFlags")]
    pub struct TextureUsageFlags: u32 {
        /// The texture supports sampling.
        const SAMPLER = SDL_GPUTextureUsageFlags::SAMPLER.0;
        /// The texture is a color render target.
        const COLOR_TARGET = SDL_GPUTextureUsageFlags::COLOR_TARGET.0;
        /// The texture is a depth-stencil render target.
        const DEPTH_STENCIL_TARGET = SDL_GPUTextureUsageFlags::DEPTH_STENCIL_TARGET.0;
        /// The texture supports storage reads in graphics stages.
        const GRAPHICS_STORAGE_READ = SDL_GPUTextureUsageFlags::GRAPHICS_STORAGE_READ.0;
        /// The texture supports storage reads in the compute stage.
        const COMPUTE_STORAGE_READ = SDL_GPUTextureUsageFlags::COMPUTE_STORAGE_READ.0;
        /// The texture supports storage writes in the compute stage.
        const COMPUTE_STORAGE_WRITE = SDL_GPUTextureUsageFlags::COMPUTE_STORAGE_WRITE.0;
        /// The texture supports simultaneous reads and writes in one compute shader.
        const COMPUTE_STORAGE_READ_WRITE = SDL_GPUTextureUsageFlags::COMPUTE_STORAGE_SIMULTANEOUS_READ_WRITE.0;
    }
}

impl_enum_transmute!(SDL_GPUTextureUsageFlags, TextureUsageFlags);

/// The number of samples per texel for a texture used as a render target.
///
/// This enum implements [`TryFrom<u8>`] and [`Into<u8>`] for easy conversions
/// between the enum variant and actual numeric sample count value.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUSampleCount")]
pub enum SampleCount {
    /// No multisampling.
    One = SDL_GPUSampleCount::_1.0,
    /// Two-sample multisampling.
    Two = SDL_GPUSampleCount::_2.0,
    /// Four-sample multisampling.
    Four = SDL_GPUSampleCount::_4.0,
    /// Eight-sample multisampling.
    Eight = SDL_GPUSampleCount::_8.0,
}

impl_enum_transmute!(SDL_GPUSampleCount, SampleCount);

impl TryFrom<u8> for SampleCount {
    type Error = ();

    /// Converts a [`u8`] representing the sample count into [`SampleCount`].
    /// Returns [`Err`] if `count` is not one of the numeric values specified
    /// by this enum`s variants.
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            _ => Err(()),
        }
    }
}

impl From<SampleCount> for u8 {
    /// Converts this enum into a `u8`, whose value represents the numeric value
    /// of the variant name of `self`.
    fn from(value: SampleCount) -> Self {
        match value {
            SampleCount::One => 1,
            SampleCount::Two => 2,
            SampleCount::Four => 4,
            SampleCount::Eight => 8,
        }
    }
}

/// The pixel format of a texture.
///
/// Format support depends on the driver, hardware, and usage flags. Query
/// [`crate::gpu::DeviceHandle::texture_supports_format`] before relying on a
/// format. The universally supported formats include common RGBA/BGRA,
/// floating-point, integer, depth, and simultaneous-read-write formats, but
/// support is usage-dependent; in particular, check D24/D32 depth formats on
/// the target device.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUTextureFormat")]
pub enum TextureFormat {
    /// 8-bit unsigned normalized single-channel color.
    A8Unorm = SDL_GPUTextureFormat::A8_UNORM.0,
    /// 8-bit unsigned normalized single-channel color.
    R8Unorm = SDL_GPUTextureFormat::R8_UNORM.0,
    /// 8-bit unsigned normalized two-channel color.
    R8g8Unorm = SDL_GPUTextureFormat::R8G8_UNORM.0,
    /// 8-bit unsigned normalized four-channel color.
    R8g8b8a8Unorm = SDL_GPUTextureFormat::R8G8B8A8_UNORM.0,
    /// 16-bit unsigned normalized single-channel color.
    R16Unorm = SDL_GPUTextureFormat::R16_UNORM.0,
    /// 16-bit unsigned normalized two-channel color.
    R16g16Unorm = SDL_GPUTextureFormat::R16G16_UNORM.0,
    /// 16-bit unsigned normalized four-channel color.
    R16g16b16a16Unorm = SDL_GPUTextureFormat::R16G16B16A16_UNORM.0,
    /// 10-bit RGB and 2-bit alpha unsigned normalized color.
    R10g10b10a2Unorm = SDL_GPUTextureFormat::R10G10B10A2_UNORM.0,
    /// 5-bit red, 6-bit green, and 5-bit blue unsigned normalized color.
    B5g6r5Unorm = SDL_GPUTextureFormat::B5G6R5_UNORM.0,
    /// 5-bit red, green, and blue with 1-bit alpha unsigned normalized color.
    B5g5r5a1Unorm = SDL_GPUTextureFormat::B5G5R5A1_UNORM.0,
    /// 4-bit red, green, blue, and alpha unsigned normalized color.
    B4g4r4a4Unorm = SDL_GPUTextureFormat::B4G4R4A4_UNORM.0,
    /// 8-bit blue, green, red, and alpha unsigned normalized color.
    B8g8r8a8Unorm = SDL_GPUTextureFormat::B8G8R8A8_UNORM.0,
    /// BC1 compressed unsigned normalized RGBA color.
    Bc1RgbaUnorm = SDL_GPUTextureFormat::BC1_RGBA_UNORM.0,
    /// BC2 compressed unsigned normalized RGBA color.
    Bc2RgbaUnorm = SDL_GPUTextureFormat::BC2_RGBA_UNORM.0,
    /// BC3 compressed unsigned normalized RGBA color.
    Bc3RgbaUnorm = SDL_GPUTextureFormat::BC3_RGBA_UNORM.0,
    /// BC4 compressed unsigned normalized red color.
    Bc4RUnorm = SDL_GPUTextureFormat::BC4_R_UNORM.0,
    /// BC5 compressed unsigned normalized red-green color.
    Bc5RgUnorm = SDL_GPUTextureFormat::BC5_RG_UNORM.0,
    /// BC7 compressed unsigned normalized RGBA color.
    Bc7RgbaUnorm = SDL_GPUTextureFormat::BC7_RGBA_UNORM.0,
    /// BC6H compressed signed-float RGB color.
    Bc6hRgbFloat = SDL_GPUTextureFormat::BC6H_RGB_FLOAT.0,
    /// BC6H compressed unsigned-float RGB color.
    Bc6hRgbUfloat = SDL_GPUTextureFormat::BC6H_RGB_UFLOAT.0,
    /// 8-bit signed normalized single-channel color.
    R8Snorm = SDL_GPUTextureFormat::R8_SNORM.0,
    /// 8-bit signed normalized two-channel color.
    R8g8Snorm = SDL_GPUTextureFormat::R8G8_SNORM.0,
    /// 8-bit signed normalized four-channel color.
    R8g8b8a8Snorm = SDL_GPUTextureFormat::R8G8B8A8_SNORM.0,
    /// 16-bit signed normalized single-channel color.
    R16Snorm = SDL_GPUTextureFormat::R16_SNORM.0,
    /// 16-bit signed normalized two-channel color.
    R16g16Snorm = SDL_GPUTextureFormat::R16G16_SNORM.0,
    /// 16-bit signed normalized four-channel color.
    R16g16b16a16Snorm = SDL_GPUTextureFormat::R16G16B16A16_SNORM.0,
    /// 16-bit floating-point single-channel color.
    R16Float = SDL_GPUTextureFormat::R16_FLOAT.0,
    /// 16-bit floating-point two-channel color.
    R16g16Float = SDL_GPUTextureFormat::R16G16_FLOAT.0,
    /// 16-bit floating-point four-channel color.
    R16g16b16a16Float = SDL_GPUTextureFormat::R16G16B16A16_FLOAT.0,
    /// 32-bit floating-point single-channel color.
    R32Float = SDL_GPUTextureFormat::R32_FLOAT.0,
    /// 32-bit floating-point two-channel color.
    R32g32Float = SDL_GPUTextureFormat::R32G32_FLOAT.0,
    /// 32-bit floating-point four-channel color.
    R32g32b32a32Float = SDL_GPUTextureFormat::R32G32B32A32_FLOAT.0,
    /// 11-bit red, 11-bit green, and 10-bit blue unsigned-float color.
    R11g11b10Ufloat = SDL_GPUTextureFormat::R11G11B10_UFLOAT.0,
    /// 8-bit unsigned integer single-channel color.
    R8Uint = SDL_GPUTextureFormat::R8_UINT.0,
    /// 8-bit unsigned integer two-channel color.
    R8g8Uint = SDL_GPUTextureFormat::R8G8_UINT.0,
    /// 8-bit unsigned integer four-channel color.
    R8g8b8a8Uint = SDL_GPUTextureFormat::R8G8B8A8_UINT.0,
    /// 16-bit unsigned integer single-channel color.
    R16Uint = SDL_GPUTextureFormat::R16_UINT.0,
    /// 16-bit unsigned integer two-channel color.
    R16g16Uint = SDL_GPUTextureFormat::R16G16_UINT.0,
    /// 16-bit unsigned integer four-channel color.
    R16g16b16a16Uint = SDL_GPUTextureFormat::R16G16B16A16_UINT.0,
    /// 32-bit unsigned integer single-channel color.
    R32Uint = SDL_GPUTextureFormat::R32_UINT.0,
    /// 32-bit unsigned integer two-channel color.
    R32g32Uint = SDL_GPUTextureFormat::R32G32_UINT.0,
    /// 32-bit unsigned integer four-channel color.
    R32g32b32a32Uint = SDL_GPUTextureFormat::R32G32B32A32_UINT.0,
    /// 8-bit signed integer single-channel color.
    R8Int = SDL_GPUTextureFormat::R8_INT.0,
    /// 8-bit signed integer two-channel color.
    R8g8Int = SDL_GPUTextureFormat::R8G8_INT.0,
    /// 8-bit signed integer four-channel color.
    R8g8b8a8Int = SDL_GPUTextureFormat::R8G8B8A8_INT.0,
    /// 16-bit signed integer single-channel color.
    R16Int = SDL_GPUTextureFormat::R16_INT.0,
    /// 16-bit signed integer two-channel color.
    R16g16Int = SDL_GPUTextureFormat::R16G16_INT.0,
    /// 16-bit signed integer four-channel color.
    R16g16b16a16Int = SDL_GPUTextureFormat::R16G16B16A16_INT.0,
    /// 32-bit signed integer single-channel color.
    R32Int = SDL_GPUTextureFormat::R32_INT.0,
    /// 32-bit signed integer two-channel color.
    R32g32Int = SDL_GPUTextureFormat::R32G32_INT.0,
    /// 32-bit signed integer four-channel color.
    R32g32b32a32Int = SDL_GPUTextureFormat::R32G32B32A32_INT.0,
    /// 8-bit unsigned normalized four-channel sRGB color.
    R8g8b8a8UnormSrgb = SDL_GPUTextureFormat::R8G8B8A8_UNORM_SRGB.0,
    /// 8-bit unsigned normalized blue-green-red-alpha sRGB color.
    B8g8r8a8UnormSrgb = SDL_GPUTextureFormat::B8G8R8A8_UNORM_SRGB.0,
    /// BC1 compressed unsigned normalized sRGB RGBA color.
    Bc1RgbaUnormSrgb = SDL_GPUTextureFormat::BC1_RGBA_UNORM_SRGB.0,
    /// BC2 compressed unsigned normalized sRGB RGBA color.
    Bc2RgbaUnormSrgb = SDL_GPUTextureFormat::BC2_RGBA_UNORM_SRGB.0,
    /// BC3 compressed unsigned normalized sRGB RGBA color.
    Bc3RgbaUnormSrgb = SDL_GPUTextureFormat::BC3_RGBA_UNORM_SRGB.0,
    /// BC7 compressed unsigned normalized sRGB RGBA color.
    Bc7RgbaUnormSrgb = SDL_GPUTextureFormat::BC7_RGBA_UNORM_SRGB.0,
    /// 16-bit unsigned normalized depth format.
    D16Unorm = SDL_GPUTextureFormat::D16_UNORM.0,
    /// 24-bit unsigned normalized depth format.
    D24Unorm = SDL_GPUTextureFormat::D24_UNORM.0,
    /// 32-bit floating-point depth format.
    D32Float = SDL_GPUTextureFormat::D32_FLOAT.0,
    /// 24-bit unsigned normalized depth with 8-bit unsigned integer stencil.
    D24UnormS8Uint = SDL_GPUTextureFormat::D24_UNORM_S8_UINT.0,
    /// 32-bit floating-point depth with 8-bit unsigned integer stencil.
    D32FloatS8Uint = SDL_GPUTextureFormat::D32_FLOAT_S8_UINT.0,
    /// 4x4 ASTC unsigned normalized color.
    Astc4x4Unorm = SDL_GPUTextureFormat::ASTC_4x4_UNORM.0,
    /// 5x4 ASTC unsigned normalized color.
    Astc5x4Unorm = SDL_GPUTextureFormat::ASTC_5x4_UNORM.0,
    /// 5x5 ASTC unsigned normalized color.
    Astc5x5Unorm = SDL_GPUTextureFormat::ASTC_5x5_UNORM.0,
    /// 6x5 ASTC unsigned normalized color.
    Astc6x5Unorm = SDL_GPUTextureFormat::ASTC_6x5_UNORM.0,
    /// 6x6 ASTC unsigned normalized color.
    Astc6x6Unorm = SDL_GPUTextureFormat::ASTC_6x6_UNORM.0,
    /// 8x5 ASTC unsigned normalized color.
    Astc8x5Unorm = SDL_GPUTextureFormat::ASTC_8x5_UNORM.0,
    /// 8x6 ASTC unsigned normalized color.
    Astc8x6Unorm = SDL_GPUTextureFormat::ASTC_8x6_UNORM.0,
    /// 8x8 ASTC unsigned normalized color.
    Astc8x8Unorm = SDL_GPUTextureFormat::ASTC_8x8_UNORM.0,
    /// 10x5 ASTC unsigned normalized color.
    Astc10x5Unorm = SDL_GPUTextureFormat::ASTC_10x5_UNORM.0,
    /// 10x6 ASTC unsigned normalized color.
    Astc10x6Unorm = SDL_GPUTextureFormat::ASTC_10x6_UNORM.0,
    /// 10x8 ASTC unsigned normalized color.
    Astc10x8Unorm = SDL_GPUTextureFormat::ASTC_10x8_UNORM.0,
    /// 10x10 ASTC unsigned normalized color.
    Astc10x10Unorm = SDL_GPUTextureFormat::ASTC_10x10_UNORM.0,
    /// 12x10 ASTC unsigned normalized color.
    Astc12x10Unorm = SDL_GPUTextureFormat::ASTC_12x10_UNORM.0,
    /// 12x12 ASTC unsigned normalized color.
    Astc12x12Unorm = SDL_GPUTextureFormat::ASTC_12x12_UNORM.0,
    /// 4x4 ASTC unsigned normalized sRGB color.
    Astc4x4UnormSrgb = SDL_GPUTextureFormat::ASTC_4x4_UNORM_SRGB.0,
    /// 5x4 ASTC unsigned normalized sRGB color.
    Astc5x4UnormSrgb = SDL_GPUTextureFormat::ASTC_5x4_UNORM_SRGB.0,
    /// 5x5 ASTC unsigned normalized sRGB color.
    Astc5x5UnormSrgb = SDL_GPUTextureFormat::ASTC_5x5_UNORM_SRGB.0,
    /// 6x5 ASTC unsigned normalized sRGB color.
    Astc6x5UnormSrgb = SDL_GPUTextureFormat::ASTC_6x5_UNORM_SRGB.0,
    /// 6x6 ASTC unsigned normalized sRGB color.
    Astc6x6UnormSrgb = SDL_GPUTextureFormat::ASTC_6x6_UNORM_SRGB.0,
    /// 8x5 ASTC unsigned normalized sRGB color.
    Astc8x5UnormSrgb = SDL_GPUTextureFormat::ASTC_8x5_UNORM_SRGB.0,
    /// 8x6 ASTC unsigned normalized sRGB color.
    Astc8x6UnormSrgb = SDL_GPUTextureFormat::ASTC_8x6_UNORM_SRGB.0,
    /// 8x8 ASTC unsigned normalized sRGB color.
    Astc8x8UnormSrgb = SDL_GPUTextureFormat::ASTC_8x8_UNORM_SRGB.0,
    /// 10x5 ASTC unsigned normalized sRGB color.
    Astc10x5UnormSrgb = SDL_GPUTextureFormat::ASTC_10x5_UNORM_SRGB.0,
    /// 10x6 ASTC unsigned normalized sRGB color.
    Astc10x6UnormSrgb = SDL_GPUTextureFormat::ASTC_10x6_UNORM_SRGB.0,
    /// 10x8 ASTC unsigned normalized sRGB color.
    Astc10x8UnormSrgb = SDL_GPUTextureFormat::ASTC_10x8_UNORM_SRGB.0,
    /// 10x10 ASTC unsigned normalized sRGB color.
    Astc10x10UnormSrgb = SDL_GPUTextureFormat::ASTC_10x10_UNORM_SRGB.0,
    /// 12x10 ASTC unsigned normalized sRGB color.
    Astc12x10UnormSrgb = SDL_GPUTextureFormat::ASTC_12x10_UNORM_SRGB.0,
    /// 12x12 ASTC unsigned normalized sRGB color.
    Astc12x12UnormSrgb = SDL_GPUTextureFormat::ASTC_12x12_UNORM_SRGB.0,
    /// 4x4 ASTC signed-float color.
    Astc4x4Float = SDL_GPUTextureFormat::ASTC_4x4_FLOAT.0,
    /// 5x4 ASTC signed-float color.
    Astc5x4Float = SDL_GPUTextureFormat::ASTC_5x4_FLOAT.0,
    /// 5x5 ASTC signed-float color.
    Astc5x5Float = SDL_GPUTextureFormat::ASTC_5x5_FLOAT.0,
    /// 6x5 ASTC signed-float color.
    Astc6x5Float = SDL_GPUTextureFormat::ASTC_6x5_FLOAT.0,
    /// 6x6 ASTC signed-float color.
    Astc6x6Float = SDL_GPUTextureFormat::ASTC_6x6_FLOAT.0,
    /// 8x5 ASTC signed-float color.
    Astc8x5Float = SDL_GPUTextureFormat::ASTC_8x5_FLOAT.0,
    /// 8x6 ASTC signed-float color.
    Astc8x6Float = SDL_GPUTextureFormat::ASTC_8x6_FLOAT.0,
    /// 8x8 ASTC signed-float color.
    Astc8x8Float = SDL_GPUTextureFormat::ASTC_8x8_FLOAT.0,
    /// 10x5 ASTC signed-float color.
    Astc10x5Float = SDL_GPUTextureFormat::ASTC_10x5_FLOAT.0,
    /// 10x6 ASTC signed-float color.
    Astc10x6Float = SDL_GPUTextureFormat::ASTC_10x6_FLOAT.0,
    /// 10x8 ASTC signed-float color.
    Astc10x8Float = SDL_GPUTextureFormat::ASTC_10x8_FLOAT.0,
    /// 10x10 ASTC signed-float color.
    Astc10x10Float = SDL_GPUTextureFormat::ASTC_10x10_FLOAT.0,
    /// 12x10 ASTC signed-float color.
    Astc12x10Float = SDL_GPUTextureFormat::ASTC_12x10_FLOAT.0,
    /// 12x12 ASTC signed-float color.
    Astc12x12Float = SDL_GPUTextureFormat::ASTC_12x12_FLOAT.0,
}

impl_enum_transmute!(SDL_GPUTextureFormat, TextureFormat, INVALID);

/// Parameters for creating a texture.
///
/// Usage flags may be combined, but some combinations are invalid, such as
/// [`TextureUsageFlags::SAMPLER`] with storage-read usage. The wrapper leaves
/// SDL's extension-property ID set to zero.
#[doc(alias = "SDL_GPUTextureCreateInfo")]
#[derive(Clone, Copy)]
pub struct TextureCreateInfo(SDL_GPUTextureCreateInfo);
impl TextureCreateInfo {
    /// Describe a texture's type, format, usage, dimensions, mip levels, and
    /// sample count.
    ///
    /// * `kind` is the base dimensionality.
    /// * `format` is the pixel format.
    /// * `usage` specifies how the texture will be used.
    /// * `size` contains the width and height.
    /// * `layer_count_or_depth` is the layer count for 2D arrays or the depth
    ///   for 3D textures.
    /// * `num_levels` is the number of mip levels.
    /// * `samples` is the number of samples per texel and applies only when the
    ///   texture is used as a render target.
    ///
    /// If the requested sample count exceeds hardware support, SDL falls back
    /// to the highest available sample count. Texture contents are undefined
    /// until written by an upload or render/compute pass.
    pub const fn new(
        kind: TextureType,
        format: TextureFormat,
        usage: TextureUsageFlags,
        size: Point<u32>,
        layer_count_or_depth: u32,
        num_levels: u32,
        samples: SampleCount,
    ) -> Self {
        let inner = SDL_GPUTextureCreateInfo {
            r#type: SDL_GPUTextureType::new(kind as _),
            format: SDL_GPUTextureFormat::new(format as _),
            usage: SDL_GPUTextureUsageFlags::new(usage.bits()),
            width: size.x,
            height: size.y,
            layer_count_or_depth,
            num_levels,
            sample_count: SDL_GPUSampleCount::new(samples as _),
            props: SDL_PropertiesID::new(0),
        };

        Self(inner)
    }
}

/// Parameters for transferring image data to or from a texture.
///
/// The transfer buffer is borrowed for `'tb`. `offset` is the starting byte of
/// image data, `pixels_per_row` is the number of pixels between rows, and
/// `rows_per_layer` is the number of rows between layers or depth slices.
///
/// If either layout count is zero, the corresponding texture-region width or
/// height is used and the data is treated as tightly packed. On some older or
/// integrated hardware, Direct3D 12 prefers row pitches aligned to 256 bytes
/// and offsets aligned to 512 bytes; otherwise SDL may make a temporary copy.
#[doc(alias = "SDL_GPUTextureTransferInfo")]
#[derive(Clone, Copy)]
pub struct TextureTransferInfo<'tb>(
    SDL_GPUTextureTransferInfo,
    PhantomData<Ref<'tb, TransferBuffer>>,
);

impl<'tb> TextureTransferInfo<'tb> {
    /// Describe image data in `tb` beginning at `offset` with the given row and
    /// layer layout.
    pub fn new(
        tb: Ref<'tb, TransferBuffer>,
        offset: u32,
        pixels_per_row: u32,
        rows_per_layer: u32,
    ) -> Self {
        let transfer_buffer = tb.handle.as_ptr();
        let inner = SDL_GPUTextureTransferInfo {
            transfer_buffer,
            offset,
            pixels_per_row,
            rows_per_layer,
        };
        Self(inner, PhantomData)
    }
}

/// A region of a texture used for data transfers.
///
/// The texture is borrowed for `'t`. The region identifies a mip level and
/// layer, starts at `(x, y, z)`, and has `(width, height, depth)` dimensions.
#[doc(alias = "SDL_GPUTextureRegion")]
#[derive(Clone, Copy)]
pub struct TextureRegion<'t>(SDL_GPUTextureRegion, PhantomData<Ref<'t, Texture>>);
impl<'t> TextureRegion<'t> {
    /// Describe a texture region at the given mip level, layer, position, and
    /// dimensions.
    pub fn new(
        tex: Ref<'t, Texture>,
        mip_level: u32,
        layer: u32,
        (x, y, z): (u32, u32, u32),
        (width, height, depth): (u32, u32, u32),
    ) -> Self {
        let texture = tex.handle.as_ptr();
        let inner = SDL_GPUTextureRegion {
            texture,
            mip_level,
            layer,
            x,
            y,
            z,
            w: width,
            h: height,
            d: depth,
        };

        Self(inner, PhantomData)
    }

    pub fn whole_2d(tex: Ref<'t, Texture>, (w, h): (u32, u32)) -> Self {
        Self::new(tex, 0, 0, (0, 0, 0), (w, h, 1))
    }
}

/// A location in a texture used when copying between textures.
///
/// The texture is borrowed for `'t`. The location identifies a mip level and
/// layer, plus the `(x, y, z)` coordinate within that subresource.
#[doc(alias = "SDL_GPUTextureLocation")]
#[derive(Clone, Copy)]
pub struct TextureLocation<'t>(
    pub(crate) SDL_GPUTextureLocation,
    PhantomData<Ref<'t, Texture>>,
);

impl<'t> TextureLocation<'t> {
    /// Describe a location at the given mip level, layer, and coordinate.
    pub fn new(
        tex: Ref<'t, Texture>,
        mip_level: u32,
        layer: u32,
        (x, y, z): (u32, u32, u32),
    ) -> Self {
        let texture = tex.handle.as_ptr();
        let inner = SDL_GPUTextureLocation {
            texture,
            mip_level,
            layer,
            x,
            y,
            z,
        };
        Self(inner, PhantomData)
    }

    /// Same as [`Self::new`], with all parameters set to zero.
    pub fn at_start(tex: Ref<'t, Texture>) -> Self {
        Self::new(tex, 0, 0, (0, 0, 0))
    }
}

/// Parameters for binding a texture and sampler together.
///
/// The texture must have been created with [`TextureUsageFlags::SAMPLER`]. The
/// texture and sampler are borrowed for `'t` and `'s`, respectively.
#[doc(alias = "SDL_GPUTextureSamplerBinding")]
#[derive(Clone, Copy)]
pub struct TextureSamplerBinding<'t, 's>(
    SDL_GPUTextureSamplerBinding,
    PhantomData<Ref<'t, Texture>>,
    PhantomData<Ref<'s, Sampler>>,
);

impl<'t, 's> TextureSamplerBinding<'t, 's> {
    /// Bind `texture` to `sampler`.
    pub fn new(texture: Ref<'t, Texture>, sampler: Ref<'s, Sampler>) -> Self {
        Self(
            SDL_GPUTextureSamplerBinding {
                texture: texture.handle.as_ptr(),
                sampler: sampler.handle.as_ptr(),
            },
            PhantomData,
            PhantomData,
        )
    }
}

/// Parameters for binding a texture for read-write access in a compute pass.
///
/// The texture must have been created with
/// [`TextureUsageFlags::COMPUTE_STORAGE_WRITE`] or
/// [`TextureUsageFlags::COMPUTE_STORAGE_READ_WRITE`]. The texture is borrowed
/// for `'t`; `mip_level` and `layer` select the subresource, and `cycle` controls
/// whether SDL cycles the texture when it is already bound.
#[doc(alias = "SDL_GPUStorageTextureReadWriteBinding")]
#[derive(Clone, Copy)]
pub struct StorageTextureReadWriteBinding<'t>(
    SDL_GPUStorageTextureReadWriteBinding,
    PhantomData<Ref<'t, Texture>>,
);

impl<'t> StorageTextureReadWriteBinding<'t> {
    /// Bind the selected mip level and layer of `texture` for read-write access.
    pub fn new(texture: Ref<'t, Texture>, mip_level: u32, layer: u32, cycle: Cycle) -> Self {
        Self(
            SDL_GPUStorageTextureReadWriteBinding {
                texture: texture.handle.as_ptr(),
                mip_level,
                layer,
                cycle: cycle.into(),
                ..Default::default()
            },
            PhantomData,
        )
    }
}

/// A texture region used in a blit operation.
///
/// The texture is borrowed for `'t`. `layer_or_depth_plane` is a layer for 2D
/// array and cube textures, or a depth plane for 3D textures. The region starts
/// at `(x, y)` and has dimensions `(w, h)`.
#[doc(alias = "SDL_GPUBlitRegion")]
#[derive(Clone, Copy)]
pub struct BlitRegion<'t>(pub(crate) SDL_GPUBlitRegion, PhantomData<Ref<'t, Texture>>);
impl<'t> BlitRegion<'t> {
    /// Describe a blit region at the given mip level, layer or depth plane, and
    /// position.
    pub fn new(
        tex: Ref<'t, Texture>,
        mip_level: u32,
        layer_or_depth_plane: u32,
        (x, y, w, h): (u32, u32, u32, u32),
    ) -> Self {
        let texture = tex.handle.as_ptr();
        Self(
            SDL_GPUBlitRegion {
                texture,
                mip_level,
                layer_or_depth_plane,
                x,
                y,
                w,
                h,
            },
            PhantomData,
        )
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a texture.
    SDL_GPUTexture, Texture
);
impl Texture {
    /// Build a [`Texture`] with additional parameters not available in [`TextureCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> TextureBuilder<'_> {
        TextureBuilder::new(props)
    }

    /// Create a texture for use in graphics or compute workflows.
    ///
    /// `device` is the GPU device that owns the texture, and `create_info`
    /// describes its type, format, usage, dimensions, mip levels, and samples.
    /// Texture contents are undefined until written by an upload or render/
    /// compute pass.
    ///
    /// Returns [`Err`] if the texture cannot be created or its usage combination
    /// is invalid.
    #[doc(alias = "SDL_CreateGPUTexture")]
    pub fn new(device: Ref<Device>, create_info: &TextureCreateInfo) -> Result<Self> {
        let handle =
            unsafe { SDL_CreateGPUTexture(device.handle.as_ptr(), &raw const create_info.0) };

        Self::from_ptr(handle)
    }

    /// Release a texture as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the texture. This method consumes
    /// the texture; it must not be referenced after this call. Unlike ordinary
    /// RAII resources, a texture created with this module has no automatic
    /// destructor, so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUTexture")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUTexture(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}

impl TextureHandle {
    /// Copy data from this texture to a transfer buffer on the GPU timeline.
    ///
    /// * `copy_pass` records the download.
    /// * `src` identifies the source texture region.
    /// * `dst` identifies the destination transfer buffer and image layout.
    ///
    /// The data is not guaranteed to have been copied until the command-buffer
    /// fence is signaled.
    #[doc(alias = "SDL_DownloadFromGPUTexture")]
    pub fn download(
        &self,
        copy_pass: Ref<CopyPass>,
        src: &TextureRegion,
        dst: &TextureTransferInfo,
    ) {
        unsafe {
            SDL_DownloadFromGPUTexture(
                copy_pass.handle.as_ptr(),
                &raw const src.0,
                &raw const dst.0,
            );
        }
    }

    /// Upload data from a transfer buffer to this texture on the GPU timeline.
    ///
    /// * `copy_pass` records the upload.
    /// * `src` identifies the source transfer buffer and image layout.
    /// * `dst` identifies the destination texture region.
    /// * `cycle` controls whether SDL cycles the texture if it is already bound;
    ///   otherwise existing data is overwritten.
    ///
    /// Subsequent commands can assume that the upload has finished. Transfer
    /// data must be aligned to a multiple of the texture format's texel size.
    #[doc(alias = "SDL_UploadToGPUTexture")]
    pub fn upload(
        &self,
        copy_pass: Ref<CopyPass>,
        src: &TextureTransferInfo,
        dst: &TextureRegion,
        cycle: Cycle,
    ) {
        unsafe {
            SDL_UploadToGPUTexture(
                copy_pass.handle.as_ptr(),
                &raw const src.0,
                &raw const dst.0,
                cycle.into(),
            );
        }
    }

    /// Attach a UTF-8 label to this texture.
    ///
    /// `device` is the GPU device that owns the texture, and `name` is the label
    /// used by debugging tools. To name a texture at creation time, prefer the
    /// texture-create name property when constructing it.
    #[doc(alias = "SDL_SetGPUTextureName")]
    pub fn set_name(&self, device: Ref<Device>, name: &CStr) {
        unsafe {
            SDL_SetGPUTextureName(device.handle.as_ptr(), self.handle.as_ptr(), name.as_ptr());
        }
    }
}
