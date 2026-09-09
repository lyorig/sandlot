//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUSampler
//! - [x] SDL_ReleaseGPUSampler

use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result,
    gpu::{EnableAnisotropy, EnableCompare},
    impl_enum_transmute, mod_reexport,
    properties::Properties,
    resource::Ref,
    resource_new_no_drop,
};

use super::device::Device;

mod_reexport!(builder);

/// A filtering operation used by a sampler.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUFilter")]
pub enum Filter {
    /// Point filtering.
    Nearest = SDL_GPUFilter::NEAREST.0,
    /// Linear filtering.
    Linear = SDL_GPUFilter::LINEAR.0,
}

/// A mipmap filtering mode used by a sampler.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUSamplerMipmapMode")]
pub enum SamplerMipmapMode {
    /// Point filtering between mipmap levels.
    Nearest = SDL_GPUSamplerMipmapMode::NEAREST.0,
    /// Linear filtering between mipmap levels.
    Linear = SDL_GPUSamplerMipmapMode::LINEAR.0,
}

/// The behavior of texture sampling when coordinates exceed the `[0, 1)` range.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUSamplerAddressMode")]
pub enum SamplerAddressMode {
    /// Wrap coordinates around.
    Repeat = SDL_GPUSamplerAddressMode::REPEAT.0,
    /// Wrap coordinates around with mirrored repeats.
    MirroredRepeat = SDL_GPUSamplerAddressMode::MIRRORED_REPEAT.0,
    /// Clamp coordinates to the `[0, 1)` range.
    ClampToEdge = SDL_GPUSamplerAddressMode::CLAMP_TO_EDGE.0,
}

/// A comparison operator for depth, stencil, and sampler operations.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUCompareOp")]
pub enum CompareOp {
    /// The comparison always evaluates to false.
    Never = SDL_GPUCompareOp::NEVER.0,
    /// The comparison evaluates `reference < test`.
    Less = SDL_GPUCompareOp::LESS.0,
    /// The comparison evaluates `reference == test`.
    Equal = SDL_GPUCompareOp::EQUAL.0,
    /// The comparison evaluates `reference <= test`.
    LessOrEqual = SDL_GPUCompareOp::LESS_OR_EQUAL.0,
    /// The comparison evaluates `reference > test`.
    Greater = SDL_GPUCompareOp::GREATER.0,
    /// The comparison evaluates `reference != test`.
    NotEqual = SDL_GPUCompareOp::NOT_EQUAL.0,
    /// The comparison evaluates `reference >= test`.
    GreaterOrEqual = SDL_GPUCompareOp::GREATER_OR_EQUAL.0,
    /// The comparison always evaluates to true.
    Always = SDL_GPUCompareOp::ALWAYS.0,
}

impl_enum_transmute!(SDL_GPUFilter, Filter);
impl_enum_transmute!(SDL_GPUSamplerMipmapMode, SamplerMipmapMode);
impl_enum_transmute!(SDL_GPUSamplerAddressMode, SamplerAddressMode);
impl_enum_transmute!(SDL_GPUCompareOp, CompareOp);

/// Parameters for creating a sampler.
///
/// The wrapper sets SDL's extension-property ID to zero because extensions are
/// not exposed by this constructor.
#[doc(alias = "SDL_GPUSamplerCreateInfo")]
#[derive(Clone, Copy)]
pub struct SamplerCreateInfo(SDL_GPUSamplerCreateInfo);
impl SamplerCreateInfo {
    /// Describe sampler filtering, addressing, LOD, anisotropy, and comparison
    /// behavior.
    ///
    /// * `min_filter` and `mag_filter` select filtering for minification and
    ///   magnification lookups.
    /// * `mipmap_mode` selects mipmap filtering.
    /// * `(u, v, w)` selects addressing for coordinates outside `[0, 1)` in
    ///   each dimension.
    /// * `mip_lod_bias` is added to the computed mipmap LOD.
    /// * `max_anisotropy` clamps the anisotropy value when `ea` enables
    ///   anisotropic filtering; it is ignored otherwise.
    /// * `compare_op` compares fetched data with a reference value when `ec`
    ///   enables comparison sampling.
    /// * `min_lod` and `max_lod` clamp the computed LOD.
    /// * `ea` enables anisotropic filtering, and `ec` enables comparison against
    ///   a reference value during lookups.
    ///
    /// The Metal driver ignores `mip_lod_bias`; apply the bias in the shader
    /// when targeting Metal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        min_filter: Filter,
        mag_filter: Filter,
        mipmap_mode: SamplerMipmapMode,
        (u, v, w): (SamplerAddressMode, SamplerAddressMode, SamplerAddressMode),
        mip_lod_bias: f32,
        max_anisotropy: f32,
        compare_op: CompareOp,
        min_lod: f32,
        max_lod: f32,
        ea: EnableAnisotropy,
        ec: EnableCompare,
    ) -> Self {
        let inner = SDL_GPUSamplerCreateInfo {
            min_filter: SDL_GPUFilter::new(min_filter as _),
            mag_filter: SDL_GPUFilter::new(mag_filter as _),
            mipmap_mode: SDL_GPUSamplerMipmapMode::new(mipmap_mode as _),
            address_mode_u: SDL_GPUSamplerAddressMode::new(u as _),
            address_mode_v: SDL_GPUSamplerAddressMode::new(v as _),
            address_mode_w: SDL_GPUSamplerAddressMode::new(w as _),
            mip_lod_bias,
            max_anisotropy,
            compare_op: SDL_GPUCompareOp::new(compare_op as _),
            min_lod,
            max_lod,
            enable_anisotropy: ea.into(),
            enable_compare: ec.into(),
            props: SDL_PropertiesID::new(0),
            ..Default::default()
        };
        Self(inner)
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a sampler.
    SDL_GPUSampler, Sampler
);
impl Sampler {
    /// Build a [`Sampler`] with additional parameters not available in [`SamplerCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> SamplerBuilder<'_> {
        SamplerBuilder::new(props)
    }

    /// Create a sampler for use when binding textures in a graphics workflow.
    ///
    /// `device` is the GPU device that owns the sampler, and `create_info`
    /// describes its filtering, addressing, LOD, anisotropy, and comparison
    /// behavior.
    ///
    /// Returns [`Err`] if the sampler cannot be created.
    #[doc(alias = "SDL_CreateGPUSampler")]
    pub fn new(device: Ref<Device>, create_info: &SamplerCreateInfo) -> Result<Self> {
        let handle =
            unsafe { SDL_CreateGPUSampler(device.handle.as_ptr(), &raw const create_info.0) };

        Self::from_ptr(handle)
    }

    /// Release a sampler as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the sampler. This method consumes
    /// the sampler; it must not be referenced after this call. Unlike ordinary
    /// RAII resources, a sampler created with this module has no automatic
    /// destructor, so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUSampler")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe { SDL_ReleaseGPUSampler(device.handle.as_ptr(), self.handle.as_ptr()) };
    }
}
