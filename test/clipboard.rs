use sandlot::{
    init::{Context, Video},
    s,
};

use rustest::{Result, test};

/// `set_text` succeeds after the video subsystem is initialized.
#[test]
fn clipboard_set_text_succeeds_after_video_init() -> Result {
    let ctx = Context::init()?;
    let vid = Video::init(ctx.as_ref())?;

    vid.clipboard_set_text(s!("clipboard test payload"))?;

    let roundtrip = vid.clipboard_text();
    assert_eq!(roundtrip.to_str(), "clipboard test payload");

    Ok(())
}

/// `has_text` reflects clipboard state after init.
#[test]
fn clipboard_has_text_after_video_init() -> Result {
    let ctx = Context::init()?;
    let vid = Video::init(ctx.as_ref())?;

    vid.clipboard_set_text(s!("exists"))?;
    assert!(vid.clipboard_has_text());

    // Reading it back should match.
    assert_eq!(vid.clipboard_text().to_str(), "exists");

    Ok(())
}
