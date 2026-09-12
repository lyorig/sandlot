use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait Handle: Copy {
    /// The "raw" type, i.e. `*mut SDL_Surface`.
    type Raw: Copy;

    /// The actual type contained within the handle, i.e. `NonZero<SDL_Surface>`.
    type Inner: Copy;

    fn as_raw(&self) -> Self::Raw;
    fn as_inner(&self) -> Self::Inner;
}

pub trait Resource: Sized {
    type Handle: Handle;

    /// Return the raw underlying handle of this object.
    ///
    /// # Safety
    /// Think of this function as returning a pointer to `self`.
    /// Handles are only valid as long as their owning objects.
    unsafe fn as_handle(&self) -> Self::Handle;

    /// Create a new reference tied to this resource.
    fn as_ref(&self) -> Ref<'_, Self> {
        unsafe { Ref::from_handle(self.as_handle()) }
    }

    /// Create a new mutable reference tied to this resource.
    fn as_mut<'a>(&'a mut self) -> RefMut<'a, Self> {
        unsafe { RefMut::from_handle(self.as_handle()) }
    }
}

pub struct Ref<'a, T: Resource> {
    pub(crate) handle: T::Handle,
    _marker: PhantomData<&'a T>,
}

impl<T: Resource> Ref<'_, T> {
    /// Construct a new reference from a handle, assuming it is valid.
    /// This conversion is zero-cost.
    ///
    /// # Safety
    /// The lifetime of the returned reference is inferred; functions building
    /// on this one should tie it to the handle's owning object, if possible.
    pub(crate) unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    fn as_raw(&self) -> <T::Handle as Handle>::Raw {
        self.handle.as_raw()
    }
}

impl<T: Resource> Clone for Ref<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Resource> Copy for Ref<'_, T> {}

impl<T: Resource> Deref for Ref<'_, T> {
    type Target = T::Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

pub struct RefMut<'a, T: Resource> {
    pub(crate) handle: T::Handle,
    _marker: PhantomData<&'a mut T>,
}

impl<T: Resource> RefMut<'_, T> {
    /// Construct a new reference from a handle, assuming it is valid.
    /// This conversion is zero-cost.
    ///
    /// # Safety
    /// The lifetime of the returned mutable reference is inferred; functions building
    /// on this one should tie it to the handle's owning object, if possible.
    pub(crate) unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    fn as_raw(&self) -> <T::Handle as Handle>::Raw {
        self.handle.as_raw()
    }
}

impl<T: Resource> Clone for RefMut<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Resource> Copy for RefMut<'_, T> {}

impl<T: Resource> Deref for RefMut<'_, T> {
    type Target = T::Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T: Resource> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

/// Hack around `#[warn(unused_parens)]`.
macro_rules! expand_parens {
    () => {
        ()
    };
    ($t:ty) => {
        $t
    };
    ($($t:ty),+) => {
        ($($t),*)
    };
}

/// Define shared behavior for an owned SDL object.
///
/// # Example usage
///
/// You can either define a type with an explicit [`Drop`]:
///
/// ```
/// use sdl3_sys::video::{SDL_DestroyWindow, SDL_Window};
/// use crate::{init::{Ref, Video}, resource::resource_new};
///
/// resource_new! {
///     pub struct Window<'ctx, 'vid> : SDL_Window, ~SDL_DestroyWindow {
///         marker: PhantomData<(Ref<'vid, Video<'ctx>>)>,
///     }
/// }
/// ```
///
/// or without (for example, if the type's destructor has multiple arguments)
///
/// ```
/// use crate::{gpu::Device, resource::Ref};
/// use sdl3_sys::gpu::SDL_GPUTexture;
///
/// resource_new! {
///     pub struct Texture<'dev> : SDL_GPUTexture {
///         marker: PhantomData<(Ref<'dev, Device>)>,
///     }
/// }
/// ```
///
/// You do **not need to import [`PhantomData`]**. That's just an artistic
/// decision to make the macro look as if you were writing a normal struct.
///
/// FIXME: Omit the `marker` field completely if no lifetimes are specified.
macro_rules! resource_new {
    (
        $(#[$meta:meta])*
        pub struct $owned:ident<$($lt:lifetime),*> : $sdl:ty {
            marker: PhantomData<($($t:ty),*)>,
        }
    ) => {
        ::paste::paste! {
            $(#[$meta])*
            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>]<$($lt),*> {
                handle: ::std::ptr::NonNull<$sdl>,
                marker: ::std::marker::PhantomData<$crate::resource::expand_parens!($($t),*)>
            }


            $(#[$meta])*
            impl<$($lt),*> [<$owned Handle>]<$($lt),*> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    ::std::ptr::NonNull::new(handle).map(|handle| Self {
                        handle,
                        marker: ::std::marker::PhantomData,
                    })
                }

                /// Convenience method to directly access the underlying pointer.
                pub(crate) fn as_ptr(&self) -> *mut $sdl {
                    self.handle.as_ptr()
                }
            }

            $(#[$meta])*
            #[doc(alias = "" $sdl "")]
            pub struct $owned<$($lt),*> {
                inner: [<$owned Handle>]<$($lt),*>
            }

            impl<$($lt),*> $owned<$($lt),*> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> $crate::Result<Self> {
                    match ::std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] {
                                handle,
                                marker: ::std::marker::PhantomData,
                            },
                        }),
                        None => Err($crate::error::Error::current()),
                    }
                }
            }

            impl<$($lt),*> ::std::ops::Deref for $owned<$($lt),*> {
                type Target = [<$owned Handle>]<$($lt),*>;

                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl<$($lt),*> ::std::ops::DerefMut for $owned<$($lt),*> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl<$($lt),*> $crate::resource::Handle for [<$owned Handle>]<$($lt),*> {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl<$($lt),*> $crate::resource::Resource for $owned<$($lt),*> {
                type Handle = [<$owned Handle>]<$($lt),*>;

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }
        }
    };

    (
        $(#[$meta:meta])*
        pub struct $owned:ident<$($lt:lifetime),*> : $sdl:ty, ~$dtor:ident {
            marker: PhantomData<($($t:ty),*)>,
        }
    ) => {
        $crate::resource::resource_new! {
            $(#[$meta])*
            pub struct $owned<$($lt),*> : $sdl {
                marker: PhantomData<($($t),*)>,
            }
        }

        ::paste::paste! {
            impl<$($lt),*> ::std::ops::Drop for $owned<$($lt),*> {
                #[doc(alias = "" $dtor "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    }
}

pub(crate) use {expand_parens, resource_new};
