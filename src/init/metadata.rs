use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

use sdl3_sys::init::*;

use crate::init::{AppKind, Context};

/// Read-only app metadata, as documented by [`SDL_GetAppMetadataProperty`].
///
/// The metadata lives in [`Properties::global`](crate::properties::Properties::global), so this type is zero-sized
/// and only borrows the [`Context`] it was obtained from.
#[doc(alias = "SDL_GetAppMetadataProperty")]
#[derive(Clone, Copy)]
pub struct ContextMetadata<'ctx> {
    marker: PhantomData<&'ctx Context>,
}

impl<'ctx> ContextMetadata<'ctx> {
    pub(super) fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }

    fn opt_str(&self, key: *const c_char) -> Option<&'ctx CStr> {
        let meta = unsafe { SDL_GetAppMetadataProperty(key) };

        if meta.is_null() {
            None
        } else {
            let cs = unsafe { CStr::from_ptr(meta) };
            Some(cs)
        }
    }

    /// The human-readable name of the application.
    ///
    /// Defaults to the application binary's name, or "SDL Application" if that isn't available.
    pub fn name(&self) -> &'ctx CStr {
        let opt = self.opt_str(SDL_PROP_APP_METADATA_NAME_STRING);

        // SAFETY: Only called for properties whose existence the SDL docs guarantee.
        unsafe { opt.unwrap_unchecked() }
    }

    /// The version of the app that is running.
    pub fn version(&self) -> Option<&'ctx CStr> {
        self.opt_str(SDL_PROP_APP_METADATA_VERSION_STRING)
    }

    /// A unique string that identifies this app, in reverse-domain format.
    pub fn identifier(&self) -> Option<&'ctx CStr> {
        self.opt_str(SDL_PROP_APP_METADATA_IDENTIFIER_STRING)
    }

    /// The human-readable name of the creator/developer/maker of this app.
    pub fn creator(&self) -> Option<&'ctx CStr> {
        self.opt_str(SDL_PROP_APP_METADATA_CREATOR_STRING)
    }

    /// The human-readable copyright notice.
    pub fn copyright(&self) -> Option<&'ctx CStr> {
        self.opt_str(SDL_PROP_APP_METADATA_COPYRIGHT_STRING)
    }

    /// A URL to the app on the web.
    pub fn url(&self) -> Option<&'ctx CStr> {
        self.opt_str(SDL_PROP_APP_METADATA_URL_STRING)
    }

    /// The kind of application this is.
    ///
    /// Defaults to [`AppKind::Application`].
    pub fn kind(&self) -> AppKind {
        let cs = self.opt_str(SDL_PROP_APP_METADATA_TYPE_STRING);

        unsafe {
            // SAFETY: SDL guarantees this property to be a valid `AppKind`.
            AppKind::from_sdl(cs.unwrap_unchecked()).unwrap_unchecked()
        }
    }
}
