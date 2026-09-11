//! Implementation checklist:
//! - [x] TTF_CreateGPUTextEngine
//! - [x] TTF_CreateGPUTextEngineWithProperties
//! - [x] TTF_CreateRendererTextEngine
//! - [x] TTF_CreateRendererTextEngineWithProperties
//! - [x] TTF_CreateSurfaceTextEngine
//! - [x] TTF_DestroyGPUTextEngine
//! - [x] TTF_DestroyRendererTextEngine
//! - [x] TTF_DestroySurfaceTextEngine
//! - [x] TTF_GetGPUTextEngineWinding
//! - [x] TTF_SetGPUTextEngineWinding

use sdl3_ttf_sys::ttf::*;

use crate::util::impl_enum_transmute;
use crate::util::resource_new;
use crate::{
    Result, error::Error, gpu::Device, renderer::Renderer, resource::Ref, util::mod_reexport,
};

mod_reexport!(builder);

/// The winding order of the vertices returned by
/// [`GpuEngineHandle::winding`] drawing data.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "TTF_GPUTextEngineWinding")]
pub enum Winding {
    Clockwise = TTF_GPUTextEngineWinding::CLOCKWISE.0,
    CounterClockwise = TTF_GPUTextEngineWinding::COUNTER_CLOCKWISE.0,
}

impl_enum_transmute!(TTF_GPUTextEngineWinding, Winding, INVALID);

resource_new!(
    /// A text engine that draws text objects with the SDL GPU API.
    TTF_TextEngine, GpuEngine, TTF_DestroyGPUTextEngine
);

impl GpuEngine {
    /// Create a text engine for drawing text with the SDL GPU API.
    ///
    /// `dev` is the GPU device to use for creating textures and drawing text.
    #[doc(alias = "TTF_CreateGPUTextEngine")]
    pub fn new(dev: Ref<Device>) -> Result<Self> {
        Self::from_ptr(unsafe { TTF_CreateGPUTextEngine(dev.as_ptr()) })
    }

    /// Bind the builder to an existing property group.
    pub fn builder(props: Ref<crate::properties::Properties>) -> GpuEngineBuilder {
        GpuEngineBuilder::new(props)
    }
}

impl GpuEngineHandle {
    /// Get the winding order of the vertices returned by
    /// [`TextHandle::gpu_draw_data`](crate::ttf::TextHandle::gpu_draw_data) for this GPU text engine.
    ///
    /// Returns an error in case of failure.
    #[doc(alias = "TTF_GetGPUTextEngineWinding")]
    pub fn winding(&self) -> Result<Winding> {
        let wind = unsafe { TTF_GetGPUTextEngineWinding(self.as_ptr()) };
        Winding::from_sdl(wind).ok_or_else(Error::current)
    }

    /// Set the winding order of the vertices returned by
    /// [`TextHandle::gpu_draw_data`](crate::ttf::TextHandle::gpu_draw_data) for this GPU text engine.
    #[doc(alias = "TTF_SetGPUTextEngineWinding")]
    pub fn set_winding(&self, wind: Winding) {
        unsafe {
            TTF_SetGPUTextEngineWinding(self.as_ptr(), wind.to_sdl());
        }
    }
}

resource_new!(
    /// A text engine that draws text objects to an `SDL_Surface`.
    TTF_TextEngine, SurfaceEngine, TTF_DestroySurfaceTextEngine
);

impl SurfaceEngine {
    /// Create a text engine for drawing text on SDL surfaces.
    #[doc(alias = "TTF_CreateSurfaceTextEngine")]
    pub fn new() -> Result<Self> {
        Self::from_ptr(unsafe { TTF_CreateSurfaceTextEngine() })
    }
}

resource_new!(
    /// A text engine that draws text objects with an SDL 2D renderer.
    TTF_TextEngine,
    RendererEngine,
    TTF_DestroyRendererTextEngine
);

impl RendererEngine {
    /// Create a text engine for drawing text on an SDL renderer.
    ///
    /// `rnd` is the renderer to use for creating textures and drawing text.
    #[doc(alias = "TTF_CreateRendererTextEngine")]
    pub fn new(rnd: Ref<Renderer>) -> Result<Self> {
        Self::from_ptr(unsafe { TTF_CreateRendererTextEngine(rnd.as_ptr()) })
    }

    /// Bind the builder to an existing property group.
    pub fn builder(props: Ref<crate::properties::Properties>) -> RendererEngineBuilder {
        RendererEngineBuilder::new(props)
    }
}
