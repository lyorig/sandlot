use std::{
    ffi::{c_char, c_void},
    marker::PhantomData,
    ptr::NonNull,
};

use sdl3_sys::video::*;

use crate::{
    properties::Properties,
    resource::Ref,
    str::Str,
    surface::{Surface, SurfaceHandle},
    window::Window,
};

#[expect(unused_imports)] // doc-only
use crate::{event::Event, pixels::Colorspace};

/// Read-only properties of a window, as documented by
/// [`SDL_GetWindowProperties`](https://wiki.libsdl.org/SDL3/SDL_GetWindowProperties).
///
/// Generic properties are returned bare since the docs guarantee their
/// existence; backend properties are returned as `Option` since they only
/// exist on their respective backends.
#[derive(Clone, Copy)]
pub struct WindowProperties<'ctx, 'vid, 'wnd> {
    inner: Ref<'wnd, Properties>,
    marker: PhantomData<Ref<'wnd, Window<'ctx, 'vid>>>,
}

impl<'ctx, 'vid, 'wnd> WindowProperties<'ctx, 'vid, 'wnd> {
    pub(super) fn new(inner: Ref<'wnd, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    fn opt_str(self, key: *const c_char) -> Option<&'wnd Str> {
        let s = unsafe { self.inner.string(key, std::ptr::null()) };

        (!s.is_null()).then(|| unsafe { Str::from_ptr(s) })
    }

    fn opt_number(self, key: *const c_char) -> Option<i64> {
        unsafe { self.inner.has(key).then(|| self.inner.number(key, 0)) }
    }

    fn opt_ptr(self, key: *const c_char) -> Option<NonNull<c_void>> {
        let p = unsafe { self.inner.pointer(key, std::ptr::null_mut()) };
        NonNull::new(p)
    }

    /// Returns the surface associated with a shaped window.
    #[doc(alias = "SDL_PROP_WINDOW_SHAPE_POINTER")]
    pub fn shape(self) -> Option<Ref<'wnd, Surface>> {
        let p = unsafe {
            self.inner
                .pointer(SDL_PROP_WINDOW_SHAPE_POINTER, std::ptr::null_mut())
        };

        SurfaceHandle::from_ptr(p.cast()).map(|h| unsafe { Ref::from_handle(h) })
    }

    /// Returns whether the window has HDR headroom above the SDR white point.
    ///
    /// This value can change when [`Event::WindowHdrStateChanged`] is
    /// sent.
    #[doc(alias = "SDL_PROP_WINDOW_HDR_ENABLED_BOOLEAN")]
    pub fn hdr_enabled(self) -> bool {
        unsafe { self.inner.bool(SDL_PROP_WINDOW_HDR_ENABLED_BOOLEAN, false) }
    }

    /// Returns the value of SDR white in the [`Colorspace::SrgbLinear`]
    /// colorspace.
    ///
    /// On Windows, this is the SDR white level in scRGB. On Apple platforms,
    /// this is always `1.0` for EDR content. The value can change when
    /// [`Event::WindowHdrStateChanged`] is sent.
    #[doc(alias = "SDL_PROP_WINDOW_SDR_WHITE_LEVEL_FLOAT")]
    pub fn sdr_white_level(self) -> f32 {
        unsafe { self.inner.float(SDL_PROP_WINDOW_SDR_WHITE_LEVEL_FLOAT, 0.) }
    }

    /// Returns the additional high dynamic range the window can display,
    /// relative to the SDR white point.
    ///
    /// This is `1.0` when HDR is not enabled. The value can change when
    /// [`Event::WindowHdrStateChanged`] is sent.
    #[doc(alias = "SDL_PROP_WINDOW_HDR_HEADROOM_FLOAT")]
    pub fn hdr_headroom(self) -> f32 {
        unsafe { self.inner.float(SDL_PROP_WINDOW_HDR_HEADROOM_FLOAT, 0.) }
    }

    /// (macOS only) Returns the `NSWindow` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_COCOA_WINDOW_POINTER")]
    pub fn cocoa_window(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_COCOA_WINDOW_POINTER)
    }

    /// (macOS only) Returns the `NSInteger` tag associated with Metal views on the window.
    #[doc(alias = "SDL_PROP_WINDOW_COCOA_METAL_VIEW_TAG_NUMBER")]
    pub fn cocoa_metal_view_tag(self) -> Option<i64> {
        self.opt_number(SDL_PROP_WINDOW_COCOA_METAL_VIEW_TAG_NUMBER)
    }

    /// (Windows only) Returns the `HWND` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WIN32_HWND_POINTER")]
    pub fn win32_hwnd(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WIN32_HWND_POINTER)
    }

    /// (Windows only) Returns the `HDC` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WIN32_HDC_POINTER")]
    pub fn win32_hdc(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WIN32_HDC_POINTER)
    }

    /// (Windows only) Returns the `HINSTANCE` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WIN32_INSTANCE_POINTER")]
    pub fn win32_instance(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WIN32_INSTANCE_POINTER)
    }

    /// (X11 only) Returns the `Display` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_X11_DISPLAY_POINTER")]
    pub fn x11_display(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_X11_DISPLAY_POINTER)
    }

    /// (X11 only) Returns the screen number associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_X11_SCREEN_NUMBER")]
    pub fn x11_screen(self) -> Option<i64> {
        self.opt_number(SDL_PROP_WINDOW_X11_SCREEN_NUMBER)
    }

    /// (X11 only) Returns the window identifier associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_X11_WINDOW_NUMBER")]
    pub fn x11_window(self) -> Option<i64> {
        self.opt_number(SDL_PROP_WINDOW_X11_WINDOW_NUMBER)
    }

    /// (Wayland only) Returns the `wl_display` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER")]
    pub fn wayland_display(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER)
    }

    /// (Wayland only) Returns the `wl_surface` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER")]
    pub fn wayland_surface(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER)
    }

    /// (Wayland only) Returns the `wp_viewport` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_VIEWPORT_POINTER")]
    pub fn wayland_viewport(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_VIEWPORT_POINTER)
    }

    /// (Wayland only) Returns the `wl_egl_window` associated with the window.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_EGL_WINDOW_POINTER")]
    pub fn wayland_egl_window(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_EGL_WINDOW_POINTER)
    }

    /// (Wayland only) Returns the `xdg_surface` associated with the window.
    ///
    /// XDG window objects do not persist across show/hide calls. This returns
    /// [`None`] while the window is hidden and must be queried again each time
    /// the window is shown.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_XDG_SURFACE_POINTER")]
    pub fn wayland_xdg_surface(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_XDG_SURFACE_POINTER)
    }

    /// (Wayland only) Returns the `xdg_toplevel` role associated with the window.
    ///
    /// XDG window objects do not persist across show/hide calls. This returns
    /// [`None`] while the window is hidden and must be queried again each time
    /// the window is shown.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_XDG_TOPLEVEL_POINTER")]
    pub fn wayland_xdg_toplevel(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_XDG_TOPLEVEL_POINTER)
    }

    /// (Wayland only) Returns the export handle associated with the window.
    ///
    /// XDG window objects do not persist across show/hide calls. This returns
    /// [`None`] while the window is hidden and must be queried again each time
    /// the window is shown.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_XDG_TOPLEVEL_EXPORT_HANDLE_STRING")]
    pub fn wayland_xdg_toplevel_export_handle(self) -> Option<&'wnd Str> {
        self.opt_str(SDL_PROP_WINDOW_WAYLAND_XDG_TOPLEVEL_EXPORT_HANDLE_STRING)
    }

    /// (Wayland only) Returns the `xdg_popup` role associated with the window.
    ///
    /// XDG window objects do not persist across show/hide calls. This returns
    /// [`None`] while the window is hidden and must be queried again each time
    /// the window is shown.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_XDG_POPUP_POINTER")]
    pub fn wayland_xdg_popup(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_XDG_POPUP_POINTER)
    }

    /// (Wayland only) Returns the `xdg_positioner` associated with the window in popup mode.
    ///
    /// XDG window objects do not persist across show/hide calls. This returns
    /// [`None`] while the window is hidden and must be queried again each time
    /// the window is shown.
    #[doc(alias = "SDL_PROP_WINDOW_WAYLAND_XDG_POSITIONER_POINTER")]
    pub fn wayland_xdg_positioner(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_WINDOW_WAYLAND_XDG_POSITIONER_POINTER)
    }
}
