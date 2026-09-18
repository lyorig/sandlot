//! Demonstrates key structs of the video subsystem, along with various drawing functions.

#![windows_subsystem = "windows"]

use sandlot::{
    Result,
    color::{Rgba, RgbaF32},
    event::Event,
    init::{Context, Video},
    rect::{Point, PointF32, Rect},
    renderer::{Renderer, RendererProperties},
    window::Window,
};
use sdl3_sys::render::SDL_Vertex;

fn print_properties(props: RendererProperties) {
    sandlot::log!("Renderer name: \"{}\"", props.name());
    sandlot::log!("HDR enabled: {}", props.is_hdr_enabled());
    sandlot::log!("HDR headroom: {}", props.hdr_headroom());
    sandlot::log!("Max texture size: {} px", props.max_texture_size());

    sandlot::log!("Supported texture formats:",);

    props
        .texture_formats()
        .iter()
        .for_each(|f| sandlot::log!("- {f}"));
}

fn vert(pos: PointF32, col: RgbaF32) -> SDL_Vertex {
    SDL_Vertex {
        position: pos.to_sdl(),
        color: col.into(),
        tex_coord: PointF32::new(0.0, 0.0).to_sdl(),
    }
}

fn run() -> Result<()> {
    let ctx = Context::init()?;
    let video = Video::leak(ctx.as_ref())?;
    let events = video.events();

    let props = ctx.global_properties()?;

    let wnd = Window::builder(props)
        .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
        .title(c"sandlot Example")
        .size(Point::new(640, 480))
        .build_cleanup(video)?;

    wnd.sync()?;

    let rnd = Renderer::builder(props)
        .window(wnd.as_ref())
        .vsync(1)
        .build_cleanup()?;

    rnd.clear()?;

    sandlot::log!("Platform = {}", sandlot::platform());
    print_properties(rnd.properties());

    rnd.set_draw_color_f32(Rgba::rgb(1., 1., 1.));
    rnd.draw_line(Point::new(10., 10.), Point::new(128., 64.))?;
    rnd.fill_rect(Some(&Rect::xywh(10., 90., 256., 256.)))?;

    rnd.set_draw_color_f32(Rgba::rgb(0., 1., 1.));
    rnd.fill_rects(&[
        Rect::xywh(100., 100., 10., 10.),
        Rect::xywh(110., 110., 20., 20.),
        Rect::xywh(130., 130., 20., 20.),
        Rect::xywh(150., 150., 30., 30.),
    ])?;

    let verts = [
        vert(PointF32::new(640., 300.), RgbaF32::RED),
        vert(PointF32::new(400., 480.), RgbaF32::GREEN),
        vert(PointF32::new(640., 480.), RgbaF32::BLUE),
    ];

    rnd.draw_geometry(&verts, None, None)?;

    rnd.present()?;

    'main: loop {
        if let Event::Quit = events.wait()? {
            break 'main;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An error occurred: {e}");
    }
}
