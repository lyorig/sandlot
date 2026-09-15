use sandlot::{
    Result,
    color::Rgba,
    event::Event,
    init,
    rect::Point,
    renderer::Renderer,
    ttf,
    window::{Window, WindowFlags},
};

fn rt(s: &str) -> ttf::RtStr<'_> {
    unsafe { ttf::RtStr::new_unchecked(s) }
}

fn run() -> Result<()> {
    let ctx = init::Context::builder()
        .creator(c"lyorig")
        .identifier(c"cz.lyorig.sandlot-ttf")
        .url(c"https://github.com/lyorig/sandlot")
        .build()?;

    let vid = init::Video::init(ctx.as_ref())?;

    let ttf = ttf::Context::init()?;
    let font = ttf::Font::open(ttf.as_ref(), c"examples/fonts/Roboto.ttf", 32.0)?;

    let wnd = Window::new(
        vid.as_ref(),
        c"TTF Example",
        Point::new(640, 480),
        WindowFlags::empty(),
    )?;

    let rnd = Renderer::new(wnd.as_ref(), None)?;
    let eng = ttf::RendererEngine::new(rnd.as_ref())?;
    let text = ttf::RendererText::new(font.as_ref(), rt("And now I see,"), eng.as_ref())?;

    rnd.clear()?;

    text.set_color_f32(Rgba::RED)?;
    text.draw(Point::new(20.0, 20.0))?;

    text.set_string(rt("with eye serene,"))?;
    text.set_color_f32(Rgba::GREEN)?;
    text.draw(Point::new(60.0, 60.0))?;

    text.set_string(rt("the very pulse of the machine."))?;
    text.set_color_f32(Rgba::BLUE)?;
    text.draw(Point::new(100.0, 100.0))?;

    rnd.present()?;

    loop {
        if let Event::Quit = vid.events().wait()? {
            break;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An error occurred: {e}");
        sandlot::log_info!("note: due to my laziness to implement proper resource loading,");
        sandlot::log_info!("      this example must be run from the project root directory.");
    }
}
