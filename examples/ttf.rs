use sandlot::{
    Result,
    color::Rgba,
    event::Event,
    init::{Context, Video},
    pixels::PixelFormat,
    rect::{Point, Rect},
    renderer::Renderer,
    surface::Surface,
    texture::Texture,
    ttf::{self, *},
    window::{Window, WindowFlags},
};

fn rt(s: &str) -> RtStr<'_> {
    unsafe { RtStr::new_unchecked(s) }
}

fn run() -> Result<()> {
    let ctx = Context::builder()
        .creator(c"lyorig")
        .identifier(c"cz.lyorig.sandlot-ttf")
        .url(c"https://github.com/lyorig/sandlot")
        .build()?;

    let vid = Video::init(ctx.as_ref())?;

    let ttf = ttf::Context::init()?;
    let font = Font::open(ttf.as_ref(), c"examples/fonts/Roboto.ttf", 32.0)?;

    let wnd = Window::new(
        vid.as_ref(),
        c"TTF Example",
        Point::new(640, 480),
        WindowFlags::empty(),
    )?;

    {
        let rnd = Renderer::new(wnd.as_ref(), None)?;
        let eng = RendererEngine::new(rnd.as_ref())?;
        let text = {
            let text = Text::new(font.as_ref(), rt("And now I see,"))?;
            RendererText::new(text, eng.as_ref())?
        };

        rnd.clear()?;

        text.set_color_f32(Rgba::RED)?;
        text.draw(Point::new(20.0, 20.0))?;

        text.set_string(rt("with eye serene,"))?;
        text.set_color_f32(Rgba::GREEN)?;
        text.set_direction(Direction::RightToLeft)?;
        text.draw(Point::new(60.0, 60.0))?;

        text.set_string(rt("the very pulse of the machine."))?;
        text.set_color_f32(Rgba::BLUE)?;
        text.set_direction(Direction::TopToBottom)?;
        text.draw(Point::new(100.0, 100.0))?;

        let tex = {
            let eng = SurfaceEngine::new()?;
            let text = SurfaceText::new(text.into_text(), eng.as_ref())?;

            text.set_direction(Direction::LeftToRight)?;
            text.set_string(rt("Look ma, I'm a texture!"))?;
            text.set_color_f32(Rgba::CYAN)?;

            let surf = Surface::new(text.size(), PixelFormat::RGBA32)?;
            text.draw(surf.as_ref(), Point::new(0, 0))?;

            Texture::from_surface(rnd.as_ref(), surf.as_ref())?
        };

        rnd.draw(
            tex.as_ref(),
            None,
            Some(&Rect::xywh(128.0, 128.0, 128.0, 32.0)),
        )?;

        rnd.present()?;
    }

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
