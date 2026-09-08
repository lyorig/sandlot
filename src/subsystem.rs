use std::marker::PhantomData;

use sdl3_sys::init::*;

use crate::{Context, Result, error::Error};

/// A RAII type handling (de)initialization of an SDL subsystem.
///
/// # Usage with `ManuallyDrop`
/// By default, dropping a subsystem de-initializes it.
/// However, since dropping a [`Context`] de-initializes everything "by force"
/// anyways, you can potentially avoid redundant FFI calls by wrapping this struct
/// in `ManuallyDrop`:
///
/// ```rust
/// use std::mem::ManuallyDrop;
/// use sandlot::{Context, subsystem::Video};
///
/// let ctx = Context::new();
///
/// // Upon going out of scope, `ctx` de-initializes the video
/// // subsystem, making it redundant to drop `vid`.
/// let vid = ManuallyDrop::new(Video::new(&ctx).unwrap());
/// ```
pub struct Subsystem<'ctx, const TYPE: u32> {
    marker: PhantomData<&'ctx Context>,
}

impl<const N: u32> Subsystem<'_, N> {
    const FLAG: SDL_InitFlags = SDL_InitFlags::new(N);

    /// Initialize a specific SDL subsystem.
    ///
    /// # Remarks
    ///
    /// This function and SDL's library initialization are interchangeable.
    #[doc(alias = "SDL_InitSubSystem")]
    pub fn new(_: &Context) -> Result<Subsystem<'_, N>> {
        let res = unsafe { SDL_InitSubSystem(Self::FLAG) };
        if res {
            Ok(Subsystem {
                marker: PhantomData,
            })
        } else {
            Err(Error::current())
        }
    }

    /// Get whether this subsystem is currently initialized.
    #[doc(alias = "SDL_WasInit")]
    pub fn is_init() -> bool {
        let flag = unsafe { SDL_WasInit(Self::FLAG) };
        flag == Self::FLAG
    }
}

impl<const N: u32> Drop for Subsystem<'_, N> {
    /// Shut down this SDL subsystem.
    ///
    /// # Remarks
    ///
    /// You still need to call SDL's quit function even if you close all open
    /// subsystems; dropping the [`Context`] does this automatically.
    #[doc(alias = "SDL_QuitSubSystem")]
    fn drop(&mut self) {
        unsafe { SDL_QuitSubSystem(Self::FLAG) };
    }
}

pub type Video<'ctx> = Subsystem<'ctx, { SDL_InitFlags::VIDEO.0 }>;
pub type Events<'ctx> = Subsystem<'ctx, { SDL_InitFlags::EVENTS.0 }>;
