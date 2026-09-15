use std::ffi::c_char;

use sdl3_ttf_sys::ttf::*;

use crate::{Result, gpu::Device, properties::Properties, renderer::Renderer, resource::Ref};

use super::{GpuEngine, RendererEngine};

const GPU_CREATE_PROPERTIES: [*const c_char; 2] = [
    TTF_PROP_GPU_TEXT_ENGINE_DEVICE,
    TTF_PROP_GPU_TEXT_ENGINE_ATLAS_TEXTURE_SIZE,
];

const RENDERER_CREATE_PROPERTIES: [*const c_char; 2] = [
    TTF_PROP_RENDERER_TEXT_ENGINE_RENDERER,
    TTF_PROP_RENDERER_TEXT_ENGINE_ATLAS_TEXTURE_SIZE,
];

/// Builder for [`GpuEngine`], using
/// [`TTF_CreateGPUTextEngineWithProperties`](https://wiki.libsdl.org/SDL3_ttf/TTF_CreateGPUTextEngineWithProperties).
#[derive(Clone, Copy)]
pub struct GpuEngineBuilder<'p> {
    inner: Ref<'p, Properties>,
}

impl<'p> GpuEngineBuilder<'p> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self { inner }
    }

    /// The GPU device used to create textures and draw text.
    #[doc(alias = "TTF_PROP_GPU_TEXT_ENGINE_DEVICE")]
    fn device(self, value: Ref<Device>) -> Self {
        self.set_pointer(TTF_PROP_GPU_TEXT_ENGINE_DEVICE, value.as_raw().cast());
        self
    }

    /// The size of the texture atlas used by the text engine.
    #[doc(alias = "TTF_PROP_GPU_TEXT_ENGINE_ATLAS_TEXTURE_SIZE")]
    pub fn atlas_texture_size(self, value: i64) -> Self {
        self.set_number(TTF_PROP_GPU_TEXT_ENGINE_ATLAS_TEXTURE_SIZE, value);
        self
    }

    /// Clear all GPU text engine creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in GPU_CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the GPU text engine.
    #[doc(alias = "TTF_CreateGPUTextEngineWithProperties")]
    pub fn build<'ctx, 'vid, 'dev>(
        self,
        dev: Ref<'dev, Device<'ctx, 'vid>>,
    ) -> Result<GpuEngine<'ctx, 'vid, 'dev>> {
        self.device(dev);
        GpuEngine::from_ptr(unsafe { TTF_CreateGPUTextEngineWithProperties(self.inner.id()) })
    }

    /// Build the GPU text engine, and clean up its creation properties.
    /// See the [`crate::properties`] module docs for more information.
    #[doc(alias = "TTF_CreateGPUTextEngineWithProperties")]
    pub fn build_cleanup<'ctx, 'vid, 'dev>(
        self,
        dev: Ref<'dev, Device<'ctx, 'vid>>,
    ) -> Result<GpuEngine<'ctx, 'vid, 'dev>> {
        let result = self.build(dev);
        Self::clear_from(self.inner);
        result
    }

    fn set_pointer(self, key: *const c_char, value: *mut std::ffi::c_void) {
        _ = unsafe { self.inner.set_pointer(key, value) };
    }

    fn set_number(self, key: *const c_char, value: i64) {
        _ = unsafe { self.inner.set_number(key, value) };
    }
}

/// Builder for [`RendererEngine`], using
/// [`TTF_CreateRendererTextEngineWithProperties`](https://wiki.libsdl.org/SDL3_ttf/TTF_CreateRendererTextEngineWithProperties).
#[derive(Clone, Copy)]
pub struct RendererEngineBuilder<'p> {
    inner: Ref<'p, Properties>,
}

impl<'p> RendererEngineBuilder<'p> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self { inner }
    }

    /// The renderer used to create textures and draw text.
    #[doc(alias = "TTF_PROP_RENDERER_TEXT_ENGINE_RENDERER")]
    fn renderer(self, value: Ref<Renderer>) -> Self {
        self.set_pointer(
            TTF_PROP_RENDERER_TEXT_ENGINE_RENDERER,
            value.as_raw().cast(),
        );
        self
    }

    /// The size of the texture atlas used by the text engine.
    #[doc(alias = "TTF_PROP_RENDERER_TEXT_ENGINE_ATLAS_TEXTURE_SIZE")]
    pub fn atlas_texture_size(self, value: i64) -> Self {
        self.set_number(TTF_PROP_RENDERER_TEXT_ENGINE_ATLAS_TEXTURE_SIZE, value);
        self
    }

    /// Clear all renderer text engine creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in RENDERER_CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the renderer text engine.
    #[doc(alias = "TTF_CreateRendererTextEngineWithProperties")]
    pub fn build<'ctx, 'vid, 'wnd, 'rnd>(
        self,
        rnd: Ref<'rnd, Renderer<'ctx, 'vid, 'wnd>>,
    ) -> Result<RendererEngine<'ctx, 'vid, 'wnd, 'rnd>> {
        self.renderer(rnd);
        RendererEngine::from_ptr(unsafe {
            TTF_CreateRendererTextEngineWithProperties(self.inner.id())
        })
    }

    /// Build the renderer text engine, and clean up its creation properties.
    /// See the [`crate::properties`] module docs for more information.
    #[doc(alias = "TTF_CreateRendererTextEngineWithProperties")]
    pub fn build_cleanup<'ctx, 'vid, 'wnd, 'rnd>(
        self,
        rnd: Ref<'rnd, Renderer<'ctx, 'vid, 'wnd>>,
    ) -> Result<RendererEngine<'ctx, 'vid, 'wnd, 'rnd>> {
        let result = self.build(rnd);
        Self::clear_from(self.inner);
        result
    }

    fn set_pointer(self, key: *const c_char, value: *mut std::ffi::c_void) {
        _ = unsafe { self.inner.set_pointer(key, value) };
    }

    fn set_number(self, key: *const c_char, value: i64) {
        _ = unsafe { self.inner.set_number(key, value) };
    }
}
