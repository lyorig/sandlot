use std::{
    ffi::{c_char, c_void},
    marker::PhantomData,
    ptr::NonNull,
};

use sdl3_sys::{
    pixels::{SDL_Colorspace, SDL_PixelFormat},
    render::*,
};

use crate::{
    gpu::{Device, DeviceHandle},
    pixels::{Colorspace, PixelFormat},
    properties::Properties,
    renderer::Renderer,
    resource::Ref,
    str::Str,
    surface::{Surface, SurfaceHandle},
    window::{Window, WindowHandle},
};

#[expect(unused_imports)]
use crate::event::Event;

/// Read-only properties of a renderer, as documented by
/// [`SDL_GetRendererProperties`](https://wiki.libsdl.org/SDL3/SDL_GetRendererProperties).
///
/// Generic properties are returned bare since the docs guarantee their
/// existence; backend properties are returned as `Option` since they only
/// exist on their respective backends.
#[derive(Clone, Copy)]
pub struct RendererProperties<'ctx, 'vid, 'wnd, 'rnd> {
    inner: Ref<'rnd, Properties>,
    marker: PhantomData<Ref<'rnd, Renderer<'ctx, 'vid, 'wnd>>>,
}

impl<'ctx, 'vid, 'wnd, 'rnd> RendererProperties<'ctx, 'vid, 'wnd, 'rnd> {
    pub(super) fn new(inner: Ref<'rnd, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// The name of the rendering driver.
    #[doc(alias = "SDL_PROP_RENDERER_NAME_STRING")]
    pub fn name(self) -> Str<'rnd> {
        self.get_str(SDL_PROP_RENDERER_NAME_STRING)
    }

    /// The window where rendering is displayed, if any.
    #[doc(alias = "SDL_PROP_RENDERER_WINDOW_POINTER")]
    pub fn window(self) -> Option<Ref<'wnd, Window<'ctx, 'vid>>> {
        let p = unsafe {
            self.inner
                .pointer(SDL_PROP_RENDERER_WINDOW_POINTER, std::ptr::null_mut())
        };

        WindowHandle::from_raw(p.cast()).map(|h| unsafe { Ref::from_handle(h) })
    }

    /// The window where rendering is displayed, if this is a surface renderer without a window.
    #[doc(alias = "SDL_PROP_RENDERER_SURFACE_POINTER")]
    pub fn surface(self) -> Option<Ref<'rnd, Surface>> {
        let p = unsafe {
            self.inner
                .pointer(SDL_PROP_RENDERER_SURFACE_POINTER, std::ptr::null_mut())
        };

        SurfaceHandle::from_raw(p.cast()).map(|h| unsafe { Ref::from_handle(h) })
    }

    /// The current VSync setting.
    #[doc(alias = "SDL_PROP_RENDERER_VSYNC_NUMBER")]
    pub fn vsync(self) -> i64 {
        unsafe { self.inner.number(SDL_PROP_RENDERER_VSYNC_NUMBER, 0) }
    }

    /// The maximum texture width/height.
    #[doc(alias = "SDL_PROP_RENDERER_MAX_TEXTURE_SIZE_NUMBER")]
    pub fn max_texture_size(self) -> i64 {
        unsafe {
            self.inner
                .number(SDL_PROP_RENDERER_MAX_TEXTURE_SIZE_NUMBER, 0)
        }
    }

    /// The available texture formats for this renderer.
    #[doc(alias = "SDL_PROP_RENDERER_TEXTURE_FORMATS_POINTER")]
    pub fn texture_formats(self) -> &'rnd [PixelFormat] {
        let begin = unsafe {
            self.inner.pointer(
                SDL_PROP_RENDERER_TEXTURE_FORMATS_POINTER,
                std::ptr::null_mut(),
            )
        }
        .cast::<SDL_PixelFormat>();

        let mut len = 0;
        while unsafe { begin.add(len).read() } != SDL_PixelFormat::UNKNOWN {
            len += 1;
        }
        unsafe { std::slice::from_raw_parts(begin.cast::<PixelFormat>(), len) }
    }

    /// Whether the renderer supports [`SDL_TEXTURE_ADDRESS_WRAP`] on non-power-of-two textures.
    #[doc(alias = "SDL_PROP_RENDERER_TEXTURE_WRAPPING_BOOLEAN")]
    pub fn supports_texture_wrapping(self) -> bool {
        unsafe {
            self.inner
                .bool(SDL_PROP_RENDERER_TEXTURE_WRAPPING_BOOLEAN, false)
        }
    }

    /// The colorspace for output to the display.
    ///
    /// Defaults to [`Colorspace::Srgb`].
    #[doc(alias = "SDL_PROP_RENDERER_OUTPUT_COLORSPACE_NUMBER")]
    pub fn output_colorspace(self) -> Colorspace {
        let cs = SDL_Colorspace::new(unsafe {
            self.inner
                .number(SDL_PROP_RENDERER_OUTPUT_COLORSPACE_NUMBER, 0) as u32
        });

        unsafe { Colorspace::from_sdl_unchecked(cs) }
    }

    /// True when the output colorspace is [`Colorspace::SrgbLinear`] and the renderer
    /// is showing on a display with HDR enabled.
    ///
    /// This can change dynamically when [`Event::WindowHdrStateChanged`] is sent.
    #[doc(alias = "SDL_PROP_RENDERER_HDR_ENABLED_BOOLEAN")]
    pub fn is_hdr_enabled(self) -> bool {
        unsafe {
            self.inner
                .bool(SDL_PROP_RENDERER_HDR_ENABLED_BOOLEAN, false)
        }
    }

    /// The value of SDR white in [`Colorspace::SrgbLinear`].
    ///
    /// When enabled, this value is automatically multiplied into the color scale.
    /// This property can change dynamically when [`Event::WindowHdrStateChanged`] is sent.
    #[doc(alias = "SDL_PROP_RENDERER_SDR_WHITE_POINT_FLOAT")]
    pub fn sdr_white_point(self) -> f32 {
        unsafe {
            self.inner
                .float(SDL_PROP_RENDERER_SDR_WHITE_POINT_FLOAT, 0.)
        }
    }

    /// The additional high dynamic range that can be displayed, in terms of the
    /// SDR white point.
    ///
    /// When HDR is not enabled, this will be 1.0. This property can change dynamically
    /// when [`Event::WindowHdrStateChanged`] is sent.
    #[doc(alias = "SDL_PROP_RENDERER_HDR_HEADROOM_FLOAT")]
    pub fn hdr_headroom(self) -> f32 {
        unsafe { self.inner.float(SDL_PROP_RENDERER_HDR_HEADROOM_FLOAT, 0.) }
    }

    /// (Direct3D 9 only) Returns the `IDirect3DDevice9` associated with the
    /// renderer.
    #[doc(alias = "SDL_PROP_RENDERER_D3D9_DEVICE_POINTER")]
    pub fn d3d9_device(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D9_DEVICE_POINTER)
    }

    /// (Direct3D 11 only) Returns the `ID3D11Device` associated with the
    /// renderer.
    #[doc(alias = "SDL_PROP_RENDERER_D3D11_DEVICE_POINTER")]
    pub fn d3d11_device(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D11_DEVICE_POINTER)
    }

    /// (Direct3D 11 only) Returns the `IDXGISwapChain1` associated with the
    /// renderer.
    ///
    /// This may change when the window is resized.
    #[doc(alias = "SDL_PROP_RENDERER_D3D11_SWAPCHAIN_POINTER")]
    pub fn d3d11_swapchain(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D11_SWAPCHAIN_POINTER)
    }

    /// (Direct3D 12 only) Returns the `ID3D12Device` associated with the
    /// renderer.
    #[doc(alias = "SDL_PROP_RENDERER_D3D12_DEVICE_POINTER")]
    pub fn d3d12_device(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D12_DEVICE_POINTER)
    }

    /// (Direct3D 12 only) Returns the `IDXGISwapChain4` associated with the
    /// renderer.
    #[doc(alias = "SDL_PROP_RENDERER_D3D12_SWAPCHAIN_POINTER")]
    pub fn d3d12_swapchain(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D12_SWAPCHAIN_POINTER)
    }

    /// (Direct3D 12 only) Returns the `ID3D12CommandQueue` associated with
    /// the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_D3D12_COMMAND_QUEUE_POINTER")]
    pub fn d3d12_command_queue(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_D3D12_COMMAND_QUEUE_POINTER)
    }

    /// (Vulkan only) Returns the `VkInstance` associated with the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_INSTANCE_POINTER")]
    pub fn vulkan_instance(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_VULKAN_INSTANCE_POINTER)
    }

    /// (Vulkan only) Returns the `VkSurfaceKHR` associated with the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_SURFACE_NUMBER")]
    pub fn vulkan_surface(self) -> Option<i64> {
        self.opt_number(SDL_PROP_RENDERER_VULKAN_SURFACE_NUMBER)
    }

    /// (Vulkan only) Returns the `VkPhysicalDevice` associated with the
    /// renderer.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_PHYSICAL_DEVICE_POINTER")]
    pub fn vulkan_physical_device(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_VULKAN_PHYSICAL_DEVICE_POINTER)
    }

    /// (Vulkan only) Returns the `VkDevice` associated with the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_DEVICE_POINTER")]
    pub fn vulkan_device(self) -> Option<NonNull<c_void>> {
        self.opt_ptr(SDL_PROP_RENDERER_VULKAN_DEVICE_POINTER)
    }

    /// (Vulkan only) Returns the queue family index used for rendering.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_GRAPHICS_QUEUE_FAMILY_INDEX_NUMBER")]
    pub fn vulkan_graphics_queue_family_index(self) -> Option<i64> {
        self.opt_number(SDL_PROP_RENDERER_VULKAN_GRAPHICS_QUEUE_FAMILY_INDEX_NUMBER)
    }

    /// (Vulkan only) Returns the queue family index used for presentation.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_PRESENT_QUEUE_FAMILY_INDEX_NUMBER")]
    pub fn vulkan_present_queue_family_index(self) -> Option<i64> {
        self.opt_number(SDL_PROP_RENDERER_VULKAN_PRESENT_QUEUE_FAMILY_INDEX_NUMBER)
    }

    /// (Vulkan only) Returns the number of swapchain images, or potential
    /// frames in flight, used by the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_VULKAN_SWAPCHAIN_IMAGE_COUNT_NUMBER")]
    pub fn vulkan_swapchain_image_count(self) -> Option<i64> {
        self.opt_number(SDL_PROP_RENDERER_VULKAN_SWAPCHAIN_IMAGE_COUNT_NUMBER)
    }

    /// (GPU only) Returns the [`Device`] associated with the renderer.
    #[doc(alias = "SDL_PROP_RENDERER_GPU_DEVICE_POINTER")]
    pub fn gpu_device(self) -> Option<Ref<'rnd, Device<'ctx, 'vid>>> {
        let p = unsafe {
            self.inner
                .pointer(SDL_PROP_RENDERER_GPU_DEVICE_POINTER, std::ptr::null_mut())
        };

        DeviceHandle::from_raw(p.cast()).map(|h| unsafe { Ref::from_handle(h) })
    }

    fn get_str(self, key: *const c_char) -> Str<'rnd> {
        let s = unsafe { self.inner.string(key, std::ptr::null()) };

        // SAFETY: Only called for properties whose existence the SDL docs guarantee.
        unsafe { Str::from_ptr_unchecked(s) }
    }

    fn opt_number(self, key: *const c_char) -> Option<i64> {
        unsafe { self.inner.has(key).then(|| self.inner.number(key, 0)) }
    }

    fn opt_ptr(self, key: *const c_char) -> Option<NonNull<c_void>> {
        let p = unsafe { self.inner.pointer(key, std::ptr::null_mut()) };
        NonNull::new(p)
    }
}
