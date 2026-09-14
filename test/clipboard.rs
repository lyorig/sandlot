use std::assert_matches;

use sandlot::{
    clipboard::{has_text, set_text, text},
    init::{Context, Video},
};

use rustest::{Result, test};

/// `set_text` fails before the video subsystem is initialized.
#[test]
fn clipboard_set_text_fails_before_video_init() {
    assert_matches!(set_text(c"hello"), Err(_));
}

/// `set_text` succeeds after the video subsystem is initialized.
#[test]
fn clipboard_set_text_succeeds_after_video_init() -> Result {
    let ctx = Context::init()?;
    let _video = Video::init(ctx.as_ref())?;

    set_text(c"clipboard test payload")?;

    let roundtrip = text();
    assert_eq!(roundtrip.to_str(), "clipboard test payload");

    Ok(())
}

/// `has_text` reflects clipboard state after init.
#[test]
fn clipboard_has_text_after_video_init() -> Result {
    let ctx = Context::init()?;
    let _video = Video::init(ctx.as_ref())?;

    set_text(c"exists")?;
    assert!(has_text());

    // Reading it back should match.
    assert_eq!(text().to_str(), "exists");

    Ok(())
}
