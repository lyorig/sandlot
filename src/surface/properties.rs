use std::marker::PhantomData;

use sdl3_sys::surface::*;

use crate::{properties::Properties, resource::Ref, str::Str, surface::Surface};

#[derive(Clone, Copy)]
pub struct SurfaceProperties<'surf> {
    props: Ref<'surf, Properties>,
    marker: PhantomData<Ref<'surf, Surface>>,
}

impl<'surf> SurfaceProperties<'surf> {
    pub(crate) fn new(props: Ref<'surf, Properties>) -> Self {
        Self {
            props,
            marker: PhantomData,
        }
    }

    /// (HDR10 and floating point formats only) Defines the value of 100%
    /// diffuse white, with higher values being displayed in the High Dynamic
    /// Range headroom.
    ///
    /// This defaults to 203 for HDR10 surfaces and 1.0 for floating point surfaces.
    #[doc(alias = "SDL_PROP_SURFACE_SDR_WHITE_POINT_FLOAT")]
    pub fn sdr_white_point(self) -> Option<f32> {
        const SENTINEL: f32 = f32::INFINITY;

        let val = unsafe {
            self.props
                .float(SDL_PROP_SURFACE_SDR_WHITE_POINT_FLOAT, SENTINEL)
        };

        (val != SENTINEL).then_some(val)
    }

    /// (HDR10 and floating point formats only) Defines the maximum dynamic
    /// range used by the content, in terms of the SDR white point.
    ///
    /// This defaults to 0.0, which disables tone mapping.
    #[doc(alias = "SDL_PROP_SURFACE_HDR_HEADROOM_FLOAT")]
    pub fn hdr_headroom(self) -> Option<f32> {
        const SENTINEL: f32 = f32::INFINITY;

        let val = unsafe {
            self.props
                .float(SDL_PROP_SURFACE_HDR_HEADROOM_FLOAT, SENTINEL)
        };

        (val != SENTINEL).then_some(val)
    }

    /// (HDR10 formats only) The tone mapping operator used when compressing from a surface with high
    /// dynamic range to another with lower dynamic range.
    ///
    /// Currently this supports "chrome", which uses the same tone mapping that
    /// Chrome uses for HDR content, the form "*=N", where N is a floating point
    /// scale factor applied in linear space, and "none", which disables tone mapping.
    ///
    /// This defaults to "chrome".
    #[doc(alias = "SDL_PROP_SURFACE_TONEMAP_OPERATOR_STRING")]
    pub fn tonemap_operator(self) -> Option<&'surf Str> {
        let ptr = unsafe {
            self.props
                .string(SDL_PROP_SURFACE_TONEMAP_OPERATOR_STRING, std::ptr::null())
        };

        (!ptr.is_null()).then(|| unsafe { Str::from_ptr(ptr) })
    }

    /// (Cursor surfaces only) The hotspot pixel offset from the left edge of the image.
    #[doc(alias = "SDL_PROP_SURFACE_HOTSPOT_X_NUMBER")]
    pub fn hotspot_x(self) -> Option<i64> {
        const SENTINEL: i64 = i64::MIN;
        let val = unsafe {
            self.props
                .number(SDL_PROP_SURFACE_HOTSPOT_X_NUMBER, SENTINEL)
        };

        (val != SENTINEL).then_some(val)
    }

    /// (Cursor surfaces only) The hotspot pixel offset from the top edge of the image.
    #[doc(alias = "SDL_PROP_SURFACE_HOTSPOT_Y_NUMBER")]
    pub fn hotspot_y(self) -> Option<i64> {
        const SENTINEL: i64 = i64::MIN;
        let val = unsafe {
            self.props
                .number(SDL_PROP_SURFACE_HOTSPOT_Y_NUMBER, SENTINEL)
        };

        (val != SENTINEL).then_some(val)
    }

    /// The number of degrees a surface's data is meant to be rotated clockwise to
    /// make the image right-side up.
    ///
    /// This is used by the camera API, if a mobile device is oriented differently
    /// than what its camera provides (i.e. - the camera always provides portrait images
    /// but the phone is being held in landscape orientation).
    ///
    /// Defaults to 0.
    #[doc(alias = "SDL_PROP_SURFACE_ROTATION_FLOAT")]
    pub fn rotation(self) -> f32 {
        unsafe { self.props.float(SDL_PROP_SURFACE_ROTATION_FLOAT, 0.0) }
    }
}
