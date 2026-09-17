//! Shows the state of the system's clipboard.

use std::ffi::CStr;

use sandlot::{Result, init::Context, init::Video};

const DESIRED_MIME: &CStr = c"image/png";

fn run() -> Result<()> {
    let ctx = Context::init()?;
    let vid = Video::init(ctx.as_ref())?;

    if vid.clipboard_has_data(DESIRED_MIME) {
        println!("Clipboard has MIME data");
        println!("-- begin MIME type enumeration --");
        for ptr in vid.clipboard_mime_types()? {
            let cs = unsafe { CStr::from_ptr(ptr.as_ptr()) };
            println!("{}", cs.to_string_lossy());
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
