//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_BeginGPURenderPass
//! - [x] SDL_EndGPURenderPass
//! - [x] SDL_BindGPUFragmentSamplers
//! - [x] SDL_BindGPUFragmentStorageBuffers
//! - [x] SDL_BindGPUFragmentStorageTextures
//! - [x] SDL_BindGPUIndexBuffer
//! - [x] SDL_BindGPUVertexBuffers
//! - [x] SDL_BindGPUVertexSamplers
//! - [x] SDL_BindGPUVertexStorageBuffers
//! - [x] SDL_BindGPUVertexStorageTextures
//! - [x] SDL_DrawGPUIndexedPrimitives
//! - [x] SDL_DrawGPUIndexedPrimitivesIndirect
//! - [x] SDL_DrawGPUPrimitives
//! - [x] SDL_DrawGPUPrimitivesIndirect
//! - [x] SDL_SetGPUBlendConstants
//! - [x] SDL_SetGPUScissor
//! - [x] SDL_SetGPUStencilReference
//! - [x] SDL_SetGPUViewport

use std::marker::PhantomData;

use sdl3_sys::gpu::*;

use crate::{
    Result,
    color::RgbaF32,
    gpu::{Buffer, Cycle, CycleResolveTexture},
    rect::{PointF32, RectI32},
    resource::{Ref, Resource},
    util::impl_enum_transmute,
    util::opt2ptr,
    util::resource_new,
};

use super::{
    buffer::BufferBinding,
    command_buffer::CommandBuffer,
    texture::{Texture, TextureSamplerBinding},
};

/// The size of elements in an index buffer.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUIndexElementSize")]
pub enum IndexElementSize {
    /// 16-bit index elements.
    Bits16 = SDL_GPUIndexElementSize::_16BIT.0,
    /// 32-bit index elements.
    Bits32 = SDL_GPUIndexElementSize::_32BIT.0,
}

/// How an attached texture is treated at the start of a render pass.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPULoadOp")]
pub enum LoadOp {
    /// Preserve the texture's previous contents.
    Load = SDL_GPULoadOp::LOAD.0,
    /// Clear the texture to its configured clear value.
    Clear = SDL_GPULoadOp::CLEAR.0,
    /// The previous contents need not be preserved and become undefined.
    DontCare = SDL_GPULoadOp::DONT_CARE.0,
}

/// How an attached texture is treated at the end of a render pass.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUStoreOp")]
pub enum StoreOp {
    /// Write render-pass contents to memory.
    Store = SDL_GPUStoreOp::STORE.0,
    /// The generated contents may be discarded and become undefined.
    DontCare = SDL_GPUStoreOp::DONT_CARE.0,
    /// Resolve multisample contents to a single-sample texture; the source
    /// multisample contents may then be discarded.
    Resolve = SDL_GPUStoreOp::RESOLVE.0,
    /// Resolve multisample contents and also store the multisample texture.
    ResolveAndStore = SDL_GPUStoreOp::RESOLVE_AND_STORE.0,
}

impl_enum_transmute!(SDL_GPUIndexElementSize, IndexElementSize);
impl_enum_transmute!(SDL_GPULoadOp, LoadOp);
impl_enum_transmute!(SDL_GPUStoreOp, StoreOp);

/// A viewport used by a render pass.
#[doc(alias = "SDL_GPUViewport")]
#[derive(Clone, Copy)]
pub struct Viewport(SDL_GPUViewport);
impl Viewport {
    /// Describe the viewport position, size, and depth range.
    pub fn new(pos: PointF32, size: PointF32, (min_depth, max_depth): (f32, f32)) -> Self {
        Self(SDL_GPUViewport {
            x: pos.x,
            y: pos.y,
            w: size.x,
            h: size.y,
            min_depth,
            max_depth,
        })
    }
}

/// Parameters for a color target used by a render pass.
///
/// The target and optional resolve texture are borrowed for `'t` and `'rt`.
/// Resolve operations require a single-sample resolve texture. Load/store
/// operations determine whether contents are loaded, cleared, discarded, stored,
/// or resolved; cycling applies when a bound texture is written.
#[doc(alias = "SDL_GPUColorTargetInfo")]
#[derive(Clone, Copy)]
pub struct ColorTargetInfo<'t, 'rt>(
    SDL_GPUColorTargetInfo,
    PhantomData<Ref<'t, Texture>>,
    PhantomData<Ref<'rt, Texture>>,
);

impl<'t, 'rt> ColorTargetInfo<'t, 'rt> {
    /// Describe a color target, its clear/load/store behavior, optional resolve
    /// target, and cycling options.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tex: Ref<'t, Texture>,
        mip_level: u32,
        layer_or_depth_plane: u32,
        clear_color: RgbaF32,
        load_op: LoadOp,
        store_op: StoreOp,
        resolve_texture: Option<Ref<'rt, Texture>>,
        (resolve_mip_level, resolve_layer): (u32, u32),
        cycle: Cycle,
        crt: CycleResolveTexture,
    ) -> Self {
        let resolve_texture = resolve_texture.map_or(std::ptr::null_mut(), |t| t.handle.as_ptr());
        Self(
            SDL_GPUColorTargetInfo {
                texture: tex.handle.as_ptr(),
                mip_level,
                layer_or_depth_plane,
                clear_color: clear_color.into(),
                load_op: SDL_GPULoadOp::new(load_op as _),
                store_op: SDL_GPUStoreOp::new(store_op as _),
                resolve_texture,
                resolve_mip_level,
                resolve_layer,
                cycle: cycle.into(),
                cycle_resolve_texture: crt.into(),
                ..Default::default()
            },
            PhantomData,
            PhantomData,
        )
    }
}

/// Parameters for a depth-stencil target used by a render pass.
///
/// The target texture is borrowed for `'t`. Depth and stencil have independent
/// load/store operations and clear values. Depth/stencil targets do not support
/// multisample resolves, and layers above 255 are not supported by SDL's ABI.
#[doc(alias = "SDL_GPUDepthStencilTargetInfo")]
#[derive(Clone, Copy)]
pub struct DepthStencilTargetInfo<'t>(SDL_GPUDepthStencilTargetInfo, PhantomData<Ref<'t, Texture>>);
impl<'t> DepthStencilTargetInfo<'t> {
    /// Describe depth/stencil clear, load/store, cycling, mip-level, and layer
    /// behavior.
    pub fn new(
        tex: Ref<'t, Texture>,
        clear_depth: f32,
        (load_op, store_op): (LoadOp, StoreOp),
        (stencil_load_op, stencil_store_op): (LoadOp, StoreOp),
        cycle: Cycle,
        clear_stencil: u8,
        (mip_level, layer): (u8, u8),
    ) -> Self {
        let texture = tex.handle.as_ptr();
        Self(
            SDL_GPUDepthStencilTargetInfo {
                texture,
                clear_depth,
                load_op: SDL_GPULoadOp::new(load_op as _),
                store_op: SDL_GPUStoreOp::new(store_op as _),
                stencil_load_op: SDL_GPULoadOp::new(stencil_load_op as _),
                stencil_store_op: SDL_GPUStoreOp::new(stencil_store_op as _),
                cycle: cycle.into(),
                clear_stencil,
                mip_level,
                layer,
            },
            PhantomData,
        )
    }
}

/// Parameters of an indirect draw command.
///
/// Commands of this type are read by
/// [`RenderPassHandle::draw_primitives_indirect`] from a buffer at the given
/// byte offset, so they must be written there with a matching layout.
///
/// # Remarks
///
/// Note that the `first_vertex` and `first_instance` parameters are NOT
/// compatible with built-in vertex/instance ID variables in shaders (for
/// example, `SV_VertexID`); GPU APIs and shader languages do not define these
/// built-in variables consistently, so if your shader depends on them, the
/// only way to keep behavior consistent and portable is to always pass 0 for
/// the correlating parameter in the draw calls.
#[doc(alias = "SDL_GPUIndirectDrawCommand")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndirectDrawCommand(SDL_GPUIndirectDrawCommand);

impl IndirectDrawCommand {
    /// Describe the vertices and instances to draw.
    pub const fn new(
        (vertex_count, instance_count): (u32, u32),
        (first_vertex, first_instance): (u32, u32),
    ) -> Self {
        Self(SDL_GPUIndirectDrawCommand {
            num_vertices: vertex_count,
            num_instances: instance_count,
            first_vertex,
            first_instance,
        })
    }

    /// The number of vertices and instances to draw.
    pub const fn counts(&self) -> (u32, u32) {
        (self.0.num_vertices, self.0.num_instances)
    }

    /// The index of the first vertex and the ID of the first instance to
    /// draw.
    pub const fn firsts(&self) -> (u32, u32) {
        (self.0.first_vertex, self.0.first_instance)
    }
}

resource_new!(
    /// An opaque handle representing a render pass.
    /// Transient; invalid once the pass ends.
    SDL_GPURenderPass, RenderPass, SDL_EndGPURenderPass
);

/// Parameters of an indirect indexed draw command.
///
/// Commands of this type are read by
/// [`RenderPassHandle::draw_indexed_primitives_indirect`] from a buffer at
/// the given byte offset, so they must be written there with a matching
/// layout.
///
/// Note that, as with [`IndirectDrawCommand`], the `first_index`/
/// `first_instance` parameters are NOT compatible with built-in
/// vertex/instance ID variables in shaders; pass 0 for portable behavior.
// FIXME(doc): not documented on the wiki; the header calls it
// SDL_GPUIndexedIndirectDrawCommand as of SDL 3.4 (formerly
// SDL_GPUIndexedIndirectCommand).
#[doc(alias = "SDL_GPUIndexedIndirectDrawCommand")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IndirectIndexedCommand(SDL_GPUIndexedIndirectDrawCommand);

impl IndirectIndexedCommand {
    /// Describe the indexed draws and instances to draw.
    pub const fn new(
        (num_indices, num_instances): (u32, u32),
        (first_index, vertex_offset): (u32, i32),
        first_instance: u32,
    ) -> Self {
        Self(SDL_GPUIndexedIndirectDrawCommand {
            num_indices,
            num_instances,
            first_index,
            vertex_offset,
            first_instance,
        })
    }

    /// The number of indices and instances to draw.
    pub const fn counts(&self) -> (u32, u32) {
        (self.0.num_indices, self.0.num_instances)
    }

    /// The index of the first index to draw, and the offset of the first
    /// vertex to draw.
    pub const fn firsts(&self) -> (u32, i32) {
        (self.0.first_index, self.0.vertex_offset)
    }

    /// The ID of the first instance to draw.
    pub const fn first_instance(&self) -> u32 {
        self.0.first_instance
    }
}
impl RenderPass {
    /// Begin a render pass on a command buffer.
    ///
    /// `color_targets` describes the color subresources and their load/store
    /// operations. `depth_stencil_target` is optional. Graphics operations must
    /// occur inside a render pass, and no other render, compute, or copy pass may
    /// begin until this pass ends. SDL initializes default viewport and scissor
    /// state when the pass begins.
    ///
    /// Loading a subresource before it has been written has undefined behavior.
    ///
    /// Returns [`Err`] if SDL cannot begin the pass.
    #[doc(alias = "SDL_BeginGPURenderPass")]
    pub fn new(
        cmdbuf: Ref<CommandBuffer>,
        color_targets: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
    ) -> Result<Self> {
        let handle = unsafe {
            SDL_BeginGPURenderPass(
                cmdbuf.handle.as_ptr(),
                color_targets.as_ptr().cast(),
                color_targets.len() as _,
                opt2ptr(depth_stencil_target).cast(),
            )
        };

        Self::from_ptr(handle)
    }

    /// Convenience function that creates a [`RenderPass`], does some work on it,
    /// then ends (drops) it.
    ///
    /// Propagates [`Err`] returned by:
    /// - [`RenderPass::new`]
    /// - `op`
    pub fn run<F: FnOnce(Ref<Self>) -> Result<()>>(
        cmdbuf: Ref<CommandBuffer>,
        color_targets: &[ColorTargetInfo],
        depth_stencil_target: Option<&DepthStencilTargetInfo>,
        op: F,
    ) -> Result<()> {
        let pass = Self::new(cmdbuf, color_targets, depth_stencil_target)?;
        op(pass.as_ref())
    }
}

impl RenderPassHandle {
    /// Set the current viewport state.
    #[doc(alias = "SDL_SetGPUViewport")]
    pub fn set_viewport(&self, viewport: &Viewport) {
        unsafe {
            SDL_SetGPUViewport(self.handle.as_ptr(), &raw const viewport.0);
        }
    }

    /// Set the current blend-constant color.
    #[doc(alias = "SDL_SetGPUBlendConstants")]
    pub fn set_blend_constants(&self, blend_constants: RgbaF32) {
        unsafe {
            SDL_SetGPUBlendConstants(self.handle.as_ptr(), blend_constants.into());
        }
    }

    /// Set the current stencil reference value.
    #[doc(alias = "SDL_SetGPUStencilReference")]
    pub fn set_stencil_reference(&self, reference: u8) {
        unsafe {
            SDL_SetGPUStencilReference(self.handle.as_ptr(), reference);
        }
    }

    /// Set the current scissor area.
    #[doc(alias = "SDL_SetGPUScissor")]
    pub fn set_scissor(&self, scissor: &RectI32) {
        unsafe {
            SDL_SetGPUScissor(self.handle.as_ptr(), scissor.as_sdl_ptr());
        }
    }

    /// Bind vertex buffers to consecutive slots beginning at `first_slot`.
    /// `bindings` supplies each buffer and byte offset for subsequent draw calls.
    #[doc(alias = "SDL_BindGPUVertexBuffers")]
    pub fn bind_vertex_buffers(&self, first_slot: u32, bindings: &[BufferBinding]) {
        unsafe {
            SDL_BindGPUVertexBuffers(
                self.handle.as_ptr(),
                first_slot,
                bindings.as_ptr().cast(),
                bindings.len() as _,
            );
        }
    }

    /// Bind an index buffer and specify its element size.
    #[doc(alias = "SDL_BindGPUIndexBuffer")]
    pub fn bind_index_buffer(&self, binding: &BufferBinding, index_element_size: IndexElementSize) {
        unsafe {
            SDL_BindGPUIndexBuffer(
                self.handle.as_ptr(),
                &raw const binding.0,
                SDL_GPUIndexElementSize::new(index_element_size as _),
            );
        }
    }

    /// Bind texture-sampler pairs to consecutive vertex-shader slots.
    /// Textures must have sampler usage enabled.
    #[doc(alias = "SDL_BindGPUVertexSamplers")]
    pub fn bind_vertex_samplers(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUVertexSamplers(
                self.handle.as_ptr(),
                first_slot,
                bindings.as_ptr().cast(),
                bindings.len() as _,
            );
        }
    }

    /// Bind graphics-storage textures to consecutive vertex-shader slots.
    /// Textures must have graphics storage-read usage enabled.
    #[doc(alias = "SDL_BindGPUVertexStorageTextures")]
    pub fn bind_vertex_storage_textures(&self, first_slot: u32, textures: &[Ref<Texture>]) {
        unsafe {
            SDL_BindGPUVertexStorageTextures(
                self.handle.as_ptr(),
                first_slot,
                textures.as_ptr().cast(),
                textures.len() as _,
            );
        }
    }

    /// Bind graphics-storage buffers to consecutive vertex-shader slots.
    /// Buffers must have graphics storage-read usage enabled.
    #[doc(alias = "SDL_BindGPUVertexStorageBuffers")]
    pub fn bind_vertex_storage_buffers(&self, first_slot: u32, buffers: &[Ref<Buffer>]) {
        unsafe {
            SDL_BindGPUVertexStorageBuffers(
                self.handle.as_ptr(),
                first_slot,
                buffers.as_ptr().cast(),
                buffers.len() as _,
            );
        }
    }

    /// Bind texture-sampler pairs to consecutive fragment-shader slots.
    /// Textures must have sampler usage enabled.
    #[doc(alias = "SDL_BindGPUFragmentSamplers")]
    pub fn bind_fragment_samplers(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUFragmentSamplers(
                self.handle.as_ptr(),
                first_slot,
                bindings.as_ptr().cast(),
                bindings.len() as _,
            );
        }
    }

    /// Bind graphics-storage textures to consecutive fragment-shader slots.
    /// Textures must have graphics storage-read usage enabled.
    #[doc(alias = "SDL_BindGPUFragmentStorageTextures")]
    pub fn bind_fragment_storage_textures(&self, first_slot: u32, textures: &[Ref<Texture>]) {
        unsafe {
            SDL_BindGPUFragmentStorageTextures(
                self.handle.as_ptr(),
                first_slot,
                textures.as_ptr().cast(),
                textures.len() as _,
            );
        }
    }

    /// Bind graphics-storage buffers to consecutive fragment-shader slots.
    /// Buffers must have graphics storage-read usage enabled.
    #[doc(alias = "SDL_BindGPUFragmentStorageBuffers")]
    pub fn bind_fragment_storage_buffers(&self, first_slot: u32, buffers: &[Ref<Buffer>]) {
        unsafe {
            SDL_BindGPUFragmentStorageBuffers(
                self.handle.as_ptr(),
                first_slot,
                buffers.as_ptr().cast(),
                buffers.len() as _,
            );
        }
    }

    /// Draw non-indexed primitives.
    ///
    /// The arguments are vertex count, instance count, first vertex, and first
    /// instance. A graphics pipeline must be bound first.
    #[doc(alias = "SDL_DrawGPUPrimitives")]
    pub fn draw_primitives(&self, n_verts: u32, n_insts: u32, first_vert: u32, first_inst: u32) {
        unsafe {
            SDL_DrawGPUPrimitives(
                self.handle.as_ptr(),
                n_verts,
                n_insts,
                first_vert,
                first_inst,
            );
        }
    }

    /// Draw non-indexed primitives using tightly packed
    /// [`IndirectDrawCommand`] parameters from `buffer`, beginning at
    /// `offset`, reading `draw_count` parameter sets.
    /// A graphics pipeline must be bound first.
    #[doc(alias = "SDL_DrawGPUPrimitivesIndirect")]
    pub fn draw_primitives_indirect(&self, buffer: Ref<Buffer>, offset: u32, draw_count: u32) {
        unsafe {
            SDL_DrawGPUPrimitivesIndirect(
                self.handle.as_ptr(),
                buffer.handle.as_ptr(),
                offset,
                draw_count,
            );
        }
    }

    /// Draw indexed primitives with instancing.
    ///
    /// The arguments are indices per instance, instance count, first index,
    /// vertex offset, and first instance. A graphics pipeline must be bound first.
    #[doc(alias = "SDL_DrawGPUIndexedPrimitives")]
    pub fn draw_indexed_primitives(
        &self,
        num_indices: u32,
        num_instances: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitives(
                self.handle.as_ptr(),
                num_indices,
                num_instances,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    /// Draw indexed primitives using tightly packed
    /// [`IndirectIndexedCommand`] parameters from `buffer`, beginning at
    /// `offset`, reading `draw_count` parameter sets.
    /// A graphics pipeline and index buffer must be bound first.
    #[doc(alias = "SDL_DrawGPUIndexedPrimitivesIndirect")]
    pub fn draw_indexed_primitives_indirect(
        &self,
        buffer: Ref<Buffer>,
        offset: u32,
        draw_count: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitivesIndirect(
                self.handle.as_ptr(),
                buffer.handle.as_ptr(),
                offset,
                draw_count,
            );
        }
    }
}
