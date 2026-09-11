//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryGPU)):
//! - [x] SDL_ClaimWindowForGPUDevice
//! - [x] SDL_CreateGPUDevice
//! - [x] SDL_CreateGPUDeviceWithProperties
//! - [x] SDL_DestroyGPUDevice
//! - [x] SDL_GetGPUDeviceDriver
//! - [x] SDL_GetGPUDeviceProperties
//! - [x] SDL_GetGPUSwapchainTextureFormat
//! - [x] SDL_GPUTextureSupportsFormat
//! - [x] SDL_GPUTextureSupportsSampleCount
//! - [x] SDL_ReleaseWindowFromGPUDevice
//! - [x] SDL_SetGPUAllowedFramesInFlight
//! - [x] SDL_SetGPUSwapchainParameters
//! - [x] SDL_WaitForGPUFences
//! - [x] SDL_WaitForGPUIdle
//! - [x] SDL_WaitForGPUSwapchain
//! - [x] SDL_WindowSupportsGPUPresentMode
//! - [x] SDL_WindowSupportsGPUSwapchainComposition
//! - [x] SDL_GetGPUShaderFormats

use std::ffi::CStr;

use sdl3_sys::gpu::*;

use crate::{
    Result,
    error::Error,
    gpu::{EnableDebug, WaitAll},
    properties::{Properties, PropertiesHandle},
    resource::Ref,
    util::impl_enum_transmute,
    util::mod_reexport,
    util::resource_new,
    util::to_result,
    window::Window,
};

use super::{
    ShaderFormats,
    fence::Fence,
    texture::{SampleCount, TextureFormat, TextureType, TextureUsageFlags},
};

mod_reexport!(builder);
mod_reexport!(properties);

/// The timing used to present swapchain textures to the OS.
///
/// [`Self::Vsync`] is always supported. [`Self::Immediate`] and
/// [`Self::Mailbox`] may not be supported on some systems; query support after
/// claiming the window before selecting either mode.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUPresentMode")]
pub enum PresentMode {
    /// Wait for vertical blanking before presenting. Tearing is not possible,
    /// but visual latency may increase.
    Vsync = SDL_GPUPresentMode::VSYNC.0,
    /// Present immediately for the lowest latency. Tearing may occur.
    Immediate = SDL_GPUPresentMode::IMMEDIATE.0,
    /// Wait for vertical blanking without presenting stale pending images.
    /// Tearing is not possible, with lower visual latency than [`Self::Vsync`].
    Mailbox = SDL_GPUPresentMode::MAILBOX.0,
}

/// The texture format and color space of swapchain textures.
///
/// [`Self::Sdr`] is always supported. Other compositions may not be supported
/// on some systems; query support after claiming the window before selecting one.
#[repr(i32)]
#[derive(Clone, Copy)]
#[doc(alias = "SDL_GPUSwapchainComposition")]
pub enum SwapchainComposition {
    /// Use a B8G8R8A8 or R8G8B8A8 swapchain with pixel values in sRGB encoding.
    Sdr = SDL_GPUSwapchainComposition::SDR.0,
    /// Use a B8G8R8A8_SRGB or R8G8B8A8_SRGB swapchain. Values are stored in
    /// sRGB encoding but accessed in shaders with a linear transfer function.
    SdrLinear = SDL_GPUSwapchainComposition::SDR_LINEAR.0,
    /// Use an R16G16B16A16_FLOAT swapchain with extended linear sRGB values,
    /// including values outside the `[0, 1]` range.
    HdrExtendedLinear = SDL_GPUSwapchainComposition::HDR_EXTENDED_LINEAR.0,
    /// Use an A2R10G10B10 or A2B10G10B10 swapchain with BT.2020 ST2084 (PQ)
    /// encoding.
    Hdr10St2084 = SDL_GPUSwapchainComposition::HDR10_ST2084.0,
}

impl_enum_transmute!(SDL_GPUPresentMode, PresentMode);
impl_enum_transmute!(SDL_GPUSwapchainComposition, SwapchainComposition);

resource_new!(
    /// An opaque handle representing the SDL_GPU context.
    SDL_GPUDevice, Device, SDL_DestroyGPUDevice
);
impl Device {
    /// Create a GPU device.
    ///
    /// `formats` indicates the shader formats that the application can provide.
    /// `debug` enables debug-mode properties and validation when supported.
    /// SDL selects the optimal GPU driver automatically; this method does not
    /// expose the optional driver-name parameter of the underlying SDL function.
    ///
    /// Returns [`Err`] if the GPU device cannot be created.
    #[doc(alias = "SDL_CreateGPUDevice")]
    pub fn new(formats: ShaderFormats, debug: EnableDebug) -> Result<Self> {
        let fmts = SDL_GPUShaderFormat::new(formats.bits());
        let handle = unsafe { SDL_CreateGPUDevice(fmts, debug.into(), std::ptr::null()) };
        Self::from_ptr(handle)
    }

    /// Build a [`Device`] with additional parameters not available in [`Device::new`].
    pub fn builder(props: Ref<Properties>) -> DeviceBuilder {
        DeviceBuilder::new(props)
    }
}

impl DeviceHandle {
    /// Claim a window and create its swapchain structure.
    ///
    /// `window` is the window to claim. It must be claimed before acquiring a
    /// swapchain texture. The initial swapchain uses [`SwapchainComposition::Sdr`]
    /// and [`PresentMode::Vsync`]; call [`Self::set_swapchain_parameters`] after
    /// claiming the window to request different parameters.
    ///
    /// Returns [`Err`] if the window cannot be claimed.
    #[doc(alias = "SDL_ClaimWindowForGPUDevice")]
    pub fn claim_window(&self, window: Ref<Window>) -> Result<()> {
        to_result(unsafe {
            SDL_ClaimWindowForGPUDevice(self.handle.as_ptr(), window.handle.as_ptr())
        })
    }

    /// Unclaim a window and destroy its swapchain structure.
    ///
    /// `window` must be a window currently claimed by this device.
    #[doc(alias = "SDL_ReleaseWindowFromGPUDevice")]
    pub fn release_window(&self, window: Ref<Window>) {
        unsafe { SDL_ReleaseWindowFromGPUDevice(self.handle.as_ptr(), window.handle.as_ptr()) };
    }

    /// Determine whether a presentation mode is supported by a window.
    ///
    /// `window` must be claimed by this device, and `pm` is the presentation
    /// mode to check. Returns `true` when the mode is supported.
    #[doc(alias = "SDL_WindowSupportsGPUPresentMode")]
    pub fn window_supports_gpu_present_mode(&self, window: Ref<Window>, pm: PresentMode) -> bool {
        unsafe {
            SDL_WindowSupportsGPUPresentMode(
                self.handle.as_ptr(),
                window.handle.as_ptr(),
                SDL_GPUPresentMode::new(pm as _),
            )
        }
    }

    /// Determine whether a swapchain composition is supported by a window.
    ///
    /// `window` must be claimed by this device, and `sc` is the swapchain
    /// composition to check. Returns `true` when the composition is supported.
    #[doc(alias = "SDL_WindowSupportsGPUSwapchainComposition")]
    pub fn window_supports_gpu_swapchain_composition(
        &self,
        window: Ref<Window>,
        sc: SwapchainComposition,
    ) -> bool {
        unsafe {
            SDL_WindowSupportsGPUSwapchainComposition(
                self.handle.as_ptr(),
                window.handle.as_ptr(),
                SDL_GPUSwapchainComposition::new(sc as _),
            )
        }
    }

    /// Block until the GPU is completely idle.
    ///
    /// Returns [`Err`] if SDL cannot wait for the device to become idle.
    #[doc(alias = "SDL_WaitForGPUIdle")]
    pub fn wait_idle(&self) -> Result<()> {
        to_result(unsafe { SDL_WaitForGPUIdle(self.handle.as_ptr()) })
    }

    /// Block until all presenting command buffers for a window finish executing.
    ///
    /// `window` must be claimed by this device.
    ///
    /// Returns [`Err`] if SDL cannot wait for the window's presenting work.
    #[doc(alias = "SDL_WaitForGPUSwapchain")]
    pub fn wait_swapchain(&self, window: Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_WaitForGPUSwapchain(self.handle.as_ptr(), window.handle.as_ptr()) })
    }

    /// Block until the given fences are signaled.
    ///
    /// `wait_all` selects whether to wait for every fence or return when any
    /// fence is signaled. `fences` is the collection of fences to wait on.
    ///
    /// Returns [`Err`] if SDL cannot wait for the fences.
    #[doc(alias = "SDL_WaitForGPUFences")]
    pub fn wait_fences(&self, wait_all: WaitAll, fences: &[Ref<Fence>]) -> Result<()> {
        to_result(unsafe {
            SDL_WaitForGPUFences(
                self.handle.as_ptr(),
                wait_all.into(),
                fences.as_ptr().cast(),
                fences.len() as _,
            )
        })
    }

    /// Return the name of the backend used to create this GPU device.
    ///
    /// Returns [`Err`] if SDL cannot retrieve the driver name.
    #[doc(alias = "SDL_GetGPUDeviceDriver")]
    pub fn driver(&self) -> Result<&str> {
        let raw = unsafe { SDL_GetGPUDeviceDriver(self.handle.as_ptr()) };
        if raw.is_null() {
            Err(Error::current())
        } else {
            let cstr = unsafe { CStr::from_ptr(raw) };
            Ok(unsafe { str::from_utf8_unchecked(cstr.to_bytes()) })
        }
    }

    /// Get the properties associated with this GPU device.
    ///
    /// Properties are optional and may differ between GPU backends and SDL
    /// versions. The returned view is borrowed from this device and provides
    /// access to the device and driver information exposed by SDL.
    #[doc(alias = "SDL_GetGPUDeviceProperties")]
    pub fn properties(&self) -> DeviceProperties<'_> {
        unsafe {
            let id = SDL_GetGPUDeviceProperties(self.handle.as_ptr());
            let handle = PropertiesHandle::from_id(id).unwrap_unchecked();
            let r = Ref::from_handle(handle);

            DeviceProperties::new(r)
        }
    }

    /// Get the texture format of a window's swapchain.
    ///
    /// `window` must be claimed by this device. The format can change when the
    /// swapchain parameters change.
    #[doc(alias = "SDL_GetGPUSwapchainTextureFormat")]
    pub fn swapchain_texture_format(&self, window: Ref<Window>) -> TextureFormat {
        unsafe {
            let fmt =
                SDL_GetGPUSwapchainTextureFormat(self.handle.as_ptr(), window.handle.as_ptr());
            TextureFormat::from_sdl_unchecked(fmt)
        }
    }

    /// Determine whether a texture format is supported for a type and usage.
    ///
    /// `format` is the texture format to check. `kind` is the texture type,
    /// such as 2D, 3D, or cube. `usage` is the set of usage scenarios to check.
    /// Returns `true` when the format is supported for all requested uses.
    #[doc(alias = "SDL_GPUTextureSupportsFormat")]
    pub fn texture_supports_format(
        &self,
        format: TextureFormat,
        kind: TextureType,
        usage: TextureUsageFlags,
    ) -> bool {
        unsafe {
            SDL_GPUTextureSupportsFormat(
                self.handle.as_ptr(),
                SDL_GPUTextureFormat::new(format as _),
                SDL_GPUTextureType::new(kind as _),
                SDL_GPUTextureUsageFlags::new(usage.bits()),
            )
        }
    }

    /// Determine whether a sample count is supported for a texture format.
    ///
    /// `format` is the texture format to check, and `sample_count` is the
    /// sample count to check. Returns `true` when the combination is supported.
    #[doc(alias = "SDL_GPUTextureSupportsSampleCount")]
    pub fn texture_supports_sample_count(
        &self,
        format: TextureFormat,
        sample_count: SampleCount,
    ) -> bool {
        unsafe {
            SDL_GPUTextureSupportsSampleCount(
                self.handle.as_ptr(),
                SDL_GPUTextureFormat::new(format as _),
                SDL_GPUSampleCount::new(sample_count as _),
            )
        }
    }

    /// Change the swapchain parameters for a claimed window.
    ///
    /// `window` must be claimed by this device. `composition` is the desired
    /// swapchain composition, and `present_mode` is the desired presentation
    /// mode.
    ///
    /// The operation fails if either requested value is unsupported. Use
    /// [`Self::window_supports_gpu_present_mode`] and
    /// [`Self::window_supports_gpu_swapchain_composition`] to check first.
    /// [`PresentMode::Vsync`] with [`SwapchainComposition::Sdr`] is always
    /// supported.
    ///
    /// Returns [`Err`] if the requested parameters are unsupported or another
    /// SDL error occurs.
    #[doc(alias = "SDL_SetGPUSwapchainParameters")]
    pub fn set_swapchain_parameters(
        &self,
        window: Ref<Window>,
        composition: SwapchainComposition,
        present_mode: PresentMode,
    ) -> Result<()> {
        to_result(unsafe {
            SDL_SetGPUSwapchainParameters(
                self.handle.as_ptr(),
                window.handle.as_ptr(),
                SDL_GPUSwapchainComposition::new(composition as _),
                SDL_GPUPresentMode::new(present_mode as _),
            )
        })
    }

    /// Return the shader formats supported by this GPU device.
    ///
    /// The returned bitflags identify the shader formats that the driver can
    /// consume.
    #[doc(alias = "SDL_GetGPUShaderFormats")]
    pub fn shader_formats(&self) -> ShaderFormats {
        let fmts = unsafe { SDL_GetGPUShaderFormats(self.handle.as_ptr()) };
        ShaderFormats::from_bits_retain(fmts.0)
    }

    /// Configure the maximum number of frames that may be in flight.
    ///
    /// `n` is the maximum number of frames pending on the GPU. It must be in
    /// the range `1..=3`; the default is `2`. Higher values can improve
    /// throughput at the cost of visual latency, while lower values can reduce
    /// latency at the cost of throughput.
    ///
    /// Changing this setting stalls and flushes the command queue to prevent
    /// synchronization issues.
    ///
    /// Returns [`Err`] if `n` is outside the supported range or SDL cannot apply
    /// the setting.
    #[doc(alias = "SDL_SetGPUAllowedFramesInFlight")]
    pub fn set_allowed_frames_in_flight(&self, n: u32) -> Result<()> {
        to_result(unsafe { SDL_SetGPUAllowedFramesInFlight(self.handle.as_ptr(), n) })
    }
}
