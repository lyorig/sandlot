//! Almost all Sandlot programs need to initialize SDL before starting to work with it.
//!
//! Almost everything can simply create a [`Context`] near startup, alongside a handful
//! of subsystems to touch. These are here to make sure SDL does not even attempt
//! to touch low-level pieces of the operating system that you don't intend to use.
//! For example, you might be using SDL for video and input but chose an external library
//! for audio, and in this case you would just need to leave off `subsystem::Audio` (not yet implemented)
//! to make sure that external library has complete control.
//!
//! When terminating, the [`Context`] will call `SDL_Quit`. This will clean up (nearly) everything
//! that SDL might have allocated, and crucially, it'll make sure that the display's resolution
//! is back to what the user expects if you had previously changed it for your game.
//!
//! SDL3 apps are strongly encouraged to call `SDL_SetAppMetadata` at startup to fill in details
//! about the program. This is completely optional, but it helps in small ways (we can provide
//! an About dialog box for the macOS menu, we can name the app in the system's audio mixer, etc).
//! Those that want to provide a lot of information should look at the more-detailed `SDL_SetAppMetadataProperty`.

use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use sdl3_sys::{
    events::{SDL_Event, SDL_PumpEvents, SDL_PushEvent, SDL_WaitEvent},
    keyboard::{SDL_StartTextInput, SDL_StopTextInput, SDL_TextInputActive},
};

use crate::{
    Result,
    error::Error,
    event::{Event, EventIter},
    resource,
    util::to_result,
    window::Window,
};

pub trait Subsystem: Sized {
    type Handle: Copy;

    /// # Safety
    /// Think of this function as returning a pointer to `self`.
    /// Handles are only valid as long as their owning objects.
    unsafe fn as_handle(&self) -> Self::Handle;

    fn as_ref(&self) -> Ref<'_, Self> {
        unsafe { Ref::from_handle(self.as_handle()) }
    }

    fn as_mut(&mut self) -> RefMut<'_, Self> {
        unsafe { RefMut::from_handle(self.as_handle()) }
    }
}

#[derive(Clone, Copy)]
pub struct Ref<'sub, T: Subsystem> {
    handle: T::Handle,
    marker: PhantomData<&'sub T>,
}

impl<T: Subsystem> Ref<'_, T> {
    /// Construct a new reference from a handle, assuming it is valid.
    /// This conversion is zero-cost.
    ///
    /// # Safety
    /// The lifetime of the returned reference is inferred; functions building
    /// on this one should tie it to the handle's owning object, if possible.
    pub unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            marker: PhantomData,
        }
    }
}

impl<T: Subsystem> Deref for Ref<'_, T> {
    type Target = T::Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

pub struct RefMut<'sub, T: Subsystem> {
    handle: T::Handle,
    marker: PhantomData<&'sub mut T>,
}

impl<T: Subsystem> Deref for RefMut<'_, T> {
    type Target = T::Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T: Subsystem> RefMut<'_, T> {
    /// Construct a new mutable reference from a handle, assuming it is valid.
    /// This conversion is zero-cost.
    ///
    /// # Safety
    /// The lifetime of the returned reference is inferred; functions building
    /// on this one should tie it to the handle's owning object, if possible.
    pub unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            marker: PhantomData,
        }
    }
}

impl<T: Subsystem> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

macro_rules! subsystem_new {
    ($(#[$meta:meta])* $name:ident, $flag:ident $(, $implied:ident => $accessor:ident)*) => {
        paste::paste! {
            #[derive(Clone, Copy)]
            pub struct [<$name Handle>];

            $(#[$meta])*
            pub struct $name<'ctx> {
                handle: [<$name Handle>],
                marker: ::std::marker::PhantomData<&'ctx $crate::Context>,
            }


            impl<'ctx> $name<'ctx> {
                /// Initialize this subsystem, returning an owning handle.
                ///
                /// Upon going out of scope, the subsystem will be deinitialized.
                ///
                /// Returns [`Err`] if initialization fails.
                pub fn init(_ctx: &'ctx $crate::Context) -> $crate::Result<Self> {
                    if unsafe { ::sdl3_sys::init::SDL_InitSubSystem(::sdl3_sys::init::SDL_InitFlags::$flag) } {
                        Ok(Self {
                            handle: [<$name Handle>],
                            marker: ::std::marker::PhantomData,
                        })
                    } else {
                        Err($crate::error::Error::current())
                    }
                }

                $(
                    #[doc = "Get a reference to the implicitly initialized [`" $implied "`] subsystem."]
                    pub fn $accessor(&self) -> $crate::init::Ref<'_, $implied<'ctx>> {
                        unsafe { $crate::init::Ref::from_handle([<$implied Handle>]) }
                    }

                    #[doc = "Get a mutable reference to the implicitly initialized [`" $implied "`] subsystem."]
                    pub fn [<$accessor _mut>](&mut self) -> $crate::init::RefMut<'_, $implied<'ctx>> {
                        unsafe { $crate::init::RefMut::from_handle([<$implied Handle>]) }
                    }
                )*

                /// Get whether this subsystem is currently initialized.
                #[doc(alias = "SDL_WasInit")]
                pub fn is_init() -> ::core::primitive::bool {
                    let init = unsafe { ::sdl3_sys::init::SDL_WasInit(::sdl3_sys::init::SDL_InitFlags::$flag) };
                    init == ::sdl3_sys::init::SDL_InitFlags::$flag
                }
            }

            impl Subsystem for $name<'_> {
                type Handle = [<$name Handle>];

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.handle
                }
            }

            impl ::std::ops::Deref for $name<'_> {
                type Target = [<$name Handle>];

                fn deref(&self) -> &Self::Target {
                    &self.handle
                }
            }

            impl ::std::ops::DerefMut for $name<'_> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.handle
                }
            }

            impl Drop for $name<'_> {
                /// Shut down this SDL subsystem.
                ///
                /// # Remarks
                ///
                /// You still need to call SDL's quit function even if you close all open
                /// subsystems (dropping the [`Context`] does this automatically).
                fn drop(&mut self) {
                    unsafe { ::sdl3_sys::init::SDL_QuitSubSystem(::sdl3_sys::init::SDL_InitFlags::$flag) };
                }
            }
        }
    };
}

subsystem_new!(
    /// The video subsystem provides access to the display and windowing system.
    /// Also initializes the events subsystem, accessible via [`Video::events`].
    Video, VIDEO, Events => events);

subsystem_new!(
    /// The events subsystem provides access to the event queue.
    Events, EVENTS);

impl EventsHandle {
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
    /// [`Self::wait`] implicitly pump the event loop. However, if you are not
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
    /// This function may implicitly pump the event loop (see [`Self::pump`]).
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

    /// Stop receiving any text input events in a window.
    ///
    /// # Remarks
    ///
    /// If [`EventsHandle::enable_text_input`] showed the screen keyboard,
    /// this function will hide it.
    #[doc(alias = "SDL_StopTextInput")]
    pub fn disable_text_input(self, wnd: resource::Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StopTextInput(wnd.handle.as_ptr()) })
    }

    /// Check whether or not Unicode text input events are enabled for a window.
    #[doc(alias = "SDL_TextInputActive")]
    pub fn is_text_input_enabled(self, wnd: resource::Ref<Window>) -> bool {
        unsafe { SDL_TextInputActive(wnd.handle.as_ptr()) }
    }

    /// Start accepting Unicode text input events in a window.
    ///
    /// # Remarks
    ///
    /// This function will enable text input ([`Event::TextInput`] and
    /// [`Event::TextEditing`] events) in the specified window. Please use
    /// this function paired with [`EventsHandle::disable_text_input`].
    ///
    /// Text input events are not received by default.
    ///
    /// On some platforms using this function shows the screen keyboard and/or
    /// activates an IME, which can prevent some key press events from being
    /// passed through.
    #[doc(alias = "SDL_StartTextInput")]
    pub fn enable_text_input(self, wnd: resource::Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StartTextInput(wnd.handle.as_ptr()) })
    }

    /// Returns an iterator over all [`Event`]s acquired since the last call
    /// (implicit or explicit) to [`EventsHandle::pump`].
    pub fn iter(self) -> EventIter {
        EventIter::new()
    }
}
