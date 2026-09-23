use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

use sdl3_sys::render::*;

use crate::{
    Result, pixels::Colorspace, properties::Properties, renderer::Renderer, resource::Ref,
    surface::Surface, window::Window,
};

#[expect(unused_imports)]
use crate::renderer::RendererHandle;

const CREATE_PROPERTIES: [*const c_char; 5] = [
    SDL_PROP_RENDERER_CREATE_NAME_STRING,
    SDL_PROP_RENDERER_CREATE_WINDOW_POINTER,
    SDL_PROP_RENDERER_CREATE_SURFACE_POINTER,
    SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER,
    SDL_PROP_RENDERER_CREATE_PRESENT_VSYNC_NUMBER,
];

#[derive(Clone, Copy)]
pub struct RendererBuilder<'p, 'ctx, 'vid, 'wnd, 'surf> {
    inner: Ref<'p, Properties>,
    marker_wnd: PhantomData<Ref<'wnd, Window<'ctx, 'vid>>>,
    marker_surf: PhantomData<Ref<'surf, Surface>>,
}

impl<'p, 'ctx, 'vid, 'wnd, 'surf> RendererBuilder<'p, 'ctx, 'vid, 'wnd, 'surf> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self {
            inner,
            marker_wnd: PhantomData,
            marker_surf: PhantomData,
        }
    }

    /// The name of the rendering driver to use, if a specific one is desired.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_NAME_STRING")]
    pub fn name(self, value: &CStr) -> Self {
        _ = unsafe {
            self.inner
                .set_string(SDL_PROP_RENDERER_CREATE_NAME_STRING, value.as_ptr())
        };

        self
    }

    /// The window where rendering is displayed. Required if this isn't a
    /// software renderer using a surface. Mutually exclusive with
    /// [`RendererBuilder::surface`].
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_WINDOW_POINTER")]
    pub fn window(self, value: Ref<'wnd, Window>) -> Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_RENDERER_CREATE_WINDOW_POINTER,
                value.as_raw().cast(),
            )
        };

        self
    }

    /// The surface where rendering is displayed, if you want a software
    /// renderer without a window.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_SURFACE_POINTER")]
    pub fn surface(self, value: Ref<'surf, Surface>) -> Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_RENDERER_CREATE_SURFACE_POINTER,
                value.as_raw().cast(),
            )
        };

        self
    }

    /// A [`Colorspace`] value describing the colorspace for output to the
    /// display. Defaults to [`Colorspace::Srgb`].
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER")]
    pub fn colorspace(self, value: Colorspace) -> Self {
        _ = unsafe {
            self.inner.set_number(
                SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER,
                value.to_sdl().0.into(),
            )
        };

        self
    }

    /// Non-zero if you want present synchronized with the refresh rate. This
    /// property can take any value that is supported by [`RendererHandle::set_vsync`]
    /// for the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_PRESENT_VSYNC_NUMBER")]
    pub fn vsync(self, value: i64) -> Self {
        _ = unsafe {
            self.inner
                .set_number(SDL_PROP_RENDERER_CREATE_PRESENT_VSYNC_NUMBER, value)
        };

        self
    }

    /// Clear all renderer creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the renderer.
    ///
    /// This doesn't require a subsystem parameter, as the [`Window`]
    /// you're creating this with needs one, proving the subsystem has been
    /// initialized.
    #[doc(alias = "SDL_CreateRendererWithProperties")]
    pub fn build(self) -> Result<Renderer<'ctx, 'vid, 'wnd>> {
        Renderer::from_raw(unsafe { SDL_CreateRendererWithProperties(self.inner.as_raw()) })
    }

    /// Build the renderer, and cleanup all properties.
    /// See the [crate::properties] module docs for more info.
    ///
    /// This doesn't require a subsystem parameter, as the [`Window`]
    /// you're creating this with needs one, proving the subsystem has been
    /// initialized.
    #[doc(alias = "SDL_CreateRendererWithProperties")]
    pub fn build_cleanup(self) -> Result<Renderer<'ctx, 'vid, 'wnd>> {
        let res =
            Renderer::from_raw(unsafe { SDL_CreateRendererWithProperties(self.inner.as_raw()) });
        Self::clear_from(self.inner);
        res
    }
}
