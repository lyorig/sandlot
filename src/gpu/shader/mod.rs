//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_CreateGPUShader
//! - [x] SDL_ReleaseGPUShader

use std::{ffi::CStr, marker::PhantomData};

use sdl3_sys::{gpu::*, properties::SDL_PropertiesID};

use crate::{
    Result, properties::Properties, resource::Ref, resource::resource_new_no_drop,
    util::mod_reexport,
};

use super::{ShaderFormat, device::Device};

mod_reexport!(builder);

/// The stage that a shader program corresponds to.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUShaderStage")]
pub enum ShaderStage {
    /// A vertex shader stage.
    Vertex = SDL_GPUShaderStage::VERTEX.0,
    /// A fragment shader stage.
    Fragment = SDL_GPUShaderStage::FRAGMENT.0,
}

/// Code and metadata for creating a shader object.
///
/// The create info borrows the shader bytecode and UTF-8 entry-point name for
/// the lifetimes encoded in its type. Those values must remain valid while the
/// create info is used to create the shader. The wrapper sets SDL's
/// extension-property ID to zero because extensions are not exposed here.
#[doc(alias = "SDL_GPUShaderCreateInfo")]
#[derive(Clone, Copy)]
pub struct ShaderCreateInfo<'bc, 'ep>(
    SDL_GPUShaderCreateInfo,
    PhantomData<&'bc [u8]>,
    PhantomData<&'ep CStr>,
);

impl<'bc, 'ep> ShaderCreateInfo<'bc, 'ep> {
    /// Describe shader code, stage, format, and resource bindings.
    ///
    /// * `code` is the shader code.
    /// * `entrypoint` is the null-terminated UTF-8 entry-point function name.
    /// * `fmt` is the format of the shader code.
    /// * `stage` is the shader stage.
    /// * `num_samplers` is the number of samplers defined in the shader.
    /// * The three resource counts are, in order, storage textures, storage
    ///   buffers, and uniform buffers.
    ///
    /// Resource bindings must follow the convention for `fmt`:
    ///
    /// - SPIR-V: vertex resources use set 0 and uniforms use set 1; fragment
    ///   resources use set 2 and uniforms use set 3. Within a resource set,
    ///   samplers come first, followed by storage textures and storage buffers.
    /// - DXBC/DXIL: vertex resources use `t[n]` and `s[n]` in space 0, with
    ///   uniforms in `b[n]` in space 1; fragment resources use space 2, with
    ///   uniforms in space 3. Within each resource set, sampled textures,
    ///   samplers, storage textures, and storage buffers follow SDL binding order.
    /// - MSL: sampled textures, then storage textures, use the `texture` table;
    ///   samplers use the `sampler` table; uniform buffers, then storage buffers,
    ///   use the `buffer` table. Each table starts at index 0 without gaps.
    ///   Vertex inputs should use `[[stage_in]]`; SDL maps vertex buffer slot 0
    ///   to buffer index 14, slot 1 to 15, and so on.
    ///
    /// In all formats, resource indices start at zero and are consecutive, and
    /// resources must appear in the order in which they are bound through SDL.
    /// For D3D12, non-system vertex semantics should use `TEXCOORD0`,
    /// `TEXCOORD1`, and so on unless the device semantic-name property is
    /// configured separately.
    pub const fn new(
        code: &'bc [u8],
        entrypoint: &'ep CStr,
        fmt: ShaderFormat,
        stage: ShaderStage,
        num_samplers: u32,
        (num_storage_textures, num_storage_buffers, num_uniform_buffers): (u32, u32, u32),
    ) -> Self {
        let inner = SDL_GPUShaderCreateInfo {
            code_size: code.len(),
            code: code.as_ptr(),
            entrypoint: entrypoint.as_ptr(),
            format: SDL_GPUShaderFormat::new(fmt as _),
            stage: SDL_GPUShaderStage::new(stage as _),
            num_samplers,
            num_storage_textures,
            num_storage_buffers,
            num_uniform_buffers,
            props: SDL_PropertiesID::new(0),
        };

        Self(inner, PhantomData, PhantomData)
    }

    /// Create vertex-shader info with no resource bindings.
    pub const fn vertex(code: &'bc [u8], entrypoint: &'ep CStr, fmt: ShaderFormat) -> Self {
        Self::new(code, entrypoint, fmt, ShaderStage::Vertex, 0, (0, 0, 0))
    }

    /// Create fragment-shader info with no resource bindings.
    pub const fn fragment(code: &'bc [u8], entrypoint: &'ep CStr, fmt: ShaderFormat) -> Self {
        Self::new(code, entrypoint, fmt, ShaderStage::Fragment, 0, (0, 0, 0))
    }
}

resource_new_no_drop!(
    /// An opaque handle representing a compiled shader object.
    SDL_GPUShader, Shader
);
impl Shader {
    /// Build a [`Shader`] with additional parameters not available in [`ShaderCreateInfo`].
    pub fn builder(props: Ref<'_, Properties>) -> ShaderBuilder<'_> {
        ShaderBuilder::new(props)
    }

    /// Create a shader for use when creating a graphics pipeline.
    ///
    /// `device` is the GPU device that owns the shader, and `create_info`
    /// describes its code, stage, format, and resource bindings. The bindings
    /// must follow the convention for the selected shader format; see
    /// [`ShaderCreateInfo::new`].
    ///
    /// Returns [`Err`] if the shader cannot be created.
    #[doc(alias = "SDL_CreateGPUShader")]
    pub fn new(device: Ref<Device>, create_info: &ShaderCreateInfo) -> Result<Self> {
        let handle =
            unsafe { SDL_CreateGPUShader(device.handle.as_ptr(), &raw const create_info.0) };

        Self::from_ptr(handle)
    }

    /// Release a shader as soon as it is safe to do so.
    ///
    /// `device` is the GPU device that owns the shader. This method consumes the
    /// shader; it must not be referenced after this call. Unlike ordinary RAII
    /// resources, a shader created with this module has no automatic destructor,
    /// so this method must be called explicitly.
    #[doc(alias = "SDL_ReleaseGPUShader")]
    pub fn drop(self, device: Ref<Device>) {
        unsafe {
            SDL_ReleaseGPUShader(device.handle.as_ptr(), self.handle.as_ptr());
        }
    }
}
