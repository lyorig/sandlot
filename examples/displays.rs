use sandlot::{Result, init::Context, init::Video};

fn print_optional_property<T: std::fmt::Display>(name: &str, opt: Option<T>) {
    match opt {
        None => println!("- {} = <unavailable>", name),
        Some(t) => println!("- {} = {}", name, t),
    }
}

fn run() -> Result<()> {
    let ctx = Context::init()?;
    let vid = Video::init(ctx.as_ref())?;

    for (i, disp) in vid.displays_all()?.iter().copied().enumerate() {
        println!(
            "Display #{}: \"{}\", bounds {} (usable {}), content scale = {:.2}",
            i,
            disp.name()?.to_string_lossy(),
            disp.bounds()?,
            disp.usable_bounds()?,
            disp.content_scale()?
        );

        let props = disp.properties();
        println!("Display properties:");
        print_optional_property("HDR enabled", Some(props.hdr_enabled()));
        print_optional_property(
            "KMS/DRM panel orientation",
            props.kmsdrm_panel_orientation(),
        );
        print_optional_property(
            "Wayland `wl_output`",
            props.wayland_wl_output().map(|p| format!("{p:p}")),
        );
        print_optional_property(
            "Windows `HMONITOR`",
            props.windows_hmonitor().map(|p| format!("{p:p}")),
        );
    }

    let p = vid.display_primary()?;

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
        sandlot::log_error!("Something went wrong: {e}");
    }
}
