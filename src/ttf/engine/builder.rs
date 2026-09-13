use std::{ffi::c_char, marker::PhantomData};

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
pub struct GpuEngineBuilder<'p, 'dev> {
    inner: Ref<'p, Properties>,
    marker: PhantomData<Ref<'dev, Device>>,
}

impl<'p, 'dev> GpuEngineBuilder<'p, 'dev> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// The GPU device used to create textures and draw text.
    #[doc(alias = "TTF_PROP_GPU_TEXT_ENGINE_DEVICE")]
    pub fn device(&mut self, value: Ref<'dev, Device>) -> &mut Self {
        self.set_pointer(
            TTF_PROP_GPU_TEXT_ENGINE_DEVICE,
            value.handle.as_ptr().cast(),
        );
        self
    }

    /// The size of the texture atlas used by the text engine.
    #[doc(alias = "TTF_PROP_GPU_TEXT_ENGINE_ATLAS_TEXTURE_SIZE")]
    pub fn atlas_texture_size(&mut self, value: i64) -> &mut Self {
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
    pub fn build(&self) -> Result<GpuEngine> {
        GpuEngine::from_ptr(unsafe { TTF_CreateGPUTextEngineWithProperties(self.inner.id()) })
    }

    /// Build the GPU text engine, and clean up its creation properties.
    /// See the [`crate::properties`] module docs for more information.
    #[doc(alias = "TTF_CreateGPUTextEngineWithProperties")]
    pub fn build_cleanup(&self) -> Result<GpuEngine> {
        let result = self.build();
        Self::clear_from(self.inner);
        result
    }

    fn set_pointer(&mut self, key: *const c_char, value: *mut std::ffi::c_void) {
        _ = unsafe { self.inner.set_pointer(key, value) };
    }

    fn set_number(&mut self, key: *const c_char, value: i64) {
        _ = unsafe { self.inner.set_number(key, value) };
    }
}

/// Builder for [`RendererEngine`], using
/// [`TTF_CreateRendererTextEngineWithProperties`](https://wiki.libsdl.org/SDL3_ttf/TTF_CreateRendererTextEngineWithProperties).
pub struct RendererEngineBuilder<'p, 'rnd, 'ctx, 'vid, 'wnd> {
    inner: Ref<'p, Properties>,
    marker: PhantomData<Ref<'rnd, Renderer<'ctx, 'vid, 'wnd>>>,
}

impl<'p, 'rnd, 'ctx, 'vid, 'wnd> RendererEngineBuilder<'p, 'rnd, 'ctx, 'vid, 'wnd> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// The renderer used to create textures and draw text.
    #[doc(alias = "TTF_PROP_RENDERER_TEXT_ENGINE_RENDERER")]
    pub fn renderer(&mut self, value: Ref<'rnd, Renderer<'ctx, 'vid, 'wnd>>) -> &mut Self {
        self.set_pointer(
            TTF_PROP_RENDERER_TEXT_ENGINE_RENDERER,
            value.handle.as_ptr().cast(),
        );
        self
    }

    /// The size of the texture atlas used by the text engine.
    #[doc(alias = "TTF_PROP_RENDERER_TEXT_ENGINE_ATLAS_TEXTURE_SIZE")]
    pub fn atlas_texture_size(&mut self, value: i64) -> &mut Self {
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
    pub fn build(&self) -> Result<RendererEngine> {
        RendererEngine::from_ptr(unsafe {
            TTF_CreateRendererTextEngineWithProperties(self.inner.id())
        })
    }

    /// Build the renderer text engine, and clean up its creation properties.
    /// See the [`crate::properties`] module docs for more information.
    #[doc(alias = "TTF_CreateRendererTextEngineWithProperties")]
    pub fn build_cleanup(&self) -> Result<RendererEngine> {
        let result = self.build();
        Self::clear_from(self.inner);
        result
    }

    fn set_pointer(&mut self, key: *const c_char, value: *mut std::ffi::c_void) {
        _ = unsafe { self.inner.set_pointer(key, value) };
    }

    fn set_number(&mut self, key: *const c_char, value: i64) {
        _ = unsafe { self.inner.set_number(key, value) };
    }
}
