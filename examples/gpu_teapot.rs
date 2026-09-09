//! Renders the Utah teapot (`models/teapot.obj`) with the GPU API.

#![windows_subsystem = "windows"]

use std::mem::ManuallyDrop;

use sandlot::{
    Context, Result,
    color::{RgbaF32, RgbaU8},
    event::Event,
    gpu::*,
    properties::Properties,
    rect::Point,
    resource::{Ref, Resource},
    subsystem::Video,
    window::Window,
};

cfg_select! {
    target_os = "macos" => {
        const VS_CODE: &[u8] = include_bytes!("shaders/teapot.metallib");
        const FS_CODE: &[u8] = VS_CODE;
        const SHADER_FMT: ShaderFormat = ShaderFormat::Metallib;
    },
    target_os = "windows" => {
        const VS_CODE: &[u8] = include_bytes!("shaders/teapot_vs.dxil");
        const FS_CODE: &[u8] = include_bytes!("shaders/teapot_fs.dxil");
        const SHADER_FMT: ShaderFormat = ShaderFormat::Dxil;
    }
    target_os = "linux" => {
        const VS_CODE: &[u8] = include_bytes!("shaders/teapot_vs.spv");
        const FS_CODE: &[u8] = include_bytes!("shaders/teapot_fs.spv");
        const SHADER_FMT: ShaderFormat = ShaderFormat::SpirV;
    }
}

const OBJ: &str = include_str!("models/teapot.obj");

/// Minimal column-major 4x4 matrix, matching Metal's `float4x4` layout.
#[derive(Clone, Copy)]
struct Mat4([f32; 16]);

impl Mat4 {
    const fn identity() -> Mat4 {
        Mat4([
            1.0, 0.0, 0.0, 0.0, // comments
            0.0, 1.0, 0.0, 0.0, // included
            0.0, 0.0, 1.0, 0.0, // for
            0.0, 0.0, 0.0, 1.0, // formatting
        ])
    }

    fn mul(&self, rhs: &Mat4) -> Mat4 {
        let mut out = [0.0f32; 16];
        for c in 0..4 {
            for r in 0..4 {
                out[c * 4 + r] = (0..4).map(|k| self.0[k * 4 + r] * rhs.0[c * 4 + k]).sum();
            }
        }
        Mat4(out)
    }

    /// Column-major perspective projection (glFrustum convention).
    fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fovy_radians / 2.0).tan();
        Mat4([
            f / aspect,
            0.0,
            0.0,
            0.0, //
            0.0,
            f,
            0.0,
            0.0, //
            0.0,
            0.0,
            (far + near) / (near - far),
            -1.0, //
            0.0,
            0.0,
            2.0 * far * near / (near - far),
            0.0, //
        ])
    }

    /// Column-major rotation about the Y axis.
    fn rot_y(angle_radians: f32) -> Mat4 {
        let (s, c) = angle_radians.sin_cos();
        Mat4([
            c, 0.0, -s, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            s, 0.0, c, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ])
    }

    /// Column-major translation.
    const fn translate(x: f32, y: f32, z: f32) -> Mat4 {
        let mut m = Mat4::identity();
        m.0[12] = x;
        m.0[13] = y;
        m.0[14] = z;
        m
    }

    fn to_bytes(self) -> [u8; size_of::<Self>()] {
        let mut bytes = [0u8; _];
        for (i, f) in self.0.iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&f.to_ne_bytes());
        }

        bytes
    }
}

/// Interleaved positions and (computed) normals, plus triangle indices.
/// `center` is the bounding-box midpoint, used to center the model on the
/// origin before rendering.
struct MeshData {
    vertices: Vec<f32>,
    indices: Vec<u16>,
    center: [f32; 3],
}

fn load_teapot() -> MeshData {
    let mut reader = std::io::Cursor::new(OBJ.as_bytes());
    let (models, _) = tobj::load_obj_buf(&mut reader, &tobj::GPU_LOAD_OPTIONS, |_| {
        Ok((Vec::new(), Default::default()))
    })
    .unwrap_or_else(|e| panic!("failed to parse embedded teapot.obj: {e}"));
    let mesh = &models[0].mesh;

    // The teapot's normals are baked into the OBJ (one `vn` per vertex, with
    // `v//vn` faces), so with `single_index` they align with `positions` and
    // need no runtime computation.
    let vertex_count = mesh.positions.len() / 3;

    assert_eq!(
        mesh.positions.len(),
        vertex_count * 3,
        "teapot positions are not a multiple of three"
    );
    assert_eq!(
        mesh.normals.len(),
        vertex_count * 3,
        "teapot normals do not match the position count"
    );
    assert!(
        vertex_count <= usize::from(u16::MAX) + 1,
        "teapot has too many vertices for 16-bit indices"
    );

    let mut vertices = Vec::with_capacity(vertex_count * 6);
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for (position, normal) in mesh
        .positions
        .chunks_exact(3)
        .zip(mesh.normals.chunks_exact(3))
    {
        vertices.extend_from_slice(position);
        vertices.extend_from_slice(normal);

        for (axis, &value) in position.iter().enumerate() {
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }

    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];

    MeshData {
        vertices,
        indices: mesh.indices.iter().map(|&i| i as u16).collect(),
        center,
    }
}

fn can_use_format(device: Ref<Device>, fmt: TextureFormat) -> bool {
    device.texture_supports_format(
        fmt,
        TextureType::_2d,
        TextureUsageFlags::DEPTH_STENCIL_TARGET,
    )
}

/// Pick a depth format the device supports. D24_UNORM is not available on all
/// backends (e.g. the Metal backend only offers D16_UNORM and D32_FLOAT).
fn pick_depth_format(device: Ref<Device>) -> TextureFormat {
    [TextureFormat::D16Unorm, TextureFormat::D24Unorm]
        .iter()
        .copied()
        .find(|&f| can_use_format(device, f))
        .unwrap_or(TextureFormat::D32Float)
}

fn run() -> Result<()> {
    let ctx = Context::new();
    let _video = ManuallyDrop::new(Video::new(&ctx)?);

    // SDL provides an existing property set, which we can conveniently abuse.
    let props = Properties::global()?;

    let device = Device::builder(props)
        .debug_mode(false)
        .shaders_metallib(true)
        .shaders_dxil(true)
        .shaders_spirv(true)
        .build_cleanup()?;

    sandlot::log!("Driver = {}", device.driver().unwrap_or("[unknown]"));

    let wnd = Window::builder(props)
        .title(c"sandlot Teapot Example")
        .size(Point::new(1280, 720))
        .build_cleanup()?;

    device.claim_window(wnd.as_ref())?;
    device.set_swapchain_parameters(wnd.as_ref(), SwapchainComposition::Sdr, PresentMode::Vsync)?;

    let mesh = load_teapot();
    sandlot::log!(
        "Teapot: {} verts, {} tris",
        mesh.vertices.len() / 6,
        mesh.indices.len() / 3
    );

    let vs = Shader::new(
        device.as_ref(),
        &ShaderCreateInfo::new(
            VS_CODE,
            c"vs_main",
            SHADER_FMT,
            ShaderStage::Vertex,
            0,
            // The counts must match the shader's resource declarations; the
            // vertex shader uses one uniform buffer (slot 0).
            (0, 0, 1),
        ),
    )?;

    let fs = Shader::new(
        device.as_ref(),
        &ShaderCreateInfo::fragment(FS_CODE, c"fs_main", SHADER_FMT),
    )?;

    // Interleaved [pos3, normal3], 24 bytes per vertex.
    let vert_bytes = (mesh.vertices.len() * 4) as u32;
    let idx_bytes = (mesh.indices.len() * 2) as u32;

    let vb = Buffer::new(
        device.as_ref(),
        &BufferCreateInfo::new(BufferUsageFlags::VERTEX, vert_bytes),
    )?;

    let ib = Buffer::new(
        device.as_ref(),
        &BufferCreateInfo::new(BufferUsageFlags::INDEX, idx_bytes),
    )?;

    let tbci = TransferBufferCreateInfo::new(TransferBufferUsage::Upload, vert_bytes + idx_bytes);
    let tb = TransferBuffer::new_with(device.as_ref(), &tbci, Cycle::No, |dst| {
        let vert_slice = unsafe {
            std::slice::from_raw_parts(mesh.vertices.as_ptr().cast::<u8>(), vert_bytes as usize)
        };

        let idx_slice = unsafe {
            std::slice::from_raw_parts(mesh.indices.as_ptr().cast::<u8>(), idx_bytes as usize)
        };

        let (vert, idx) = dst.split_at_mut(vert_bytes as _);

        vert.copy_from_slice(vert_slice);
        idx.copy_from_slice(idx_slice);
    })?;

    CommandBuffer::run(device.as_ref(), |cmdbuf| {
        CopyPass::run(cmdbuf, |copy_pass| {
            vb.upload(
                copy_pass,
                &TransferBufferLocation::whole(tb.as_ref()),
                &BufferRegion::whole(vb.as_ref(), vert_bytes),
                Cycle::No,
            );

            ib.upload(
                copy_pass,
                &TransferBufferLocation::new(tb.as_ref(), vert_bytes),
                &BufferRegion::whole(ib.as_ref(), idx_bytes),
                Cycle::No,
            );

            Ok(())
        })
    })?;

    // The vertex layout: slot 0, 24-byte stride, position + normal.
    let vbd = [VertexBufferDescription::new(0, 24, VertexInputRate::Vertex)];
    let attrs = [
        VertexAttribute::new(0, 0, VertexElementFormat::Float3, 0),
        VertexAttribute::new(1, 0, VertexElementFormat::Float3, 12),
    ];

    let stencil = StencilOpState::new(
        StencilOp::Keep,
        StencilOp::Keep,
        StencilOp::Keep,
        CompareOp::Always,
    );

    let blend = ColorTargetBlendState::new(
        (BlendFactor::One, BlendFactor::Zero),
        BlendOp::Add,
        (BlendFactor::One, BlendFactor::Zero),
        BlendOp::Add,
        ColorComponentFlags::all(), // RGBA
        EnableBlend::No,
        EnableColorWriteMask::No,
    );

    // The pipeline's color target format must match the swapchain's.
    let swapchain_format = device.swapchain_texture_format(wnd.as_ref());
    let ctd = [ColorTargetDescription::new(swapchain_format, blend)];

    // The depth texture and the pipeline must use the same, device-supported format.
    let depth_format = pick_depth_format(device.as_ref());
    sandlot::log!(
        "Depth format: {}",
        match depth_format {
            TextureFormat::D16Unorm => "D16Unorm",
            TextureFormat::D24Unorm => "D24Unorm",
            TextureFormat::D32Float => "D32Float",
            _ => "other",
        }
    );

    let pipeline = {
        let create_info = GraphicsPipelineCreateInfo::new(
            vs.as_ref(),
            fs.as_ref(),
            VertexInputState::new(&vbd, &attrs),
            PrimitiveType::TriangleList,
            RasterizerState::new(
                FillMode::Fill,
                CullMode::Back,
                FrontFace::CounterClockwise,
                0.0,
                0.0,
                0.0,
                EnableDepthBias::No,
                EnableDepthClip::Yes,
            ),
            MultisampleState::new(SampleCount::One, EnableAlphaToCoverage::No),
            DepthStencilState::new(
                CompareOp::Less,
                stencil,
                stencil,
                0xFF,
                0xFF,
                EnableDepthTest::Yes,
                EnableDepthWrite::Yes,
                EnableStencilTest::No,
            ),
            GraphicsPipelineTargetInfo::new(&ctd, Some(depth_format)),
        );

        GraphicsPipeline::new(device.as_ref(), &create_info)?
    };

    // Assume the swapchain texture's dims are equal to the window's.
    let depth = {
        let tci = TextureCreateInfo::new(
            TextureType::_2d,
            depth_format,
            TextureUsageFlags::DEPTH_STENCIL_TARGET,
            wnd.size().map(i32::cast_unsigned),
            1, // layer count / depth
            1, // mip levels
            SampleCount::One,
        );

        Texture::builder(props)
            .name(c"Teapot Texture")
            .build_cleanup(device.as_ref(), tci)?
    };

    let mut angle = 0.0f32;

    let col = RgbaF32::from(RgbaU8::rgb_hex(0x6f32a8));

    let trans = Mat4::translate(-mesh.center[0], -mesh.center[1], -mesh.center[2]);

    'frames: loop {
        for event in Event::iter() {
            if let Event::Quit = event {
                break 'frames;
            }
        }

        CommandBuffer::run(device.as_ref(), |cmdbuf| {
            let (mut width, mut height) = (0u32, 0u32);
            if let Some(tex) = cmdbuf
                .wait_for_swapchain_texture(wnd.as_ref(), (Some(&mut width), Some(&mut height)))?
            {
                let color_target = ColorTargetInfo::new(
                    tex,
                    0,
                    0,
                    col,
                    LoadOp::Clear,
                    StoreOp::Store,
                    None,
                    (0, 0),
                    Cycle::No,
                    CycleResolveTexture::No,
                );
                let depth_target = DepthStencilTargetInfo::new(
                    depth.as_ref(),
                    1.0,
                    (LoadOp::Clear, StoreOp::DontCare),
                    (LoadOp::DontCare, StoreOp::DontCare),
                    Cycle::No,
                    0,
                    (0u8, 0u8),
                );

                RenderPass::run(cmdbuf, &[color_target], Some(&depth_target), |rp| {
                    pipeline.bind(rp);

                    // Center the model (its bounding box is not centered at the
                    // origin: the teapot sits on the y = 0 plane), then rotate it
                    // in place about its center.
                    let model = trans.mul(&Mat4::rot_y(angle));
                    let second_model = Mat4::translate(-3.0, 0.0, -3.0).mul(&model);
                    let third_model = Mat4::translate(3.0, 0.0, -3.0).mul(&model);

                    const VIEW: Mat4 = Mat4::translate(0.0, 0.0, -4.5);

                    let proj = Mat4::perspective(
                        60.0f32.to_radians(),
                        width as f32 / height as f32,
                        0.1,
                        100.0,
                    );
                    let models = [model, second_model, third_model];
                    let view_proj = proj.mul(&VIEW);

                    const SZ: usize = size_of::<Mat4>();

                    let mut uniforms = [0u8; SZ * 4];
                    uniforms[..SZ].copy_from_slice(&view_proj.to_bytes());

                    for (index, model) in models.iter().enumerate() {
                        let start = (index + 1) * SZ;
                        uniforms[start..start + SZ].copy_from_slice(&model.to_bytes());
                    }

                    cmdbuf.push_vertex_uniform_data(0, &uniforms);

                    rp.bind_vertex_buffers(0, &[BufferBinding::new(vb.as_ref(), 0)]);
                    rp.bind_index_buffer(
                        &BufferBinding::new(ib.as_ref(), 0),
                        IndexElementSize::Bits16,
                    );
                    rp.draw_indexed_primitives(
                        mesh.indices.len() as u32,
                        models.len() as u32,
                        0,
                        0,
                        0,
                    );

                    Ok(())
                })?;
            }

            Ok(())
        })?;

        angle += 0.01;
    }

    device.wait_idle()?;

    device.release_window(wnd.as_ref());
    pipeline.drop(device.as_ref());
    fs.drop(device.as_ref());
    vs.drop(device.as_ref());
    ib.drop(device.as_ref());
    vb.drop(device.as_ref());
    tb.drop(device.as_ref());
    depth.drop(device.as_ref());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An unexpected error occurred: {e}");
    }
}
