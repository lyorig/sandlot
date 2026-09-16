use sandlot::{
    Result,
    color::{Rgb, Rgba},
    event::Event,
    init::{Context, Video},
    pixels::PixelFormat,
    rect::Point,
    renderer::Renderer,
    surface::Surface,
    texture::Texture,
    window::{Window, WindowFlags},
};

fn run() -> Result<()> {
    let ctx = Context::init()?;
    let vid = Video::leak(ctx.as_ref())?;

    let wnd = Window::new(
        vid,
        c"Sandlot Palette Example",
        Point::new(640, 480),
        WindowFlags::empty(),
    )?;

    {
        let rnd = Renderer::new(wnd.as_ref(), None)?;

        let tex = {
            // 2 bits for the palette index => 4 colors.
            let surf = Surface::new(Point::new(160, 120), PixelFormat::Index2Lsb)?;
            let pal = surf.create_palette()?;
            pal.set_colors(&[Rgba::RED, Rgba::GREEN, Rgba::BLUE, Rgba::WHITE], 0)?;

            surf.lock_with(|px| {
                let len = surf.pixel_row_len();
                sandlot::log!("Pitch: {}, pixel row len: {}", surf.pitch(), len);
                sandlot::log!("Must lock? {}", surf.must_lock());

                // 00 01 10 11 (red, green, blue, white)
                let mut value = 0b00011011;

                px.chunks_exact_mut(surf.pitch() as usize)
                    .map(|row| &mut row[..len])
                    .for_each(|row| {
                        row.fill(value);
                        value = value.rotate_right(2);
                    });
            });

            sandlot::log!(
                "Closest palette index to `#117003`: {:x}",
                surf.map_rgb(Rgb::new(0x11, 0x70, 0x03))
            );

            Texture::from_surface(rnd.as_ref(), surf.as_ref())?
        };

        rnd.clear()?;
        rnd.draw(tex.as_ref(), None, None)?;
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
    }
}
