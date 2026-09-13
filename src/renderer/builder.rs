use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
};

use sdl3_sys::{pixels::SDL_Colorspace, render::*};

use crate::{
    Result, properties::Properties, renderer::Renderer, resource::Ref, surface::Surface,
    window::Window,
};

const CREATE_PROPERTIES: [*const c_char; 5] = [
    SDL_PROP_RENDERER_CREATE_NAME_STRING,
    SDL_PROP_RENDERER_CREATE_WINDOW_POINTER,
    SDL_PROP_RENDERER_CREATE_SURFACE_POINTER,
    SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER,
    SDL_PROP_RENDERER_CREATE_PRESENT_VSYNC_NUMBER,
];

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
    pub fn name(&mut self, value: &CStr) -> &mut Self {
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
    pub fn window(&mut self, value: Ref<'wnd, Window>) -> &mut Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_RENDERER_CREATE_WINDOW_POINTER,
                value.handle.as_ptr().cast(),
            )
        };

        self
    }

    /// The surface where rendering is displayed, if you want a software
    /// renderer without a window.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_SURFACE_POINTER")]
    pub fn surface(&mut self, value: Ref<'surf, Surface>) -> &mut Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_RENDERER_CREATE_SURFACE_POINTER,
                value.handle.as_ptr().cast(),
            )
        };

        self
    }

    /// An [`SDL_Colorspace`] value describing the colorspace for output to the
    /// display. Defaults to `SDL_COLORSPACE_SRGB`.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER")]
    pub fn colorspace(&mut self, value: SDL_Colorspace) -> &mut Self {
        _ = unsafe {
            self.inner.set_number(
                SDL_PROP_RENDERER_CREATE_OUTPUT_COLORSPACE_NUMBER,
                value.0.into(),
            )
        };

        self
    }

    /// Non-zero if you want present synchronized with the refresh rate. This
    /// property can take any value that is supported by `SDL_SetRenderVSync`
    /// for the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_CREATE_PRESENT_VSYNC_NUMBER")]
    pub fn vsync(&mut self, value: i64) -> &mut Self {
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
    pub fn build(&self) -> Result<Renderer> {
        Renderer::from_ptr(unsafe { SDL_CreateRendererWithProperties(self.inner.id()) })
    }

    /// Build the renderer, and cleanup all properties.
    /// See the [crate::properties] module docs for more info.
    ///
    /// This doesn't require a subsystem parameter, as the [`Window`]
    /// you're creating this with needs one, proving the subsystem has been
    /// initialized.
    #[doc(alias = "SDL_CreateRendererWithProperties")]
    pub fn build_cleanup(&self) -> Result<Renderer> {
        let res = Renderer::from_ptr(unsafe { SDL_CreateRendererWithProperties(self.inner.id()) });
        Self::clear_from(self.inner);
        res
    }
}
