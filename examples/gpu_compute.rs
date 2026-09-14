//! A minimal compute shader example.

#![windows_subsystem = "windows"]

use sandlot::{
    Result,
    gpu::*,
    init::{AppKind, Context, Video},
    properties::Properties,
};

cfg_select! {
    target_os = "macos" => {
        const COMPUTE_CODE: &[u8] = include_bytes!("shaders/compute.metallib");
        const SHADER_FMT: ShaderFormat = ShaderFormat::Metallib;
    },
    target_os = "windows" => {
        const COMPUTE_CODE: &[u8] = include_bytes!("shaders/compute_cs.dxil");
        const SHADER_FMT: ShaderFormat = ShaderFormat::Dxil;
    }
    target_os = "linux" => {
        const COMPUTE_CODE: &[u8] = include_bytes!("shaders/compute_cs.spv");
        const SHADER_FMT: ShaderFormat = ShaderFormat::SpirV;
    }
}

fn run() -> Result<()> {
    let ctx = Context::builder()
        .name(c"Sandlot GPU Compute")
        .creator(c"lyorig")
        .kind(AppKind::Application)
        .version(c"v0.1.4")
        .identifier(c"cz.lyorig.gpucompute")
        .url(c"https://github.com/lyorig/sandlot")
        .build()?;

    let video = Video::init(ctx.as_ref())?;

    let props = Properties::global()?;

    let device = Device::builder(props)
        .debug_mode(false)
        .shaders_metallib(true)
        .shaders_dxil(true)
        .shaders_spirv(true)
        .build_cleanup(video.as_ref())?;

    let pipeline_info = ComputePipelineCreateInfo::new(
        COMPUTE_CODE,
        c"cs_main",
        SHADER_FMT,
        0,
        0,
        (0, 0, 0, 1),
        (1, 1, 1),
    );

    let pipeline = ComputePipeline::builder(props)
        .name(c"Compute Pipeline")
        .build_cleanup(device.as_ref(), pipeline_info)?;

    type Buf = BufferUsageFlags;
    let buffer_info =
        BufferCreateInfo::new(Buf::COMPUTE_STORAGE_READ | Buf::COMPUTE_STORAGE_WRITE, 4);

    let buffer = Buffer::builder(props)
        .name(c"Compute Buffer")
        .build_cleanup(device.as_ref(), buffer_info)?;

    // Infallible.
    _ = CommandBuffer::run(device.as_ref(), |cmdbuf| {
        let rwb = StorageBufferReadWriteBinding::new(buffer.as_ref(), Cycle::No);
        _ = ComputePass::run(cmdbuf, &[], &[rwb], |pass| {
            pass.bind(pipeline.as_ref());
            pass.dispatch((1, 1, 1));

            Ok(())
        });

        Ok(())
    });

    device.wait_idle()?;

    buffer.drop(device.as_ref());
    pipeline.drop(device.as_ref());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An unexpected error occurred: {e}");
    }
}
