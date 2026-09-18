//! Shows the state of the system's clipboard.

use std::ffi::CStr;

use sandlot::{
    Result,
    init::{AppKind, Context, Video},
    s,
};

const DESIRED_MIME: &CStr = c"image/png";

fn run() -> Result<()> {
    let ctx = Context::builder()
        .name(s!("Sandlot Clipboard Introspection Demo"))
        .version(s!("v0.1.4"))
        .creator(s!("lyorig"))
        .url(s!("https://github.com/lyorig/sandlot"))
        .kind(AppKind::Application)
        .build()?;

    let vid = Video::init(ctx.as_ref())?;

    if vid.clipboard_has_data(DESIRED_MIME) {
        println!("Clipboard has MIME data");
        println!("-- begin MIME type enumeration --");
        for cs in vid.clipboard_mime_types()? {
            println!("{cs}");
        }
        println!("-- end MIME type enumeration --");

        let data = vid.clipboard_data(DESIRED_MIME)?;
        println!("Clipboard data is {} bytes", data.len());
    } else if vid.clipboard_has_text() {
        println!("Clipboard has text");
        println!("Text: \"{}\"", vid.clipboard_text());
    } else {
        println!("Clipboard has neither MIME data nor text");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        sandlot::log_error!("Something went wrong: {e}");
    }
}
