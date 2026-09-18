use rustest::test;

use sandlot::{
    init::{AppKind, Context},
    s,
    str::Str,
};

/// `Context::metadata` reads back what `Context::builder` set.
#[test]
fn init_context_metadata() {
    const NAME: Str = s!("Sandlot");
    const VERSION: Str = s!("0.1.4"); // at the time of writing
    const IDENTIFIER: Str = s!("cz.lyorig.sandlot");
    const CREATOR: Str = s!("lyorig");
    const COPYRIGHT: Str = s!("Copyright (c) lyorig");
    const URL: Str = s!("https://github.com/lyorig/sandlot");

    let ctx = Context::builder()
        .name(NAME)
        .version(VERSION)
        .identifier(IDENTIFIER)
        .creator(CREATOR)
        .copyright(COPYRIGHT)
        .url(URL)
        .kind(AppKind::Game)
        .build()
        .unwrap();

    let md = ctx.metadata();
    assert_eq!(md.name(), NAME);
    assert_eq!(md.version(), Some(VERSION));
    assert_eq!(md.identifier(), Some(IDENTIFIER));
    assert_eq!(md.creator(), Some(CREATOR));
    assert_eq!(md.copyright(), Some(COPYRIGHT));
    assert_eq!(md.url(), Some(URL));
    assert_eq!(md.kind(), AppKind::Game);
}

/// `Context::metadata` provides default properties when not set explicitly.
#[test]
fn init_context_metadata_default() {
    let ctx = Context::init().unwrap();

    let md = ctx.metadata();

    // Turns out, SDL returns the binary name by default!
    // Only if that isn't available is the default "SDL Application" used.
    assert!(!md.name().is_empty());
    assert_eq!(md.version(), None);
    assert_eq!(md.identifier(), None);
    assert_eq!(md.creator(), None);
    assert_eq!(md.copyright(), None);
    assert_eq!(md.url(), None);
    assert_eq!(md.kind(), AppKind::Application);
}
