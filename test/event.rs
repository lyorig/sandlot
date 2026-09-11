use rustest::test;
use sdl3_sys::events::*;

use sandlot::{event::Event, init::Context, init::Events};

/// [`SDL_Event`] -> [`Event`] conversion.
#[test]
fn event_sdl_to_hal() {
    // Manually set the timestamp for testing purposes.
    let ticks = sandlot::ticks_ns();

    let hal = Event::from(&SDL_Event {
        clipboard: SDL_ClipboardEvent {
            r#type: SDL_EVENT_CLIPBOARD_UPDATE,
            timestamp: ticks,
            ..Default::default()
        },
    });

    let Event::ClipboardUpdate(cu) = hal else {
        panic!("Expected clipboard update");
    };

    assert_eq!(cu.timestamp, ticks);
}

/// [`Event`] -> [`SDL_Event`] conversion.
#[test]
fn event_hal_to_sdl() {
    let mut sdl = SDL_Event::from(&Event::Quit);

    // Manually set the timestamp for testing purposes.
    let ticks = sandlot::ticks_ns();
    sdl.quit.timestamp = ticks;

    assert!(unsafe { sdl.quit.r#type } == SDL_EVENT_QUIT);
    assert!(unsafe { sdl.r#type } == SDL_EVENT_QUIT);
    assert_eq!(unsafe { sdl.quit }.timestamp, ticks);
    assert_eq!(unsafe { sdl.common }.timestamp, ticks);
}

/// [`Event::set_timestamp`].
#[test]
fn event_timestamp() {
    let mut evt = Event::Quit;
    let ticks = sandlot::ticks_ns();
    evt.set_timestamp(ticks);

    let sdl = SDL_Event::from(&evt);
    assert_eq!(unsafe { sdl.common }.timestamp, ticks);
}

/// [`EventsHandle::push`] testing.
#[test]
fn event_push() {
    // Initialize events.
    let ctx = Context::new();
    let evts = Events::init(&ctx).unwrap();

    // Should work now.
    evts.push(&Event::Quit).unwrap();

    let evt = evts.iter().next().unwrap();
    let Event::Quit = evt else {
        panic!("Expected quit event");
    };
}
