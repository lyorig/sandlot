//! Almost all Sandlot programs need to initialize SDL before starting to work with it.
//!
//! Almost everything can simply create a [`Context`] near startup, alongside a handful
//! of subsystems to touch. These are here to make sure SDL does not even attempt
//! to touch low-level pieces of the operating system that you don't intend to use.
//! For example, you might be using SDL for video and input but chose an external library
//! for audio, and in this case you would just need to leave off `subsystem::Audio` (not yet implemented)
//! to make sure that external library has complete control.
//!
//! When terminating, the [`Context`] will call [`SDL_Quit`]. This will clean up (nearly) everything
//! that SDL might have allocated, and crucially, it'll make sure that the display's resolution
//! is back to what the user expects if you had previously changed it for your game.
//!
//! Sandlot apps are strongly encouraged to call [`Context::builder`] at startup to fill in details
//! about the program. This is completely optional, but it helps in small ways (we can provide
//! an About dialog box for the macOS menu, we can name the app in the system's audio mixer, etc).
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryInit)):
//! - [x] SDL_GetAppMetadataProperty
//! - [x] SDL_Init
//! - [x] SDL_InitSubSystem
//! - [x] SDL_IsMainThread
//! - [x] SDL_Quit
//! - [x] SDL_QuitSubSystem
//! - [ ] SDL_RunOnMainThread
//! - [ ] SDL_SetAppMetadata (unnecessary)
//! - [x] SDL_SetAppMetadataProperty
//! - [x] SDL_WasInit

use std::{marker::PhantomData, mem::MaybeUninit, ops::Deref, ptr::NonNull};

use sdl3_sys::{
    events::{SDL_Event, SDL_PumpEvents, SDL_PushEvent, SDL_WaitEvent},
    filesystem::{SDL_GetBasePath, SDL_GetUserFolder},
    init::{SDL_Init, SDL_InitFlags, SDL_Quit},
    keyboard::{SDL_StartTextInput, SDL_StopTextInput, SDL_TextInputActive},
    video::{SDL_GetGrabbedWindow, SDL_GetWindowFromID, SDL_GetWindows},
};

use crate::{
    Result,
    boxed::Box,
    error::Error,
    event::{Event, EventIter},
    fs::Folder,
    resource,
    util::{c_ptr_to_str, mod_reexport, opt2res_map, to_result},
    window::{Window, WindowHandle, WindowId},
};

mod_reexport!(builder);
mod_reexport!(metadata);

#[derive(Clone, Copy)]
pub struct ContextHandle;

impl ContextHandle {
    /// Read the app metadata set via [`Context::builder`].
    ///
    /// The metadata lives in the global property group, so the returned
    /// [`ContextMetadata`] is zero-sized and borrows this [`Context`].
    ///
    /// All metadata string values are UTF-8.
    #[doc(alias = "SDL_GetAppMetadataProperty")]
    pub fn metadata(&self) -> ContextMetadata<'_> {
        ContextMetadata::new()
    }

    /// Get the directory where the application was run from.
    /// [`Err`] is returned on error or when the platform does not implement this functionality.
    ///
    /// Returns an absolute, UTF-8 path to the application data directory. The
    /// path is guaranteed to end with a path separator (`\\` on Windows and `/`
    /// on most other platforms).
    ///
    /// On macOS and iOS, an application inside a `.app` bundle returns the
    /// bundle's resource directory by default. This can be changed with the
    /// `SDL_FILESYSTEM_BASE_DIR_TYPE` property in `Info.plist`: `resource`
    /// selects the resource directory, `bundle` selects the bundle directory,
    /// and `parent` selects the directory containing the bundle. On Android,
    /// this returns `./`, which allows filesystem operations to use internal
    /// storage and the asset system. On Nintendo 3DS, this returns the
    /// application's `romfs` directory, which is not writable.
    ///
    /// SDL caches the result, but the first call may be slow.
    ///
    /// The cached path is freed when this [`Context`] is dropped, so its lifetime
    /// is tied to `&self`.
    #[doc(alias = "SDL_GetBasePath")]
    pub fn base_path(&self) -> Result<&str> {
        let ptr = unsafe { SDL_GetBasePath() };
        if ptr.is_null() {
            Err(Error::current())
        } else {
            // SAFETY: The string returned by `SDL_GetBasePath()`
            // is guaranteed to be valid UTF-8.
            Ok(unsafe { c_ptr_to_str(ptr) })
        }
    }

    /// Find the most suitable user folder for a specific purpose.
    ///
    /// `folder` selects the type of folder to find, such as
    /// [`Folder::Documents`] or [`Folder::Downloads`]. These are user folders
    /// intended for the user to access and manage. For application-specific data,
    /// use [`crate::fs::pref_path`] instead.
    ///
    /// Returns [`Err`] if the requested folder is unsupported or cannot be found.
    /// The returned path is guaranteed to end with a path separator (`\\` on
    /// Windows and `/` on most other platforms).
    ///
    /// SDL caches the result. The cached path is freed when this [`Context`] is
    /// dropped, so its lifetime is tied to `&self`.
    #[doc(alias = "SDL_GetUserFolder")]
    pub fn user_folder(&self, folder: Folder) -> Result<&str> {
        let ptr = unsafe { SDL_GetUserFolder(folder.into()) };

        // SAFETY: SDL guarantees the path is valid UTF-8.
        unsafe {
            opt2res_map(NonNull::new(ptr.cast_mut()), |ptr| {
                c_ptr_to_str(ptr.as_ptr())
            })
        }
    }
}

/// A zero-sized type that mainly exists to call [`SDL_Quit`].
/// As such, think of it as a guard that creates a scope for
/// the initialization of subsystems, ensuring they're properly
/// quit once it goes out of scope.
pub struct Context {
    handle: ContextHandle,
}

impl Context {
    /// Create a new context, enabling you to further initialize individual subsystems.
    ///
    /// Returns [`Err`] if SDL fails basic initialization.
    ///
    /// Only call this on the main thread!
    #[doc(alias = "SDL_Init")]
    pub fn init() -> Result<Self> {
        // This initializes the main thread and other basic stuff,
        // like setting app metadata.
        if unsafe { SDL_Init(SDL_InitFlags::new(0)) } {
            Ok(Self {
                handle: ContextHandle,
            })
        } else {
            Err(Error::current())
        }
    }

    /// Create a [`Context`], specifying metadata about your app through a builder-like interface.
    ///
    /// This metadata is stored in [`Properties::global`](crate::properties::Properties::global).
    ///
    /// # Remarks
    ///
    /// You can optionally provide metadata about your app to SDL. This is not required, but strongly encouraged.
    ///
    /// There are several locations where SDL can make use of metadata (an "About" box in the macOS menu bar,
    /// the name of the app can be shown on some audio mixers, etc). Any piece of metadata can be left out,
    /// if a specific detail doesn't make sense for the app.
    ///
    /// Once set, this metadata can be read using [`ContextHandle::metadata`].
    #[doc(alias = "SDL_SetAppMetadataProperty")]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }

    /// # Safety
    ///
    /// The caller must only use the returned handle within
    /// the lifetime of the backing [`Context`].
    pub unsafe fn as_handle(&self) -> ContextHandle {
        unsafe { Subsystem::as_handle(self) }
    }

    pub fn as_ref(&self) -> Ref<'_, Self> {
        Subsystem::as_ref(self)
    }
}

impl Deref for Context {
    type Target = ContextHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Subsystem for Context {
    type Handle = ContextHandle;

    unsafe fn as_handle(&self) -> Self::Handle {
        self.handle
    }
}

impl Drop for Context {
    /// Deinitialize SDL, quitting all subsystems.
    #[doc(alias = "SDL_Quit")]
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
    }
}

pub trait Subsystem: Sized {
    type Handle: Copy;

    /// # Safety
    ///
    /// The caller must only use the returned handle within
    /// the lifetime of the backing subsystem.
    unsafe fn as_handle(&self) -> Self::Handle;

    fn as_ref(&self) -> Ref<'_, Self> {
        unsafe { Ref::from_handle(self.as_handle()) }
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
    ///
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

macro_rules! subsystem_new {
    ($(#[$meta:meta])* $name:ident, $flag:ident $(, $implied:ident => $accessor:ident)*) => {
        paste::paste! {
            #[derive(Clone, Copy)]
            pub struct [<$name Handle>]<'ctx> {
                $(pub $accessor: [<$implied Handle>]<'ctx>,)*
                marker: ::std::marker::PhantomData<&'ctx $crate::init::Context>,
            }

            impl<'ctx> [<$name Handle>]<'ctx> {
                $(
                    #[doc = "Get a reference to the implicitly initialized [`" $implied "`] subsystem."]
                    pub fn $accessor(&self) -> $crate::init::Ref<'_, $implied<'ctx>> {
                        unsafe { $crate::init::Ref::from_handle(self.$accessor) }
                    }
                )*
            }

            $(#[$meta])*
            pub struct $name<'ctx> {
                handle: [<$name Handle>]<'ctx>,
            }


            impl<'ctx> $name<'ctx> {
                /// Initialize this subsystem, returning an owning handle.
                ///
                /// Upon going out of scope, the subsystem will be deinitialized.
                ///
                /// Returns [`Err`] if initialization fails.
                pub fn init(_ctx: $crate::init::Ref<'ctx, $crate::init::Context>) -> $crate::Result<Self> {
                    if unsafe { ::sdl3_sys::init::SDL_InitSubSystem(::sdl3_sys::init::SDL_InitFlags::$flag) } {
                        Ok(Self {
                            handle: [<$name Handle>] {
                                $(
                                    $accessor: [<$implied Handle>] {
                                        marker: ::std::marker::PhantomData
                                    },
                                )*
                                marker: ::std::marker::PhantomData
                            },
                        })
                    } else {
                        Err($crate::error::Error::current())
                    }
                }

                /// # Safety
                ///
                /// The caller must only use the returned handle within
                /// the lifetime of the backing subsystem.
                pub unsafe fn as_handle(&self) -> [<$name Handle>]<'ctx> {
                    unsafe { $crate::init::Subsystem::as_handle(self) }
                }

                pub fn as_ref(&self) -> $crate::init::Ref<'_, $name<'_>> {
                    unsafe { $crate::init::Ref::from_handle(self.as_handle()) }
                }

                /// Get whether this subsystem is currently initialized.
                #[doc(alias = "SDL_WasInit")]
                pub fn is_init() -> ::core::primitive::bool {
                    let init = unsafe { ::sdl3_sys::init::SDL_WasInit(::sdl3_sys::init::SDL_InitFlags::$flag) };
                    init == ::sdl3_sys::init::SDL_InitFlags::$flag
                }
            }

            impl<'ctx> Subsystem for $name<'ctx> {
                type Handle = [<$name Handle>]<'ctx>;

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.handle
                }
            }

            impl<'ctx> ::std::ops::Deref for $name<'ctx> {
                type Target = [<$name Handle>]<'ctx>;

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
    /// Also initializes the events subsystem, accessible via [`VideoHandle::events`].
    Video, VIDEO, Events => events);

impl<'ctx> VideoHandle<'ctx> {
    /// Get the window that currently has an input grab enabled.
    ///
    /// Returns [`None`] if input is not grabbed.
    #[doc(alias = "SDL_GetGrabbedWindow")]
    pub fn grabbed_window(&self) -> Option<WindowHandle<'ctx, '_>> {
        WindowHandle::from_ptr(unsafe { SDL_GetGrabbedWindow() })
    }

    /// Get a list of valid windows.
    #[doc(alias = "SDL_GetWindows")]
    pub fn windows<'a>(&self) -> Result<Box<[resource::Ref<'a, Window<'ctx, '_>>]>> {
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetWindows(count.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `count` window pointers.
        // `Ref<Window>` as the same size and alignment as `*mut SDL_Window`.
        unsafe { Box::from_raw_parts_nullck(ptr.cast(), count.assume_init() as usize) }
    }

    /// Get a window from a stored ID.
    ///
    /// Returns [`None`] if no window with that ID exists.
    ///
    /// # Safety
    ///
    /// The lifetime of the returned reference is inferred.
    /// In practice, it's going to be valid until the window is destroyed.
    /// It is your responsibility to only use it before that happens.
    ///
    /// # Remarks
    ///
    /// The numeric ID is what `SDL_WindowEvent` references, and is necessary
    /// to map these events to specific window objects.
    #[doc(alias = "SDL_GetWindowFromID")]
    pub unsafe fn window_from_id<'a>(
        &self,
        id: WindowId,
    ) -> Option<resource::Ref<'a, Window<'ctx, '_>>> {
        let ptr = unsafe { SDL_GetWindowFromID(id.as_raw()) };
        WindowHandle::from_ptr(ptr).map(|h| unsafe { resource::Ref::from_handle(h) })
    }
}

subsystem_new!(
    /// The events subsystem provides access to the event queue.
    Events, EVENTS);

impl EventsHandle<'_> {
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
        to_result(unsafe { SDL_StopTextInput(wnd.as_raw()) })
    }

    /// Check whether or not Unicode text input events are enabled for a window.
    #[doc(alias = "SDL_TextInputActive")]
    pub fn is_text_input_enabled(self, wnd: resource::Ref<Window>) -> bool {
        unsafe { SDL_TextInputActive(wnd.as_raw()) }
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
        to_result(unsafe { SDL_StartTextInput(wnd.as_raw()) })
    }

    /// Returns an iterator over all [`Event`]s acquired since the last call
    /// (implicit or explicit) to [`EventsHandle::pump`].
    pub fn iter(self) -> EventIter {
        EventIter::new()
    }
}
