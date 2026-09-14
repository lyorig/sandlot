use std::ffi::{CStr, c_char};

use sdl3_sys::init::*;

use crate::init::Context;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppKind {
    /// A video game.
    Game,
    /// A media player.
    MediaPlayer,
    /// A generic application.
    Application,
}

impl AppKind {
    /// Try to parse an [`AppKind`] from the format expected by [`SDL_PROP_APP_METADATA_TYPE_STRING`].
    ///
    /// Returns [`None`] if the value is not recognized.
    pub const fn from_sdl(value: &CStr) -> Option<Self> {
        match value.to_bytes() {
            b"game" => Some(AppKind::Game),
            b"mediaplayer" => Some(AppKind::MediaPlayer),
            b"application" => Some(AppKind::Application),
            _ => None,
        }
    }

    /// Get a string representation of this app kind in the format expected by [`SDL_PROP_APP_METADATA_TYPE_STRING`].
    pub const fn to_sdl(self) -> &'static CStr {
        match self {
            AppKind::Game => c"game",
            AppKind::MediaPlayer => c"mediaplayer",
            AppKind::Application => c"application",
        }
    }
}

pub struct ContextBuilder;

impl ContextBuilder {
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// # Panics
    ///
    /// This method simply calls [`Context::new`]. Consult its documentation for panic conditions.
    pub fn build(self) -> Context {
        Context::new()
    }

    fn set(self, key: *const c_char, value: &CStr) -> Self {
        unsafe { SDL_SetAppMetadataProperty(key, value.as_ptr()) };
        self
    }

    /// The human-readable name of the application, like "My Game 2: Bad Guy's Revenge!".
    ///
    /// This will show up anywhere the OS shows the name of the application separately from window titles,
    /// such as volume control applets, etc.
    ///
    /// This defaults to the application's binary name, or "SDL Application" if that isn't available.
    pub fn name(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_NAME_STRING, value)
    }

    /// The version of the app that is running.
    ///
    /// There are no rules on format, so "1.0.3beta2" and "April 22nd, 2024"
    /// and a git hash are all valid options.
    ///
    /// This has no default.
    pub fn version(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_VERSION_STRING, value)
    }

    /// A unique string that identifies this app.
    ///
    /// This must be in reverse-domain format, like "com.example.mygame2".
    /// This string is used by desktop compositors to identify and group windows together,
    /// as well as match applications with associated desktop settings and icons.
    ///
    /// If you plan to package your application in a container such as Flatpak,
    /// the app ID should match the name of your Flatpak container as well.
    ///
    /// This has no default.
    pub fn identifier(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_IDENTIFIER_STRING, value)
    }

    /// The human-readable name of the creator/developer/maker of this app, like "MojoWorkshop, LLC".
    ///
    /// This has no default.
    pub fn creator(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_CREATOR_STRING, value)
    }

    /// The human-readable copyright notice, like "Copyright (c) 2024 MojoWorkshop, LLC" or whatnot.
    ///
    /// Keep this to one line, don't paste a copy of a whole software license in here.
    ///
    /// This has no default.
    pub fn copyright(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_COPYRIGHT_STRING, value)
    }

    /// A URL to the app on the web.
    ///
    /// Maybe a product page, or a storefront, or even a GitHub repository, for user's further information.
    ///
    /// This has no default.
    pub fn url(self, value: &CStr) -> Self {
        self.set(SDL_PROP_APP_METADATA_URL_STRING, value)
    }

    /// The kind of application this is.
    ///
    /// Internally, this string can be "game" for a video game, "mediaplayer" for a media player,
    /// or generically "application" if nothing else applies. Future versions of SDL might add new kinds.
    ///
    /// Sandlot will keep the [`AppKind`] enum in sync with the SDL-provided types.
    ///
    /// Defaults to [`AppKind::Application`].
    pub fn kind(self, value: AppKind) -> Self {
        self.set(SDL_PROP_APP_METADATA_TYPE_STRING, value.to_sdl())
    }
}
