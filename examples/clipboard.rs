use std::ffi::CStr;

use sandlot::{Context, Result, clipboard, subsystem::Video};

const DESIRED_MIME: &CStr = c"image/png";

fn run() -> Result<()> {
    let ctx = Context::new();
    let _vid = Video::new(&ctx)?;

    if clipboard::has_data(DESIRED_MIME) {
        println!("Clipboard has MIME data");
        println!("-- begin MIME type enumeration --");
        for ptr in clipboard::mime_types()? {
            let cs = unsafe { CStr::from_ptr(ptr.as_ptr()) };
            println!("{}", cs.to_string_lossy());
        }
        println!("-- end MIME type enumeration --");

        let data = clipboard::data(DESIRED_MIME)?;
        println!("Clipboard data is {} bytes", data.len());
    } else if clipboard::has_text() {
        println!("Clipboard has text");
        println!("Text: \"{}\"", clipboard::text());
    } else {
        println!("Clipboard has neither MIME data nor text");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Something went wrong: {e}");
    }
}
