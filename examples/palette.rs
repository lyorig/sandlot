//! Demonstrates creating an indexed surface with a palette, along with
//! modifying its raw pixel data using [`SurfaceHandle::lock_pixels`].

use sandlot::{
    Result,
    color::{Rgb, Rgba},
    event::Event,
    init::{Context, Video},
    pixels::PixelFormat,
    rect::Point,
    renderer::Renderer,
    resource::Ref,
    surface::{Surface, SurfaceProperties},
    texture::Texture,
    window::{Window, WindowFlags},
};

#[expect(unused_imports)] // doc-only
use sandlot::surface::SurfaceHandle;

fn draw_surface_to(wnd: Ref<Window>, surf: Surface) -> Result<()> {
    let rnd = Renderer::new(wnd, None)?;
    let tex = Texture::from_surface(rnd.as_ref(), surf.as_ref())?;

    rnd.clear()?;
    rnd.draw(tex.as_ref(), None, None)?;
    rnd.present()?;

    Ok(())
}

fn print_optional_property<T: std::fmt::Display>(name: &str, opt: Option<T>) {
    match opt {
        None => sandlot::log!("- {} = <unavailable>", name),
        Some(t) => sandlot::log!("- {} = {}", name, t),
    }
}

fn print_properties(props: SurfaceProperties) {
    sandlot::log!("Properties:");
    print_optional_property("SDR white point", props.sdr_white_point());
    print_optional_property("HDR headroom", props.hdr_headroom());
    print_optional_property("Tone map operator", props.tonemap_operator());
    print_optional_property("hotspot_x", props.hotspot_x());
    print_optional_property("hotspot_y", props.hotspot_y());
    sandlot::log!("- Rotation: {}", props.rotation());
}

fn run() -> Result<()> {
    let ctx = Context::init()?;
    let vid = Video::leak(ctx.as_ref())?;

    let wnd = Window::new(
        vid,
        c"Sandlot Palette Example",
        Point::new(640, 480),
        WindowFlags::empty(),
    )?;

    // 2 bits for the palette index => 4 colors.
    let surf = Surface::new(Point::new(160, 120), PixelFormat::Index2Lsb)?;
    let pal = surf.create_palette()?;

    // Index mapping:
    // 0b00 -> red
    // 0b01 -> green
    // 0b10 -> blue
    // 0b11 -> white
    pal.set_colors(&[Rgba::RED, Rgba::GREEN, Rgba::BLUE, Rgba::WHITE], 0)?;

    // Modify the surface's pixels with a rotating bit pattern.
    //
    // Since `Surface::new` initializes the surface's pixels to zeroes,
    // if we let its pixels be, the whole surface would be red.
    surf.lock_pixels(|px| {
        let len = surf.pixel_row_len();
        sandlot::log!("Pitch: {}, pixel row len: {}", surf.pitch(), len);
        sandlot::log!("Must lock? {}", surf.must_lock());

        // 00 01 10 11 (red, green, blue, white)
        let mut value = 0b00011011;

        // For conciseness, we fill the entire row (including padding).
        px.chunks_exact_mut(surf.pitch() as usize).for_each(|row| {
            row.fill(value);
            value = value.rotate_right(2);
        });
    });

    print_properties(surf.properties());

    sandlot::log!(
        "Closest palette index to `#117003`: {:x}",
        surf.map_rgb(Rgb::new(0x11, 0x70, 0x03))
    );

    draw_surface_to(wnd.as_ref(), surf)?;

    loop {
        if let Event::Quit = vid.events().wait()? {
            break;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("{e}");
    }
}
