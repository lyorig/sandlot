use rustest::{Result, main, test};

use sandlot::{
    init::{Context, Video},
    pixels::PixelFormat,
    properties::Properties,
    rect::{Point, PointI32},
    renderer::Renderer,
    texture::{Texture, TextureAccess},
    window::Window,
};

mod clipboard;
mod color;
mod error;
mod event;
mod fs;
mod init;
mod log;
mod string;
mod texture;
mod ttf;

/// Basic initialization stuff.
#[test]
fn main_init() -> Result {
    let ctx = Context::init()?;
    let vid = Video::init(ctx.as_ref())?;

    const WINDOW_SIZE: PointI32 = Point::new(128, 128);

    let props = Properties::new()?;

    let wnd = Window::builder(props.as_ref())
        .hidden(true)
        .size(WINDOW_SIZE)
        .build(vid.as_ref())?;

    assert_eq!(wnd.size(), WINDOW_SIZE);

    let rnd = Renderer::builder(props.as_ref())
        .window(wnd.as_ref())
        .build()?;

    let tex = Texture::builder(props.as_ref())
        .format(PixelFormat::Rgb24)
        .access(TextureAccess::Static)
        .size(Point::new(16, 16))
        .build(rnd.as_ref())?;

    assert_eq!(tex.size(), Point::new(16.0, 16.0));

    Ok(())
}

#[test]
fn main_subsystems() -> Result {
    let ctx = Context::init()?;

    {
        let _vid = Video::init(ctx.as_ref())?;
        assert!(Video::is_init());
    }

    assert!(!Video::is_init());

    Ok(())
}

#[test]
fn main_manually_drop() -> Result {
    {
        let ctx = Context::init()?;

        {
            let _vid = Video::leak(ctx.as_ref())?;
            assert!(Video::is_init());
        }

        // Still initialized, since `ManuallyDrop` skips the destructor.
        assert!(Video::is_init());
    }

    // Context should've cleaned everything up.
    assert!(!Video::is_init());

    Ok(())
}

#[main]
fn main() {}
