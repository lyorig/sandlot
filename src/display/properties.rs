use std::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use sdl3_sys::video::*;

use crate::{display::Display, properties::Properties, resource::Ref};

#[derive(Clone, Copy)]
pub struct DisplayProperties<'ctx, 'vid, 'disp> {
    inner: Ref<'disp, Properties>,
    marker: PhantomData<&'disp Display<'ctx, 'vid>>,
}

impl<'ctx, 'vid, 'disp> DisplayProperties<'ctx, 'vid, 'disp> {
    pub(super) fn new(inner: Ref<'disp, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Whether the display has HDR headroom above the SDR white point. This is
    /// for informational and diagnostic purposes only, as not all platforms
    /// provide this information at the display level.
    #[doc(alias = "SDL_PROP_DISPLAY_HDR_ENABLED_BOOLEAN")]
    pub fn hdr_enabled(self) -> bool {
        unsafe { self.inner.bool(SDL_PROP_DISPLAY_HDR_ENABLED_BOOLEAN, false) }
    }

    /// (KMS/DRM only) The "panel orientation" property for the display in degrees
    /// of clockwise rotation. Note that this is provided only as a hint, and the
    /// application is responsible for any coordinate transformations needed to conform
    /// to the requested display orientation.
    #[doc(alias = "SDL_PROP_DISPLAY_KMSDRM_PANEL_ORIENTATION")]
    pub fn kmsdrm_panel_orientation(self) -> Option<i64> {
        const SENTINEL: i64 = i64::MAX;

        let val = unsafe {
            self.inner
                .number(SDL_PROP_DISPLAY_KMSDRM_PANEL_ORIENTATION_NUMBER, SENTINEL)
        };

        (val != SENTINEL).then_some(val)
    }

    /// (Wayland only) The `wl_output` associated with the display.
    #[doc(alias = "SDL_PROP_DISPLAY_WAYLAND_WL_OUTPUT_POINTER")]
    pub fn wayland_wl_output(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_DISPLAY_WAYLAND_WL_OUTPUT_POINTER)
    }

    /// (Windows only) The `HMONITOR` associated with the display.
    #[doc(alias = "SDL_PROP_DISPLAY_WINDOWS_HMONITOR_POINTER")]
    pub fn windows_hmonitor(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_DISPLAY_WINDOWS_HMONITOR_POINTER)
    }

    fn opt_ptr(self, key: *const i8) -> Option<NonNull<c_void>> {
        let ptr = unsafe { self.inner.pointer(key, std::ptr::null_mut()) };
        NonNull::new(ptr)
    }
}
