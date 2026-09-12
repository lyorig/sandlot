use rustest::{Result, test};
use sandlot::{
    init::Context,
    pixels::PixelFormat,
    properties::Properties,
    rect::Point,
    renderer::Renderer,
    resource::Resource,
    texture::{Texture, TextureAccess},
    window::Window,
};
use sdl3_sys::render::SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER;

/// `Texture::builder` with `SDL_CreateTextureWithProperties`.
#[test]
fn texture_builder() -> Result {
    let _ctx = Context::new();
    let props = Properties::new()?;

    let wnd = Window::builder(props.as_ref())
        .hidden(true)
        .size(Point::new(128, 128))
        .build()?;

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

/// `Texture::properties` reflects what was used at creation.
#[test]
fn texture_properties() -> Result {
    let _ctx = Context::new();
    let props = Properties::new()?;

    let wnd = Window::builder(props.as_ref())
        .hidden(true)
        .size(Point::new(128, 128))
        .build()?;

    let rnd = Renderer::builder(props.as_ref())
        .window(wnd.as_ref())
        .build()?;

    let tex = Texture::builder(props.as_ref())
        .format(PixelFormat::Rgb24)
        .access(TextureAccess::Static)
        .size(Point::new(16, 16))
        .build(rnd.as_ref())?;

    let tp = tex.properties();
    assert!(tp.format() == PixelFormat::Rgb24);
    assert!(tp.access() == TextureAccess::Static);
    assert_eq!(tp.width(), 16);
    assert_eq!(tp.height(), 16);

    Ok(())
}

/// `build_cleanup` clears the texture creation properties.
#[test]
fn texture_build_cleanup() -> Result {
    let _ctx = Context::new();
    let props = Properties::new()?;

    let wnd = Window::builder(props.as_ref())
        .hidden(true)
        .size(Point::new(128, 128))
        .build()?;

    let rnd = Renderer::builder(props.as_ref())
        .window(wnd.as_ref())
        .build()?;

    let tex = Texture::builder(props.as_ref())
        .size(Point::new(16, 16))
        .build_cleanup(rnd.as_ref())?;

    assert_eq!(tex.size(), Point::new(16.0, 16.0));

    assert!(!unsafe { props.as_ref().has(SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER) });

    Ok(())
}
