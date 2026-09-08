use sandlot::{Context, Result, display::Display, subsystem::Video};

fn run() -> Result<()> {
    let ctx = Context::new();
    let _vid = Video::new(&ctx)?;

    for (i, disp) in Display::all()?.iter().copied().enumerate() {
        println!(
            "Display #{}: \"{}\", bounds {} (usable {}), content scale = {:.2}",
            i,
            disp.name()?.to_string_lossy(),
            disp.bounds()?,
            disp.usable_bounds()?,
            disp.content_scale()?
        )
    }

    let p = Display::primary()?;

    println!("All primary desktop display modes:");
    for (x, y, hz) in p.fullscreen_modes()?.iter().map(|dm| {
        let dm = unsafe { dm.read() };
        (dm.w, dm.h, dm.refresh_rate)
    }) {
        println!("{x}x{y}, {hz} Hz");
    }

    println!(
        "Primary display has ID {}, and name \"{}\"",
        p.id().0,
        p.name()?.to_string_lossy(),
    );

    if let Some(o) = p.current_orientation() {
        println!("Current orientation is available and is \"{o}\"");
    }

    if let Some(o) = p.natural_orientation() {
        println!("Natural orientation is available and is \"{o}\"");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Something went wrong: {e}");
    }
}
