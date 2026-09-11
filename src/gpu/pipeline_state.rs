//! Types describing the state of a graphics pipeline.
//!
//! Everything in this module is a building block of
//! [`GraphicsPipelineCreateInfo`](crate::gpu::graphics_pipeline::GraphicsPipelineCreateInfo).

use std::marker::PhantomData;

use bitflags::bitflags;
use sdl3_sys::gpu::*;

use crate::{gpu::enums::*, util::impl_enum_transmute};

use super::{
    sampler::CompareOp,
    texture::{SampleCount, TextureFormat},
};

/// The format of a vertex attribute.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUVertexElementFormat")]
pub enum VertexElementFormat {
    Int = SDL_GPUVertexElementFormat::INT.0,
    Int2 = SDL_GPUVertexElementFormat::INT2.0,
    Int3 = SDL_GPUVertexElementFormat::INT3.0,
    Int4 = SDL_GPUVertexElementFormat::INT4.0,
    Uint = SDL_GPUVertexElementFormat::UINT.0,
    Uint2 = SDL_GPUVertexElementFormat::UINT2.0,
    Uint3 = SDL_GPUVertexElementFormat::UINT3.0,
    Uint4 = SDL_GPUVertexElementFormat::UINT4.0,
    Float = SDL_GPUVertexElementFormat::FLOAT.0,
    Float2 = SDL_GPUVertexElementFormat::FLOAT2.0,
    Float3 = SDL_GPUVertexElementFormat::FLOAT3.0,
    Float4 = SDL_GPUVertexElementFormat::FLOAT4.0,
    Byte2 = SDL_GPUVertexElementFormat::BYTE2.0,
    Byte4 = SDL_GPUVertexElementFormat::BYTE4.0,
    Ubyte2 = SDL_GPUVertexElementFormat::UBYTE2.0,
    Ubyte4 = SDL_GPUVertexElementFormat::UBYTE4.0,
    Byte2Norm = SDL_GPUVertexElementFormat::BYTE2_NORM.0,
    Byte4Norm = SDL_GPUVertexElementFormat::BYTE4_NORM.0,
    Ubyte2Norm = SDL_GPUVertexElementFormat::UBYTE2_NORM.0,
    Ubyte4Norm = SDL_GPUVertexElementFormat::UBYTE4_NORM.0,
    Short2 = SDL_GPUVertexElementFormat::SHORT2.0,
    Short4 = SDL_GPUVertexElementFormat::SHORT4.0,
    Ushort2 = SDL_GPUVertexElementFormat::USHORT2.0,
    Ushort4 = SDL_GPUVertexElementFormat::USHORT4.0,
    Short2Norm = SDL_GPUVertexElementFormat::SHORT2_NORM.0,
    Short4Norm = SDL_GPUVertexElementFormat::SHORT4_NORM.0,
    Ushort2Norm = SDL_GPUVertexElementFormat::USHORT2_NORM.0,
    Ushort4Norm = SDL_GPUVertexElementFormat::USHORT4_NORM.0,
    Half2 = SDL_GPUVertexElementFormat::HALF2.0,
    Half4 = SDL_GPUVertexElementFormat::HALF4.0,
}

impl_enum_transmute!(SDL_GPUVertexElementFormat, VertexElementFormat, INVALID);

/// The rate at which vertex attributes are read from buffers.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUVertexInputRate")]
pub enum VertexInputRate {
    /// Address attributes by vertex index.
    Vertex = SDL_GPUVertexInputRate::VERTEX.0,
    /// Address attributes by instance index.
    Instance = SDL_GPUVertexInputRate::INSTANCE.0,
}

/// How polygons are rasterized.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUFillMode")]
pub enum FillMode {
    /// Rasterize filled polygons.
    Fill = SDL_GPUFillMode::FILL.0,
    /// Draw polygon edges as line segments.
    Line = SDL_GPUFillMode::LINE.0,
}

/// The triangle-facing direction to cull.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUCullMode")]
pub enum CullMode {
    /// Do not cull triangles.
    None = SDL_GPUCullMode::NONE.0,
    /// Cull front-facing triangles.
    Front = SDL_GPUCullMode::FRONT.0,
    /// Cull back-facing triangles.
    Back = SDL_GPUCullMode::BACK.0,
}

/// The vertex winding treated as front-facing.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUFrontFace")]
pub enum FrontFace {
    /// Counter-clockwise vertex winding is front-facing.
    CounterClockwise = SDL_GPUFrontFace::COUNTER_CLOCKWISE.0,
    /// Clockwise vertex winding is front-facing.
    Clockwise = SDL_GPUFrontFace::CLOCKWISE.0,
}

/// A factor used when blending source pixels with destination pixels.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUBlendFactor")]
pub enum BlendFactor {
    /// Zero.
    Zero = SDL_GPUBlendFactor::ZERO.0,
    /// One.
    One = SDL_GPUBlendFactor::ONE.0,
    /// Source color.
    SrcColor = SDL_GPUBlendFactor::SRC_COLOR.0,
    /// One minus source color.
    OneMinusSrcColor = SDL_GPUBlendFactor::ONE_MINUS_SRC_COLOR.0,
    /// Destination color.
    DstColor = SDL_GPUBlendFactor::DST_COLOR.0,
    /// One minus destination color.
    OneMinusDstColor = SDL_GPUBlendFactor::ONE_MINUS_DST_COLOR.0,
    /// Source alpha.
    SrcAlpha = SDL_GPUBlendFactor::SRC_ALPHA.0,
    /// One minus source alpha.
    OneMinusSrcAlpha = SDL_GPUBlendFactor::ONE_MINUS_SRC_ALPHA.0,
    /// Destination alpha.
    DstAlpha = SDL_GPUBlendFactor::DST_ALPHA.0,
    /// One minus destination alpha.
    OneMinusDstAlpha = SDL_GPUBlendFactor::ONE_MINUS_DST_ALPHA.0,
    /// The blend constant.
    ConstantColor = SDL_GPUBlendFactor::CONSTANT_COLOR.0,
    /// One minus the blend constant.
    OneMinusConstantColor = SDL_GPUBlendFactor::ONE_MINUS_CONSTANT_COLOR.0,
    /// `min(source alpha, 1 - destination alpha)`.
    SrcAlphaSaturate = SDL_GPUBlendFactor::SRC_ALPHA_SATURATE.0,
}

/// The operation used to combine source and destination pixels.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUBlendOp")]
pub enum BlendOp {
    /// `source * source_factor + destination * destination_factor`.
    Add = SDL_GPUBlendOp::ADD.0,
    /// `source * source_factor - destination * destination_factor`.
    Subtract = SDL_GPUBlendOp::SUBTRACT.0,
    /// `destination * destination_factor - source * source_factor`.
    ReverseSubtract = SDL_GPUBlendOp::REVERSE_SUBTRACT.0,
    /// The component-wise minimum of source and destination.
    Min = SDL_GPUBlendOp::MIN.0,
    /// The component-wise maximum of source and destination.
    Max = SDL_GPUBlendOp::MAX.0,
}

impl_enum_transmute!(SDL_GPUBlendOp, BlendOp, INVALID);

/// The operation applied to a stored stencil value.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUStencilOp")]
pub enum StencilOp {
    /// Keep the current value.
    Keep = SDL_GPUStencilOp::KEEP.0,
    /// Set the value to zero.
    Zero = SDL_GPUStencilOp::ZERO.0,
    /// Set the value to the reference value.
    Replace = SDL_GPUStencilOp::REPLACE.0,
    /// Increment and clamp to the maximum value.
    IncrementAndClamp = SDL_GPUStencilOp::INCREMENT_AND_CLAMP.0,
    /// Decrement and clamp to zero.
    DecrementAndClamp = SDL_GPUStencilOp::DECREMENT_AND_CLAMP.0,
    /// Bitwise-invert the current value.
    Invert = SDL_GPUStencilOp::INVERT.0,
    /// Increment and wrap to zero.
    IncrementAndWrap = SDL_GPUStencilOp::INCREMENT_AND_WRAP.0,
    /// Decrement and wrap to the maximum value.
    DecrementAndWrap = SDL_GPUStencilOp::DECREMENT_AND_WRAP.0,
}

impl_enum_transmute!(SDL_GPUStencilOp, StencilOp, INVALID);

bitflags! {
    /// Selects the color components written by a graphics pipeline.
    #[derive(Clone, Copy)]
    #[doc(alias = "SDL_GPUColorComponentFlags")]
    pub struct ColorComponentFlags: u8 {
        /// Enable writes to the red component.
        const R = SDL_GPUColorComponentFlags::R.0;
        /// Enable writes to the green component.
        const G = SDL_GPUColorComponentFlags::G.0;
        /// Enable writes to the blue component.
        const B = SDL_GPUColorComponentFlags::B.0;
        /// Enable writes to the alpha component.
        const A = SDL_GPUColorComponentFlags::A.0;
    }
}

impl_enum_transmute!(SDL_GPUVertexInputRate, VertexInputRate);
impl_enum_transmute!(SDL_GPUFillMode, FillMode);
impl_enum_transmute!(SDL_GPUCullMode, CullMode);
impl_enum_transmute!(SDL_GPUFrontFace, FrontFace);
impl_enum_transmute!(SDL_GPUBlendFactor, BlendFactor);
impl_enum_transmute!(SDL_GPUColorComponentFlags, ColorComponentFlags);

/// Parameters for a vertex buffer used by a graphics pipeline.
#[doc(alias = "SDL_GPUVertexBufferDescription")]
#[derive(Clone, Copy)]
pub struct VertexBufferDescription(SDL_GPUVertexBufferDescription);
impl VertexBufferDescription {
    /// Describe a vertex buffer binding slot, element pitch, and input rate.
    ///
    /// `slot` is the binding slot, `pitch` is the size and stride of one vertex
    /// element, and `input_rate` selects vertex- or instance-based addressing.
    /// The reserved instance-step-rate field is set to zero.
    pub fn new(slot: u32, pitch: u32, input_rate: VertexInputRate) -> Self {
        Self(SDL_GPUVertexBufferDescription {
            slot,
            pitch,
            input_rate: SDL_GPUVertexInputRate::new(input_rate as _),
            instance_step_rate: 0, // "Reserved for future use. Must be set to 0."
        })
    }
}

/// Description of a vertex shader input attribute.
#[doc(alias = "SDL_GPUVertexAttribute")]
#[derive(Clone, Copy)]
pub struct VertexAttribute(SDL_GPUVertexAttribute);
impl VertexAttribute {
    /// Describe an attribute location, buffer slot, element format, and byte offset.
    ///
    /// `location` values must be unique within the vertex input state.
    pub fn new(location: u32, buffer_slot: u32, format: VertexElementFormat, offset: u32) -> Self {
        Self(SDL_GPUVertexAttribute {
            location,
            buffer_slot,
            format: SDL_GPUVertexElementFormat::new(format as _),
            offset,
        })
    }
}

/// Vertex-buffer and vertex-attribute descriptions for a graphics pipeline.
///
/// The descriptor and attribute slices are borrowed for `'vbd` and `'va`.
#[doc(alias = "SDL_GPUVertexInputState")]
#[derive(Clone, Copy)]
pub struct VertexInputState<'vbd, 'va>(
    pub(crate) SDL_GPUVertexInputState,
    PhantomData<&'vbd [VertexBufferDescription]>,
    PhantomData<&'va [VertexAttribute]>,
);
impl<'vbd, 'va> VertexInputState<'vbd, 'va> {
    /// Build vertex input state from buffer descriptions and attributes.
    pub fn new(
        descriptions: &'vbd [VertexBufferDescription],
        attributes: &'va [VertexAttribute],
    ) -> Self {
        VertexInputState(
            SDL_GPUVertexInputState {
                vertex_buffer_descriptions: descriptions.as_ptr().cast(),
                num_vertex_buffers: descriptions.len() as _,
                vertex_attributes: attributes.as_ptr().cast(),
                num_vertex_attributes: attributes.len() as _,
            },
            PhantomData,
            PhantomData,
        )
    }
}

/// Rasterization state for a graphics pipeline.
///
/// Line fill mode is unsupported on many Android devices and may fall back to
/// filled polygons. D3D12 enables depth clamping even when depth clipping is
/// enabled; matching clamp-and-clip behavior on Metal and Vulkan may require
/// manual fragment-shader depth clamping.
#[doc(alias = "SDL_GPURasterizerState")]
#[derive(Clone, Copy)]
pub struct RasterizerState(pub(crate) SDL_GPURasterizerState);
impl RasterizerState {
    /// Describe polygon fill, culling, front-face winding, depth bias, and
    /// depth clip behavior.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fill_mode: FillMode,
        cull_mode: CullMode,
        front_face: FrontFace,
        depth_bias_constant_factor: f32,
        depth_bias_clamp: f32,
        depth_bias_slope_factor: f32,
        db: EnableDepthBias,
        dc: EnableDepthClip,
    ) -> Self {
        Self(SDL_GPURasterizerState {
            fill_mode: SDL_GPUFillMode::new(fill_mode as _),
            cull_mode: SDL_GPUCullMode::new(cull_mode as _),
            front_face: SDL_GPUFrontFace::new(front_face as _),
            depth_bias_constant_factor,
            depth_bias_clamp,
            depth_bias_slope_factor,
            enable_depth_bias: db.into(),
            enable_depth_clip: dc.into(),
            ..Default::default()
        })
    }
}

/// Multisampling state for a graphics pipeline.
#[doc(alias = "SDL_GPUMultisampleState")]
#[derive(Clone, Copy)]
pub struct MultisampleState(pub(crate) SDL_GPUMultisampleState);
impl MultisampleState {
    /// Set the rasterization sample count and alpha-to-coverage behavior.
    /// Reserved SDL fields are initialized to their required zero/false values.
    pub fn new(sample_count: SampleCount, eatc: EnableAlphaToCoverage) -> Self {
        Self(SDL_GPUMultisampleState {
            sample_count: SDL_GPUSampleCount::new(sample_count as _),
            sample_mask: 0,     // "Reserved for future use. Must be set to 0."
            enable_mask: false, // "Reserved for future use. Must be set to false."
            enable_alpha_to_coverage: eatc.into(),
            ..Default::default()
        })
    }
}

/// Stencil operation state for a graphics pipeline.
#[doc(alias = "SDL_GPUStencilOpState")]
#[derive(Clone, Copy)]
pub struct StencilOpState(SDL_GPUStencilOpState);
impl StencilOpState {
    /// Describe the operations for stencil failure, depth/stencil success, and
    /// depth failure, along with the stencil comparison operation.
    pub fn new(
        fail_op: StencilOp,
        pass_op: StencilOp,
        depth_fail_op: StencilOp,
        compare_op: CompareOp,
    ) -> Self {
        Self(SDL_GPUStencilOpState {
            fail_op: SDL_GPUStencilOp::new(fail_op as _),
            pass_op: SDL_GPUStencilOp::new(pass_op as _),
            depth_fail_op: SDL_GPUStencilOp::new(depth_fail_op as _),
            compare_op: SDL_GPUCompareOp::new(compare_op as _),
        })
    }
}

/// Depth and stencil state for a graphics pipeline.
///
/// Depth writes are disabled automatically when depth testing is disabled.
#[doc(alias = "SDL_GPUDepthStencilState")]
#[derive(Clone, Copy)]
pub struct DepthStencilState(pub(crate) SDL_GPUDepthStencilState);
impl DepthStencilState {
    /// Describe depth comparison, front/back stencil operations, masks, and
    /// depth/stencil enable flags.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        compare_op: CompareOp,
        back_stencil_state: StencilOpState,
        front_stencil_state: StencilOpState,
        compare_mask: u8,
        write_mask: u8,
        edt: EnableDepthTest,
        edw: EnableDepthWrite,
        est: EnableStencilTest,
    ) -> Self {
        Self(SDL_GPUDepthStencilState {
            compare_op: SDL_GPUCompareOp::new(compare_op as _),
            back_stencil_state: back_stencil_state.0,
            front_stencil_state: front_stencil_state.0,
            compare_mask,
            write_mask,
            enable_depth_test: edt.into(),
            enable_depth_write: edw.into(),
            enable_stencil_test: est.into(),
            ..Default::default()
        })
    }
}

/// Blend state for a color target.
#[doc(alias = "SDL_GPUColorTargetBlendState")]
#[derive(Clone, Copy)]
pub struct ColorTargetBlendState(SDL_GPUColorTargetBlendState);
impl ColorTargetBlendState {
    /// Describe source/destination blend factors and operations for RGB and
    /// alpha, plus the color write mask and enable flags.
    #[allow(deprecated, clippy::too_many_arguments)]
    pub fn new(
        (src_color_bf, dst_color_bf): (BlendFactor, BlendFactor),
        color_blend_op: BlendOp,
        (src_alpha_bf, dst_alpha_bf): (BlendFactor, BlendFactor),
        alpha_blend_op: BlendOp,
        color_write_mask: ColorComponentFlags,
        eb: EnableBlend,
        ecwm: EnableColorWriteMask,
    ) -> Self {
        Self(SDL_GPUColorTargetBlendState {
            src_color_blendfactor: SDL_GPUBlendFactor::new(src_color_bf as _),
            dst_color_blendfactor: SDL_GPUBlendFactor::new(dst_color_bf as _),
            color_blend_op: SDL_GPUBlendOp::new(color_blend_op as _),
            src_alpha_blendfactor: SDL_GPUBlendFactor::new(src_alpha_bf as _),
            dst_alpha_blendfactor: SDL_GPUBlendFactor::new(dst_alpha_bf as _),
            alpha_blend_op: SDL_GPUBlendOp::new(alpha_blend_op as _),
            color_write_mask: SDL_GPUColorComponentFlags::new(color_write_mask.bits()),
            enable_blend: eb.into(),
            enable_color_write_mask: ecwm.into(),
            ..Default::default()
        })
    }
}

/// Parameters for a color target used by a graphics pipeline.
#[doc(alias = "SDL_GPUColorTargetDescription")]
#[derive(Clone, Copy)]
pub struct ColorTargetDescription(SDL_GPUColorTargetDescription);
impl ColorTargetDescription {
    /// Describe the target texture format and blend state.
    pub fn new(format: TextureFormat, blend_state: ColorTargetBlendState) -> Self {
        Self(SDL_GPUColorTargetDescription {
            format: SDL_GPUTextureFormat::new(format as _),
            blend_state: blend_state.0,
        })
    }
}

/// Render-target descriptions used by a graphics pipeline.
///
/// The color-target description slice is borrowed for `'ctd`.
#[doc(alias = "SDL_GPUGraphicsPipelineTargetInfo")]
#[derive(Clone, Copy)]
pub struct GraphicsPipelineTargetInfo<'ctd>(
    pub(crate) SDL_GPUGraphicsPipelineTargetInfo,
    PhantomData<&'ctd [ColorTargetDescription]>,
);

impl<'ctd> GraphicsPipelineTargetInfo<'ctd> {
    /// Describe color targets and the optional depth-stencil target.
    pub fn new(
        descriptions: &'ctd [ColorTargetDescription],
        depth_stencil_format: Option<TextureFormat>,
    ) -> Self {
        let (depth_stencil_format, has_depth_stencil_target) = match depth_stencil_format {
            Some(dsf) => (dsf.to_sdl(), true),
            None => (SDL_GPUTextureFormat::default(), false),
        };

        Self(
            SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: descriptions.as_ptr().cast(),
                num_color_targets: descriptions.len() as _,
                depth_stencil_format,
                has_depth_stencil_target,
                ..Default::default()
            },
            PhantomData,
        )
    }
}
