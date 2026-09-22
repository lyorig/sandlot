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
//!
//! # Clipboard (`Video::clipboard_*`)
//!
//! Access to the system clipboard. Useful both for reading information from other
//! processes and publishing information of its own. This is not just text!
//! SDL apps can access and publish data by mimetype.
//!
//! ## Basic use (text)
//!
//! Obtaining and publishing simple text to the system clipboard is as easy as
//! calling [`VideoHandle::clipboard_text`] and [`VideoHandle::clipboard_set_text`],
//! respectively. These deal with strings in UTF-8 encoding. Data transmission
//! and encoding conversion is completely managed by SDL.
//!
//! ## Clipboard callbacks (data other than text)
//!
//! Things get more complicated when the clipboard contains something other
//! than text. Not only can the system clipboard contain data of any type, in
//! some cases it can contain the same data in different formats! For example,
//! an image painting app might let the user copy a graphic to the clipboard,
//! and offers it in .BMP, .JPG, or .PNG format for other apps to consume.
//!
//! Obtaining clipboard data ("pasting") like this is a matter of calling
//! [`VideoHandle::clipboard_data`] and telling it the mimetype of the data you want.
//! But how does one know if that format is available? [`VideoHandle::clipboard_has_data`]
//! an report if a specific mimetype is offered, and [`VideoHandle::clipboard_mime_types`]
//! can provide the entire list of mimetypes available, so the app can decide what to do with the
//! data and what formats it can support.
//!
//! Setting the clipboard ("copying") to arbitrary data is done with
//! [`SDL_SetClipboardData`]. The app does not provide the data in this call,
//! but rather the mimetypes it is willing to provide and a callback function.
//! During the callback, the app will generate the data. This allows massive
//! data sets to be provided to the clipboard, without any data being copied
//! before it is explicitly requested. More specifically, it allows an app to
//! offer data in multiple formats without providing a copy of all of them
//! upfront. If the app has an image that it could provide in PNG or JPG
//! format, it doesn't have to encode it to either of those unless and until
//! something tries to paste it.
//!
//! ## Primary Selection
//!
//! The X11 and Wayland video targets have a concept of the "primary
//! selection" in addition to the usual clipboard. This is generally
//! highlighted (but not explicitly copied) text from various apps. SDL offers
//! APIs for this through [`SDL_GetPrimarySelectionText`] and
//! [`SDL_SetPrimarySelectionText`]. SDL offers these APIs on platforms without
//! this concept, too, but only so far that it will keep a copy of a string
//! that the app sets for later retrieval; the operating system will not ever
//! attempt to change the string externally if it doesn't support a primary
//! selection.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryClipboard)):
//! - [x] SDL_ClearClipboardData
//! - [x] SDL_GetClipboardData
//! - [x] SDL_GetClipboardMimeTypes
//! - [x] SDL_GetClipboardText
//! - [ ] SDL_GetPrimarySelectionText
//! - [x] SDL_HasClipboardData
//! - [x] SDL_HasClipboardText
//! - [ ] SDL_HasPrimarySelectionText
//! - [ ] SDL_SetClipboardData
//! - [x] SDL_SetClipboardText
//! - [ ] SDL_SetPrimarySelectionText

use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use sdl3_sys::{
    filesystem::{SDL_GetBasePath, SDL_GetUserFolder},
    init::*,
    properties::SDL_GetGlobalProperties,
};

use crate::{
    Result,
    error::Error,
    fs::Folder,
    properties::{Properties, PropertiesHandle},
    resource,
    util::{c_ptr_to_str, mod_reexport, opt2res_map},
};

#[expect(unused_imports)]
use sdl3_sys::clipboard::{
    SDL_GetPrimarySelectionText, SDL_SetClipboardData, SDL_SetPrimarySelectionText,
};

mod_reexport!(audio);
mod_reexport!(builder);
mod_reexport!(metadata);
mod_reexport!(video);
mod_reexport!(events);

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

    /// Get the global SDL properties.
    #[doc(alias = "SDL_GetGlobalProperties")]
    pub fn global_properties(&self) -> Result<resource::Ref<'_, Properties>> {
        let id = unsafe { SDL_GetGlobalProperties() };
        match PropertiesHandle::from_id(id) {
            Some(p) => Ok(unsafe { resource::Ref::from_handle(p) }),
            None => Err(Error::current()),
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
        // This initializes the main thread and other basic stuff.
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
    /// This metadata is stored in the global properties (see [`ContextHandle::global_properties`]).
    /// Although it **doesn't seem to be used** by SDL as of version 3.4.16, specifying it is
    /// still recommended, as future versions may make use of it.
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

    /// Obtain a raw handle to this resource.
    ///
    /// Handles are essentially raw pointers to an underlying resource, having next to no lifetime guarantees.
    /// Using them at the wrong time (i.e. after their resource has been destroyed) will break many invariants
    /// that the API relies on.
    ///
    /// # Safety
    ///
    /// The caller must only use the returned handle within the lifetime of the backing resource.
    pub unsafe fn as_handle(&self) -> ContextHandle {
        self.handle
    }

    pub fn as_ref(&self) -> Ref<'_, Self> {
        unsafe { Ref::from_handle(self.handle) }
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
}

pub struct Ref<'sub, T: Subsystem> {
    handle: T::Handle,
    marker: PhantomData<&'sub T>,
}

impl<T: Subsystem> Clone for Ref<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Subsystem> Copy for Ref<'_, T> {}

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
                $($accessor: $crate::init::[<$implied Handle>]<'ctx>,)*
                pub(crate) marker: ::std::marker::PhantomData<&'ctx $crate::init::Context>,
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
                #[doc(alias = "SDL_InitSubSystem")]
                pub fn init(_ctx: $crate::init::Ref<'ctx, $crate::init::Context>) -> $crate::Result<Self> {
                    if unsafe { ::sdl3_sys::init::SDL_InitSubSystem(::sdl3_sys::init::SDL_InitFlags::$flag) } {
                        Ok(Self {
                            handle: [<$name Handle>] {
                                $(
                                    $accessor: $crate::init::[<$implied Handle>] {
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

                /// Initialize this subsystem, deferring its deinitialization to the [`Context`](crate::init::Context) drop.
                ///
                /// Returns [`Err`] if initialization fails.
                #[doc(alias = "SDL_InitSubSystem")]
                pub fn leak(ctx: $crate::init::Ref<'ctx, $crate::init::Context>) -> $crate::Result<$crate::init::Ref<'ctx, $name<'ctx>>> {
                    let sub = Self::init(ctx)?;
                    let md = ::std::mem::ManuallyDrop::new(sub);

                    Ok(unsafe { $crate::init::Ref::from_handle(md.as_handle())})
                }

                /// Obtain a raw handle to this resource.
                ///
                /// Handles are essentially raw pointers to an underlying resource, having next to no lifetime guarantees.
                /// Using them at the wrong time (i.e. after their resource has been destroyed) will break many invariants
                /// that the API relies on.
                ///
                /// # Safety
                ///
                /// The caller must only use the returned handle within the lifetime of the backing resource.
                pub unsafe fn as_handle(&self) -> [<$name Handle>]<'ctx> {
                    self.handle
                }

                pub fn as_ref(&self) -> $crate::init::Ref<'_, $name<'_>> {
                    unsafe { $crate::init::Ref::from_handle(self.handle) }
                }

                /// Get whether this subsystem is currently initialized.
                #[doc(alias = "SDL_WasInit")]
                pub fn is_init() -> ::core::primitive::bool {
                    let init = unsafe { ::sdl3_sys::init::SDL_WasInit(::sdl3_sys::init::SDL_InitFlags::$flag) };
                    init == ::sdl3_sys::init::SDL_InitFlags::$flag
                }
            }

            impl<'ctx> $crate::init::Subsystem for $name<'ctx> {
                type Handle = [<$name Handle>]<'ctx>;
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
                /// subsystems (dropping the [`Context`](crate::init::Context) does this automatically).
                fn drop(&mut self) {
                    unsafe { ::sdl3_sys::init::SDL_QuitSubSystem(::sdl3_sys::init::SDL_InitFlags::$flag) };
                }
            }
        }
    };
}

pub(crate) use subsystem_new;
