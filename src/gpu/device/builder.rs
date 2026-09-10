use std::{
    ffi::{CStr, c_char},
    marker::PhantomData,
    ptr,
};

use sdl3_sys::gpu::*;

use crate::{Result, gpu::Device, properties::Properties, resource::Ref};

const CREATE_PROPERTIES: [*const c_char; 21] = [
    SDL_PROP_GPU_DEVICE_CREATE_DEBUGMODE_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_PREFERLOWPOWER_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_VERBOSE_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_NAME_STRING,
    SDL_PROP_GPU_DEVICE_CREATE_FEATURE_CLIP_DISTANCE_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_FEATURE_DEPTH_CLAMPING_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_FEATURE_INDIRECT_DRAW_FIRST_INSTANCE_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_FEATURE_ANISOTROPY_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_PRIVATE_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_SPIRV_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXBC_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXIL_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_MSL_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_SHADERS_METALLIB_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_D3D12_ALLOW_FEWER_RESOURCE_SLOTS_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_D3D12_SEMANTIC_NAME_STRING,
    SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_VERSION_NUMBER,
    SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_PATH_STRING,
    SDL_PROP_GPU_DEVICE_CREATE_VULKAN_REQUIRE_HARDWARE_ACCELERATION_BOOLEAN,
    SDL_PROP_GPU_DEVICE_CREATE_VULKAN_OPTIONS_POINTER,
    SDL_PROP_GPU_DEVICE_CREATE_METAL_ALLOW_MACFAMILY1_BOOLEAN,
];

/// Builder for [`Device`], using
/// [`SDL_CreateGPUDeviceWithProperties`](https://wiki.libsdl.org/SDL3/SDL_CreateGPUDeviceWithProperties).
pub struct DeviceBuilder<'p, 'vo> {
    inner: Ref<'p, Properties>,
    marker: PhantomData<&'vo SDL_GPUVulkanOptions>,
}

impl<'p, 'vo> DeviceBuilder<'p, 'vo> {
    pub(super) fn new(inner: Ref<'p, Properties>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Enable debug mode properties and validations. Defaults to `true`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_DEBUGMODE_BOOLEAN")]
    pub fn debug_mode(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_DEBUGMODE_BOOLEAN, value)
    }

    /// Prefer energy efficiency over maximum GPU performance. Defaults to `false`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_PREFERLOWPOWER_BOOLEAN")]
    pub fn prefer_low_power(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_PREFERLOWPOWER_BOOLEAN, value)
    }

    /// Automatically log useful debug information on device creation. Defaults to `true`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_VERBOSE_BOOLEAN")]
    pub fn verbose(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_VERBOSE_BOOLEAN, value)
    }

    /// The name of the GPU driver to use, if a specific one is desired.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_NAME_STRING")]
    pub fn name(&mut self, value: &CStr) -> &mut Self {
        self.set_string(SDL_PROP_GPU_DEVICE_CREATE_NAME_STRING, value)
    }

    /// Enable the Vulkan `shaderClipDistance` feature. Defaults to `true`.
    /// Disabling optional features allows the application to run on some
    /// older Android devices.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_FEATURE_CLIP_DISTANCE_BOOLEAN")]
    pub fn clip_distance(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_FEATURE_CLIP_DISTANCE_BOOLEAN,
            value,
        )
    }

    /// Enable the Vulkan `depthClamp` feature. Defaults to `true`.
    /// Disabling optional features allows the application to run on some
    /// older Android devices.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_FEATURE_DEPTH_CLAMPING_BOOLEAN")]
    pub fn depth_clamping(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_FEATURE_DEPTH_CLAMPING_BOOLEAN,
            value,
        )
    }

    /// Enable the Vulkan `drawIndirectFirstInstance` feature. Defaults to `true`.
    /// Disabling optional features allows the application to run on some
    /// older Android devices.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_FEATURE_INDIRECT_DRAW_FIRST_INSTANCE_BOOLEAN")]
    pub fn indirect_draw_first_instance(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_FEATURE_INDIRECT_DRAW_FIRST_INSTANCE_BOOLEAN,
            value,
        )
    }

    /// Enable the Vulkan `samplerAnisotropy` feature. Defaults to `true`.
    /// Disabling optional features allows the application to run on some
    /// older Android devices.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_FEATURE_ANISOTROPY_BOOLEAN")]
    pub fn anisotropy(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_FEATURE_ANISOTROPY_BOOLEAN, value)
    }

    /// The app is able to provide shaders for an NDA platform.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_PRIVATE_BOOLEAN")]
    pub fn shaders_private(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_PRIVATE_BOOLEAN, value)
    }

    /// The app is able to provide SPIR-V shaders, if applicable.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_SPIRV_BOOLEAN")]
    pub fn shaders_spirv(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_SPIRV_BOOLEAN, value)
    }

    /// The app is able to provide DXBC shaders, if applicable.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXBC_BOOLEAN")]
    pub fn shaders_dxbc(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXBC_BOOLEAN, value)
    }

    /// The app is able to provide DXIL shaders, if applicable.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXIL_BOOLEAN")]
    pub fn shaders_dxil(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_DXIL_BOOLEAN, value)
    }

    /// The app is able to provide MSL shaders, if applicable.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_MSL_BOOLEAN")]
    pub fn shaders_msl(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_MSL_BOOLEAN, value)
    }

    /// The app is able to provide Metal shader libraries, if applicable.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_SHADERS_METALLIB_BOOLEAN")]
    pub fn shaders_metallib(&mut self, value: bool) -> &mut Self {
        self.set_bool(SDL_PROP_GPU_DEVICE_CREATE_SHADERS_METALLIB_BOOLEAN, value)
    }

    /// Allow D3D12 Tier 1 resource binding support, if the application uses
    /// 8 or fewer storage resources across all shader stages. Defaults to `false`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_D3D12_ALLOW_FEWER_RESOURCE_SLOTS_BOOLEAN")]
    pub fn d3d12_allow_fewer_resource_slots(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_D3D12_ALLOW_FEWER_RESOURCE_SLOTS_BOOLEAN,
            value,
        )
    }

    /// The prefix to use for all D3D12 vertex semantics. Defaults to `"TEXCOORD"`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_D3D12_SEMANTIC_NAME_STRING")]
    pub fn d3d12_semantic_name(&mut self, value: &CStr) -> &mut Self {
        self.set_string(SDL_PROP_GPU_DEVICE_CREATE_D3D12_SEMANTIC_NAME_STRING, value)
    }

    /// The D3D12 Agility SDK version. Certain feature checks are only possible
    /// on Windows 11 by default; by setting this alongside the path property
    /// and vendoring D3D12Core.dll, they become possible on older platforms.
    /// The version must match the one given in the DLL.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_VERSION_NUMBER")]
    pub fn d3d12_agility_sdk_version(&mut self, value: i64) -> &mut Self {
        self.set_number(
            SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_VERSION_NUMBER,
            value,
        )
    }

    /// The path to the D3D12 Agility SDK DLL, relative to the executable path
    /// of the app. Do not put the DLL in the same directory as the exe.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_PATH_STRING")]
    pub fn d3d12_agility_sdk_path(&mut self, value: &CStr) -> &mut Self {
        self.set_string(
            SDL_PROP_GPU_DEVICE_CREATE_D3D12_AGILITY_SDK_PATH_STRING,
            value,
        )
    }

    /// Require hardware acceleration for the Vulkan device, excluding
    /// software renderers such as Lavapipe. Defaults to `false`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_VULKAN_REQUIRE_HARDWARE_ACCELERATION_BOOLEAN")]
    pub fn vulkan_require_hardware_acceleration(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_VULKAN_REQUIRE_HARDWARE_ACCELERATION_BOOLEAN,
            value,
        )
    }

    /// Configure Vulkan-specific options during device creation. This allows
    /// configuring a variety of Vulkan-specific options such as increasing the
    /// API version and opting into extensions aside from the minimal set SDL
    /// requires. The referenced struct is read at build time and must outlive
    /// `build()`.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_VULKAN_OPTIONS_POINTER")]
    pub fn vulkan_options(&mut self, value: &'vo SDL_GPUVulkanOptions) -> &mut Self {
        _ = unsafe {
            self.inner.set_pointer(
                SDL_PROP_GPU_DEVICE_CREATE_VULKAN_OPTIONS_POINTER,
                ptr::from_ref(value) as _,
            )
        };
        self
    }

    /// Allow macOS support for `MTLGPUFamilyMac1` hardware, if the
    /// application does not write to sRGB textures.
    #[doc(alias = "SDL_PROP_GPU_DEVICE_CREATE_METAL_ALLOW_MACFAMILY1_BOOLEAN")]
    pub fn metal_allow_macfamily1(&mut self, value: bool) -> &mut Self {
        self.set_bool(
            SDL_PROP_GPU_DEVICE_CREATE_METAL_ALLOW_MACFAMILY1_BOOLEAN,
            value,
        )
    }

    /// Clear all device creation properties from a property group.
    pub fn clear_from(props: Ref<Properties>) {
        for key in CREATE_PROPERTIES {
            _ = unsafe { props.clear(key) };
        }
    }

    /// Build the device.
    #[doc(alias = "SDL_CreateGPUDeviceWithProperties")]
    pub fn build(&self) -> Result<Device> {
        Device::from_ptr(unsafe { SDL_CreateGPUDeviceWithProperties(self.inner.id()) })
    }

    /// Build the device, and cleanup all properties.
    /// See the [crate::properties] module docs for more info.
    #[doc(alias = "SDL_CreateGPUDeviceWithProperties")]
    pub fn build_cleanup(&self) -> Result<Device> {
        let res = Device::from_ptr(unsafe { SDL_CreateGPUDeviceWithProperties(self.inner.id()) });
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

    fn set_string(&mut self, key: *const c_char, value: &CStr) -> &mut Self {
        _ = unsafe { self.inner.set_string(key, value.as_ptr()) };
        self
    }
}
