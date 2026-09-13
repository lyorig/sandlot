use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

use sdl3_sys::video::*;

use crate::{Result, init, properties::Properties, rect::PointI32, resource::Ref, window::Window};

const CREATE_PROPERTIES: [*const c_char; 26] = [
    SDL_PROP_WINDOW_CREATE_ALWAYS_ON_TOP_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_BORDERLESS_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_CONSTRAIN_POPUP_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_EXTERNAL_GRAPHICS_CONTEXT_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_FOCUSABLE_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_FULLSCREEN_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_HEIGHT_NUMBER,
    SDL_PROP_WINDOW_CREATE_HIDDEN_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_HIGH_PIXEL_DENSITY_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_MAXIMIZED_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_MENU_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_METAL_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_MINIMIZED_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_MODAL_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_MOUSE_GRABBED_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_OPENGL_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_PARENT_POINTER,
    SDL_PROP_WINDOW_CREATE_RESIZABLE_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_TITLE_STRING,
    SDL_PROP_WINDOW_CREATE_TRANSPARENT_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_TOOLTIP_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_UTILITY_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_VULKAN_BOOLEAN,
    SDL_PROP_WINDOW_CREATE_WIDTH_NUMBER,
    SDL_PROP_WINDOW_CREATE_X_NUMBER,
    SDL_PROP_WINDOW_CREATE_Y_NUMBER,
];

pub struct WindowBuilder<'p, 'parent, 'parent_ctx, 'parent_vid> {
    inner: Ref<'p, Properties>,
    marker: PhantomData<Ref<'parent, Window<'parent_ctx, 'parent_vid>>>,
}

impl<'p, 'parent, 'parent_ctx, 'parent_vid> WindowBuilder<'p, 'parent, 'parent_ctx, 'parent_vid> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// True if the window should be always on top.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_ALWAYS_ON_TOP_BOOLEAN")]
    pub fn always_on_top(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_ALWAYS_ON_TOP_BOOLEAN, value)
    }

    /// True if the window has no window decoration.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_BORDERLESS_BOOLEAN")]
    pub fn borderless(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_BORDERLESS_BOOLEAN, value)
    }

    /// True if the "tooltip" and "menu" window types should be automatically
    /// constrained to be entirely within display bounds (default), false if
    /// no constraints on the position are desired.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_CONSTRAIN_POPUP_BOOLEAN")]
    pub fn constrain_popup(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_CONSTRAIN_POPUP_BOOLEAN, value)
    }

    /// True if the window will be used with an externally managed graphics context.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_EXTERNAL_GRAPHICS_CONTEXT_BOOLEAN")]
    pub fn ext_gfx_context(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_WINDOW_CREATE_EXTERNAL_GRAPHICS_CONTEXT_BOOLEAN,
            value,
        )
    }

    /// True if the window should accept keyboard input (defaults to true).
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_FOCUSABLE_BOOLEAN")]
    pub fn focusable(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_FOCUSABLE_BOOLEAN, value)
    }

    /// True if the window should start in fullscreen mode at desktop resolution.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_FULLSCREEN_BOOLEAN")]
    pub fn fullscreen(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_FULLSCREEN_BOOLEAN, value)
    }

    /// The height of the window.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_HEIGHT_NUMBER")]
    pub fn height(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_WINDOW_CREATE_HEIGHT_NUMBER, value)
    }

    /// True if the window should start hidden.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_HIDDEN_BOOLEAN")]
    pub fn hidden(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_HIDDEN_BOOLEAN, value)
    }

    /// True if the window uses a high pixel density buffer if possible.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_HIGH_PIXEL_DENSITY_BOOLEAN")]
    pub fn high_pixel_density(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_HIGH_PIXEL_DENSITY_BOOLEAN, value)
    }

    /// True if the window should start maximized.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_MAXIMIZED_BOOLEAN")]
    pub fn maximized(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_MAXIMIZED_BOOLEAN, value)
    }

    /// True if the window is a popup menu.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_MENU_BOOLEAN")]
    pub fn menu(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_MENU_BOOLEAN, value)
    }

    /// True if the window will be used with Metal rendering.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_METAL_BOOLEAN")]
    pub fn metal(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_METAL_BOOLEAN, value)
    }

    /// True if the window should start minimized.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_MINIMIZED_BOOLEAN")]
    pub fn minimized(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_MINIMIZED_BOOLEAN, value)
    }

    /// True if the window is modal to its parent.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_MODAL_BOOLEAN")]
    pub fn modal(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_MODAL_BOOLEAN, value)
    }

    /// True if the window starts with grabbed mouse focus.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_MOUSE_GRABBED_BOOLEAN")]
    pub fn mouse_grabbed(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_MOUSE_GRABBED_BOOLEAN, value)
    }

    /// True if the window will be used with OpenGL rendering.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_OPENGL_BOOLEAN")]
    pub fn opengl(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_OPENGL_BOOLEAN, value)
    }

    /// A [`Window`] that will be the parent of this window. Required for
    /// windows with the "tooltip", "menu", and "modal" properties.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_PARENT_POINTER")]
    pub fn parent(&mut self, value: Ref<'parent, Window>) -> &mut Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_WINDOW_CREATE_PARENT_POINTER,
                value.handle.as_ptr().cast(),
            )
        };
        self
    }

    /// True if the window should be resizable.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_RESIZABLE_BOOLEAN")]
    pub fn resizable(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_RESIZABLE_BOOLEAN, value)
    }

    /// The title of the window, in UTF-8 encoding.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_TITLE_STRING")]
    pub fn title(&mut self, value: &CStr) -> &mut Self {
        _ = unsafe {
            self.inner
                .set_string(SDL_PROP_WINDOW_CREATE_TITLE_STRING, value.as_ptr())
        };
        self
    }

    /// True if the window is transparent in the areas with an alpha of 0.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_TRANSPARENT_BOOLEAN")]
    pub fn transparent(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_TRANSPARENT_BOOLEAN, value)
    }

    /// True if the window is a tooltip.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_TOOLTIP_BOOLEAN")]
    pub fn tooltip(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_TOOLTIP_BOOLEAN, value)
    }

    /// True if the window is a utility window, not showing in the task bar and
    /// window list.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_UTILITY_BOOLEAN")]
    pub fn utility(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_UTILITY_BOOLEAN, value)
    }

    /// True if the window will be used with Vulkan rendering.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_VULKAN_BOOLEAN")]
    pub fn vulkan(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_WINDOW_CREATE_VULKAN_BOOLEAN, value)
    }

    /// The width of the window.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_WIDTH_NUMBER")]
    pub fn width(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_WINDOW_CREATE_WIDTH_NUMBER, value)
    }

    /// The x position of the window, or `SDL_WINDOWPOS_CENTERED`. Defaults to
    /// `SDL_WINDOWPOS_UNDEFINED`. Relative to the parent for windows with the
    /// "tooltip" or "menu" property set.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_X_NUMBER")]
    pub fn x(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_WINDOW_CREATE_X_NUMBER, value)
    }

    /// The y position of the window, or `SDL_WINDOWPOS_CENTERED`. Defaults to
    /// `SDL_WINDOWPOS_UNDEFINED`. Relative to the parent for windows with the
    /// "tooltip" or "menu" property set.
    #[doc(alias = "SDL_PROP_WINDOW_CREATE_Y_NUMBER")]
    pub fn y(&mut self, value: i64) -> &mut Self {
        self.set_number(SDL_PROP_WINDOW_CREATE_Y_NUMBER, value)
    }

    /// Utility method that calls `self.width()` and `self.height()`.
    pub fn size(&mut self, size: PointI32) -> &mut Self {
        self.width(size.x.into());
        self.height(size.y.into())
    }

    /// Utility method that calls `self.x()` and `self.y()`.
    pub fn position(&mut self, pos: PointI32) -> &mut Self {
        self.x(pos.x.into());
        self.y(pos.y.into())
    }

    /// Clear all window creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the window.
    #[doc(alias = "SDL_CreateWindowWithProperties")]
    pub fn build<'ctx, 'vid>(
        &self,
        _video: init::Ref<'vid, init::Video<'ctx>>,
    ) -> Result<Window<'ctx, 'vid>> {
        Window::from_ptr(unsafe { SDL_CreateWindowWithProperties(self.inner.id()) })
    }

    /// Build the window, and cleanup all properties.
    /// See the [crate::properties] module docs for more info.
    #[doc(alias = "SDL_CreateWindowWithProperties")]
    pub fn build_cleanup<'ctx, 'vid>(
        &self,
        _video: init::Ref<'vid, init::Video<'ctx>>,
    ) -> Result<Window<'ctx, 'vid>> {
        let res = Window::from_ptr(unsafe { SDL_CreateWindowWithProperties(self.inner.id()) });
        Self::clear_from(self.inner);
        res
    }

    fn set_bool(&mut self, key: *const c_char, value: bool) -> &mut Self {
        _ = unsafe { self.inner.set_bool(key, value) };
        self
    }

    fn set_number(&mut self, key: *const c_char, value: i64) -> &mut Self {
        _ = unsafe { self.inner.set_number(key, value) };
        self
    }
}
