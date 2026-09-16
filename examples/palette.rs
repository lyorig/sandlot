use sandlot::{
    Result,
    color::{Rgb, Rgba},
    pixels::PixelFormat,
    rect::Point,
    surface::Surface,
};

fn run() -> Result<()> {
    let surf = Surface::new(Point::new(64, 64), PixelFormat::Index1Lsb)?;
    let pal = surf.create_palette()?;
    pal.set_colors(&[Rgba::RED, Rgba::GREEN], 0)?;

    sandlot::log!(
        "Closest palette index to `#117003`: {:x}",
        surf.map_rgb(Rgb::new(0x11, 0x70, 0x03))
    );

    // Won't work, since `surf` uses an indexed pixel format.
    assert!(surf.save_bmp(c"/tmp/surface.bmp").is_err());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("An error occurred: {e}");
    }
}
