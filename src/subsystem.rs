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
use std::marker::PhantomData;

use sdl3_sys::init::*;

use crate::{Context, Result, error::Error};

/// A RAII type handling (de)initialization of an SDL subsystem.
///
/// # Usage with `ManuallyDrop`
/// By default, dropping a subsystem de-initializes it.
/// However, since dropping a [`Context`] de-initializes everything "by force"
/// anyways, you can potentially avoid redundant FFI calls by wrapping this struct
/// in `ManuallyDrop`:
///
/// ```rust
/// use std::mem::ManuallyDrop;
/// use sandlot::{Context, subsystem::Video};
///
/// let ctx = Context::new();
///
/// // Upon going out of scope, `ctx` de-initializes the video
/// // subsystem, making it redundant to drop `vid`.
/// let vid = ManuallyDrop::new(Video::new(&ctx).unwrap());
/// ```
pub struct Subsystem<'ctx, const TYPE: u32> {
    marker: PhantomData<&'ctx Context>,
}

impl<const N: u32> Subsystem<'_, N> {
    const FLAG: SDL_InitFlags = SDL_InitFlags::new(N);

    /// Initialize a specific SDL subsystem.
    ///
    /// # Remarks
    ///
    /// This function and SDL's library initialization are interchangeable.
    #[doc(alias = "SDL_InitSubSystem")]
    pub fn new(_: &Context) -> Result<Subsystem<'_, N>> {
        let res = unsafe { SDL_InitSubSystem(Self::FLAG) };
        if res {
            Ok(Subsystem {
                marker: PhantomData,
            })
        } else {
            Err(Error::current())
        }
    }

    /// Get whether this subsystem is currently initialized.
    #[doc(alias = "SDL_WasInit")]
    pub fn is_init() -> bool {
        let flag = unsafe { SDL_WasInit(Self::FLAG) };
        flag == Self::FLAG
    }
}

impl<const N: u32> Drop for Subsystem<'_, N> {
    /// Shut down this SDL subsystem.
    ///
    /// # Remarks
    ///
    /// You still need to call SDL's quit function even if you close all open
    /// subsystems; dropping the [`Context`] does this automatically.
    #[doc(alias = "SDL_QuitSubSystem")]
    fn drop(&mut self) {
        unsafe { SDL_QuitSubSystem(Self::FLAG) };
    }
}

#[expect(unused_imports)]
use crate::event::Event;

/// SDL's video subsystem is largely interested in abstracting window management
/// from the underlying operating system. You can create windows, manage them in
/// various ways, set them fullscreen, and get events when interesting things happen
/// with them, such as the mouse or keyboard interacting with a window.
///
/// The video subsystem is also interested in abstracting away some platform-specific
/// differences in OpenGL: context creation, swapping buffers, etc. This may be crucial to your app,
/// but also you are not required to use OpenGL at all. In fact, SDL can provide rendering to those
/// windows as well, either with an easy-to-use 2D API or with a more-powerful GPU API.
/// Of course, it can simply get out of your way and give you the window handles you need to use
/// Vulkan, Direct3D, Metal, or whatever else you like directly, too.
///
/// The video subsystem covers a lot of functionality, out of necessity,
/// so it is worth perusing [the list of functions](https://wiki.libsdl.org/SDL3/CategoryVideo)
/// just to see what's available, but most apps can get by with simply creating a window and listening for events,
/// so start with [`Window::new`](crate::window::Window::new) and [`Event::iter`].
pub type Video<'ctx> = Subsystem<'ctx, { SDL_InitFlags::VIDEO.0 }>;

/// Event queue management.
///
/// It's extremely common—often required—that an app deal with SDL's event
/// queue. Almost all useful information about interactions with the real
/// world flow through here: the user interacting with the computer and app,
/// hardware coming and going, the system changing in some way, etc.
///
/// An app generally takes a moment, perhaps at the start of a new frame, to
/// examine any events that have occurred since the last time and process or
/// ignore them. This is generally done by polling in a loop via [`Event::iter`]
/// until it returns [`None`]. (If using the main callbacks, events are provided
/// one at a time in calls to `SDL_AppEvent` before the next call to
/// `SDL_AppIterate`; in this scenario, the app does not poll at all.)
///
/// There are other forms of control, too: `SDL_PeepEvents` has more
/// functionality at the cost of more complexity, and `SDL_WaitEvent` can block
/// the process until something interesting happens, which might be beneficial
/// for certain types of programs on low-power hardware. One may also call
/// `SDL_AddEventWatch` to set a callback when new events arrive.
///
/// The app is free to generate their own events, too: [`Event::push`] allows
/// the app to put events onto the queue for later retrieval;
/// `SDL_RegisterEvents` can guarantee that these events have a type that isn't
/// in use by other parts of the system.
pub type Events<'ctx> = Subsystem<'ctx, { SDL_InitFlags::EVENTS.0 }>;
