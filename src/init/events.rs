use std::mem::MaybeUninit;

use sdl3_sys::{
    events::*,
};

use crate::{
    Result,
    error::Error,
    event::{Event, EventIter},
    init::subsystem_new,
    util::to_result,
};

subsystem_new!(
    /// The events subsystem provides access to the event queue.
    Events, EVENTS);

impl<'ctx> EventsHandle<'ctx> {
    /// Add an event to the event queue.
    ///
    /// The event is copied into the queue.
    ///
    /// Returns [`Err`] if the event was filtered or on failure; a common
    /// reason for error is the event queue being full.
    ///
    /// # Remarks
    ///
    /// The event queue can actually be used as a two way communication
    /// channel. Not only can events be read from the queue, but the user can
    /// also push their own events onto it.
    ///
    /// Note: Pushing device input events onto the queue doesn't modify the
    /// state of the device within SDL.
    ///
    /// Note: Events pushed onto the queue get passed through the event
    /// filter.
    ///
    /// For pushing application-specific events, please use
    /// `SDL_RegisterEvents` to get an event type that does not conflict with
    /// other code that also wants its own custom event types.
    #[doc(alias = "SDL_PushEvent")]
    pub fn push(self, e: &Event) -> Result<()> {
        // NOTE: The timestamp is set internally in `SDL_PushEvent()`.
        let mut e = SDL_Event::from(e);
        to_result(unsafe { SDL_PushEvent(&raw mut e) })
    }

    /// Pump the event loop, gathering events from the input devices.
    ///
    /// # Remarks
    ///
    /// This function updates the event queue and internal input device state.
    ///
    /// This function gathers all the pending input information from devices
    /// and places it in the event queue. Without calls to this function no
    /// events would ever be placed on the queue. Usually the need for calls
    /// to it is hidden, since polling via [`EventIter`] or waiting via
    /// [`EventsHandle::wait`] implicitly pump the event loop. However, if you are not
    /// polling or waiting for events (e.g. you are filtering them), then you
    /// must call this function to force an event queue update.
    #[doc(alias = "SDL_PumpEvents")]
    pub fn pump(self) {
        unsafe { SDL_PumpEvents() };
    }

    /// Wait indefinitely for the next available event.
    ///
    /// Returns [`Err`] if there was an error while waiting for events.
    ///
    /// # Remarks
    ///
    /// This function may implicitly pump the event loop (see [`EventsHandle::pump`]).
    #[doc(alias = "SDL_WaitEvent")]
    pub fn wait(self) -> Result<Event> {
        let mut e = MaybeUninit::<SDL_Event>::uninit();

        if unsafe { SDL_WaitEvent(e.as_mut_ptr()) } {
            // SAFETY: SDL fully initializes the event on success.
            // The layouts of both types match.
            Ok(unsafe { e.assume_init_ref() }.into())
        } else {
            Err(Error::current())
        }
    }

    /// Returns an iterator over all [`Event`]s acquired since the last call
    /// (implicit or explicit) to [`EventsHandle::pump`].
    pub fn iter(&self) -> EventIter<'ctx, '_> {
        EventIter::new()
    }
}
