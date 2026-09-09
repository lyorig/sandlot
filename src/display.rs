//! SDL display API.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryVideo)):
//! - [x] SDL_GetClosestFullscreenDisplayMode
//! - [x] SDL_GetCurrentDisplayMode
//! - [x] SDL_GetCurrentDisplayOrientation
//! - [x] SDL_GetDesktopDisplayMode
//! - [x] SDL_GetDisplayBounds
//! - [x] SDL_GetDisplayContentScale
//! - [x] SDL_GetDisplayForPoint
//! - [x] SDL_GetDisplayForRect
//! - [x] SDL_GetDisplayName
//! - [ ] SDL_GetDisplayProperties
//! - [x] SDL_GetDisplays
//! - [x] SDL_GetDisplayUsableBounds
//! - [x] SDL_GetFullscreenDisplayModes
//! - [x] SDL_GetNaturalDisplayOrientation
//! - [x] SDL_GetPrimaryDisplay

use crate::{
    Result, boolenum,
    boxed::Box,
    error::Error,
    impl_enum_transmute,
    rect::{PointI32, RectI32},
    util::opt2res_map,
};

use sdl3_sys::video::*;
use std::{ffi::CStr, mem::MaybeUninit, num::NonZero, ptr::NonNull};

boolenum!(
    IncludeHighDensityModes,
    "Whether to include high-density display modes in enumeration."
);

/// Display orientation values; the way a display is rotated.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum DisplayOrientation {
    /// The display is in landscape mode, with the right side up, relative to
    /// portrait mode.
    Landscape = SDL_DisplayOrientation::LANDSCAPE.0,
    /// The display is in landscape mode, with the left side up, relative to
    /// portrait mode.
    LandscapeFlipped = SDL_DisplayOrientation::LANDSCAPE_FLIPPED.0,
    /// The display is in portrait mode.
    Portrait = SDL_DisplayOrientation::PORTRAIT.0,
    /// The display is in portrait mode, upside down.
    PortraitFlipped = SDL_DisplayOrientation::PORTRAIT_FLIPPED.0,
}

impl_enum_transmute!(SDL_DisplayOrientation, DisplayOrientation);

impl std::fmt::Display for DisplayOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

fn sdl2do(sdl: SDL_DisplayOrientation) -> Option<DisplayOrientation> {
    if sdl == SDL_DisplayOrientation::UNKNOWN {
        None
    } else {
        use std::mem::transmute;

        type Src = SDL_DisplayOrientation;
        type Dst = DisplayOrientation;
        Some(unsafe { transmute::<Src, Dst>(sdl) })
    }
}

/// A handle to a display owned by SDL.
///
/// This is essentially just a number, and since displays can be
/// added and removed at will, member functions mostly return a [`Result`].
///
/// Some SDL display-related functions provide owned data, while others return
/// a pointer to data managed by SDL itself. Due to the aforementioned display
/// invalidation stuff, this would be quite a mess to implement as a reference
/// in Rust (owing to lifetimes), so instead, the raw pointer is provided,
/// offloading the risk to you.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct Display {
    id: NonZero<u32>,
}

impl Display {
    /// Accepts [`NonZero`], since zero is an invalid display ID.
    pub fn new(id: NonZero<u32>) -> Self {
        Self { id }
    }

    pub(crate) fn from_sdl(id: SDL_DisplayID) -> Result<Self> {
        opt2res_map(NonZero::new(id.0), |id| Self { id })
    }

    /// Get a list of currently connected displays.
    #[doc(alias = "SDL_GetDisplays")]
    pub fn all() -> Result<Box<[Self]>> {
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetDisplays(count.as_mut_ptr()) };

        unsafe { Box::from_raw_parts_nullck(ptr.cast(), count.assume_init() as _) }
    }

    /// Return the primary display.
    #[doc(alias = "SDL_GetPrimaryDisplay")]
    pub fn primary() -> Result<Self> {
        Self::from_sdl(unsafe { SDL_GetPrimaryDisplay() })
    }

    /// Get the display containing a point.
    #[doc(alias = "SDL_GetDisplayForPoint")]
    pub fn for_point(point: PointI32) -> Result<Self> {
        Self::from_sdl(unsafe { SDL_GetDisplayForPoint(point.as_sdl_ptr()) })
    }

    /// Get the display primarily containing a rect.
    ///
    /// Returns the display entirely containing the rect, or closest to the
    /// center of the rect.
    #[doc(alias = "SDL_GetDisplayForRect")]
    pub fn for_rect(rect: RectI32) -> Result<Self> {
        Self::from_sdl(unsafe { SDL_GetDisplayForRect(rect.as_sdl_ptr()) })
    }

    /// Returns the "raw" SDL handle type. Intended for interfacing with
    /// SDL display functions.
    pub fn id(&self) -> SDL_DisplayID {
        unsafe { std::mem::transmute(self.id) }
    }

    /// Get the name of a display in UTF-8 encoding.
    ///
    /// The returned string is guaranteed to be valid UTF-8.
    #[doc(alias = "SDL_GetDisplayName")]
    pub fn name(&self) -> Result<&CStr> {
        let ptr = unsafe { SDL_GetDisplayName(self.id()) };
        if ptr.is_null() {
            Err(Error::current())
        } else {
            Ok(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Get the desktop area represented by a display.
    ///
    /// # Remarks
    ///
    /// The primary display is often located at (0,0), but may be placed at a
    /// different location depending on monitor layout.
    #[doc(alias = "SDL_GetDisplayBounds")]
    pub fn bounds(&self) -> Result<RectI32> {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            if SDL_GetDisplayBounds(self.id(), ret.as_mut_ptr()) {
                Ok(std::mem::transmute_copy(ret.assume_init_ref()))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get the usable desktop area represented by a display, in screen
    /// coordinates.
    ///
    /// # Remarks
    ///
    /// This is the same area as [`Display::bounds`] reports, but with
    /// portions reserved by the system removed. For example, on Apple's
    /// macOS, this subtracts the area occupied by the menu bar and dock.
    ///
    /// Setting a window to be fullscreen generally bypasses these unusable
    /// areas, so these are good guidelines for the maximum space available
    /// to a non-fullscreen window.
    #[doc(alias = "SDL_GetDisplayUsableBounds")]
    pub fn usable_bounds(&self) -> Result<RectI32> {
        let mut ret = MaybeUninit::uninit();
        unsafe {
            if SDL_GetDisplayUsableBounds(self.id(), ret.as_mut_ptr()) {
                Ok(std::mem::transmute_copy(ret.assume_init_ref()))
            } else {
                Err(Error::current())
            }
        }
    }

    /// Get information about the current display mode.
    ///
    /// # Remarks
    ///
    /// There's a difference between this function and
    /// [`Display::desktop_mode`] when SDL runs fullscreen and has changed
    /// the resolution. In that case this function will return the current
    /// display mode, and not the previous native display mode.
    ///
    /// The returned pointer is managed by SDL and is owned by the display.
    #[doc(alias = "SDL_GetCurrentDisplayMode")]
    pub fn current_mode(&self) -> Result<NonNull<SDL_DisplayMode>> {
        let ptr = unsafe { SDL_GetCurrentDisplayMode(self.id()) };
        if ptr.is_null() {
            Err(Error::current())
        } else {
            Ok(unsafe { NonNull::new_unchecked(ptr.cast_mut()) })
        }
    }

    /// Get information about the desktop's display mode.
    ///
    /// # Remarks
    ///
    /// There's a difference between this function and
    /// [`Display::current_mode`] when SDL runs fullscreen and has changed
    /// the resolution. In that case this function will return the previous
    /// native display mode, and not the current display mode.
    ///
    /// The returned pointer is managed by SDL and is owned by the display.
    #[doc(alias = "SDL_GetDesktopDisplayMode")]
    pub fn desktop_mode(&self) -> Result<NonNull<SDL_DisplayMode>> {
        let ptr = unsafe { SDL_GetDesktopDisplayMode(self.id()) };
        if ptr.is_null() {
            Err(Error::current())
        } else {
            Ok(unsafe { NonNull::new_unchecked(ptr.cast_mut()) })
        }
    }

    /// Get a list of fullscreen display modes available on a display.
    ///
    /// # Remarks
    ///
    /// The display modes are sorted in this priority:
    ///
    /// - w -> largest to smallest
    /// - h -> largest to smallest
    /// - bits per pixel -> more colors to fewer colors
    /// - packed pixel layout -> largest to smallest
    /// - refresh rate -> highest to lowest
    /// - pixel density -> lowest to highest
    #[doc(alias = "SDL_GetFullscreenDisplayModes")]
    pub fn fullscreen_modes(&self) -> Result<Box<[NonNull<SDL_DisplayMode>]>> {
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe { SDL_GetFullscreenDisplayModes(self.id(), count.as_mut_ptr()) };

        // SAFETY: On success, SDL allocates `count` display modes.
        unsafe { Box::from_raw_parts_nullck(ptr.cast(), count.assume_init() as _) }
    }

    /// Get the content scale of a display.
    ///
    /// # Remarks
    ///
    /// The content scale is the expected scale for content based on the DPI
    /// settings of the display. For example, a 4K display might have a 2.0
    /// (200%) display scale, which means that the user expects UI elements
    /// to be twice as big on this display, to aid in readability.
    ///
    /// After window creation,
    /// [`crate::window::WindowHandle::display_scale`] should be used to query the content
    /// scale factor for individual windows instead of querying the display
    /// for a window and calling this function, as the per-window content
    /// scale factor may differ from the base value of the display it is on,
    /// particularly on high-DPI and/or multi-monitor desktop
    /// configurations.
    #[doc(alias = "SDL_GetDisplayContentScale")]
    pub fn content_scale(&self) -> Result<f32> {
        let ret = unsafe { SDL_GetDisplayContentScale(self.id()) };
        if ret == 0. {
            Err(Error::current())
        } else {
            Ok(ret)
        }
    }

    /// Get the orientation of a display.
    ///
    /// Returns [`None`] if the orientation isn't available.
    #[doc(alias = "SDL_GetCurrentDisplayOrientation")]
    pub fn current_orientation(&self) -> Option<DisplayOrientation> {
        sdl2do(unsafe { SDL_GetCurrentDisplayOrientation(self.id()) })
    }

    /// Get the orientation of a display when it is unrotated.
    ///
    /// Returns [`None`] if the orientation isn't available.
    #[doc(alias = "SDL_GetNaturalDisplayOrientation")]
    pub fn natural_orientation(&self) -> Option<DisplayOrientation> {
        sdl2do(unsafe { SDL_GetNaturalDisplayOrientation(self.id()) })
    }

    /// Get the closest match to the requested display mode.
    ///
    /// `size` is the desired width and height in pixels; `refresh_rate` is
    /// the desired refresh rate, or `0.0` for the desktop refresh rate;
    /// `ihdm` controls whether high density modes are included in the
    /// search.
    ///
    /// Returns the closest display mode equal to or larger than the desired
    /// mode.
    ///
    /// # Remarks
    ///
    /// The available display modes are scanned and the closest mode matching
    /// the requested mode is returned. The mode format and refresh rate
    /// default to the desktop mode if they are set to 0. The modes are
    /// scanned with size being first priority, format being second priority,
    /// and finally checking the refresh rate. If all the available modes are
    /// too small, an error is returned.
    #[doc(alias = "SDL_GetClosestFullscreenDisplayMode")]
    pub fn closest_fullscreen_mode(
        &self,
        size: PointI32,
        refresh_rate: f32,
        ihdm: IncludeHighDensityModes,
    ) -> Result<SDL_DisplayMode> {
        let mut ret = MaybeUninit::uninit();

        unsafe {
            if SDL_GetClosestFullscreenDisplayMode(
                self.id(),
                size.x,
                size.y,
                refresh_rate,
                ihdm.into(),
                ret.as_mut_ptr(),
            ) {
                Ok(ret.assume_init())
            } else {
                Err(Error::current())
            }
        }
    }
}
