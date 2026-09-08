#![windows_subsystem = "windows"]

use std::mem::ManuallyDrop;

use sandlot::{
    Context, Result,
    color::Rgba,
    event::{Event, EventIter},
    properties::Properties,
    rect::{Point, Rect},
    renderer::{Renderer, RendererProperties},
    resource::Resource,
    subsystem::Video,
    window::Window,
};

fn print_properties(props: RendererProperties) {
    sandlot::log!("Renderer name: \"{}\"", props.name());
    sandlot::log!("HDR enabled: {}", props.hdr_enabled());
    sandlot::log!("HDR headroom: {}", props.hdr_headroom());
    sandlot::log!("Max texture size: {} px", props.max_texture_size());
    sandlot::log!("# of texture formats: {}", props.texture_formats().len());
}

fn run() -> Result<()> {
    let ctx = Context::new();
    let _vid = ManuallyDrop::new(Video::new(&ctx)?);

    let props = Properties::global()?;

    let wnd = Window::builder(props)
        .position(Point::new(Window::POS_CENTERED, Window::POS_CENTERED))
        .title(c"sandlot Example")
        .size(Point::new(640, 480))
        .build_cleanup()?;

    wnd.sync()?;

    let rnd = Renderer::builder(props)
        .window(wnd.as_ref())
        .vsync(1)
        .build_cleanup()?;

    rnd.clear()?;

    print_properties(rnd.properties());

    sandlot::log!("Platform = {}", sandlot::platform());

    rnd.set_draw_color_f32(Rgba::rgb(1., 1., 1.));
    rnd.draw_line(Point::new(10., 10.), Point::new(128., 64.))?;
    rnd.fill_rect(Rect::xywh(10., 90., 256., 256.))?;

    rnd.set_draw_color_f32(Rgba::rgb(0., 1., 1.));
    rnd.fill_rects(&[
        Rect::xywh(100., 100., 10., 10.),
        Rect::xywh(110., 110., 20., 20.),
        Rect::xywh(130., 130., 20., 20.),
        Rect::xywh(150., 150., 30., 30.),
    ])?;

    rnd.present()?;

    'main: loop {
        rnd.clear()?;

        for event in EventIter::new() {
            if let Event::Quit = event {
                break 'main;
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An error occurred: {e}");
    }
}
