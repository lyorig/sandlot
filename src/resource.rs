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

#[macro_export]
macro_rules! resource_new_impl {
    ($sdl:ident, $owned:ident) => {
        paste::paste! {
            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>] {
                pub(crate) handle: std::ptr::NonNull<$sdl>,
            }

            impl [<$owned Handle>] {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    std::ptr::NonNull::new(handle).map(|handle| Self { handle })
                }

                pub(crate) fn as_ptr(&self) -> *mut $sdl {
                    self.handle.as_ptr()
                }
            }

            impl $owned {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> $crate::Result<Self> {
                    match std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] { handle },
                        }),
                        None => Err($crate::error::Error::current()),
                    }
                }
            }

            impl std::ops::Deref for $owned {
                type Target = [<$owned Handle>];
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl std::ops::DerefMut for $owned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl $crate::resource::Handle for [<$owned Handle>] {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl $crate::resource::Resource for $owned {
                type Handle = [<$owned Handle>];

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }
        }
    };
}

#[macro_export]
macro_rules! resource_new_no_drop {
    ($(#[$meta:meta])* $sdl:ident, $owned:ident) => {
        paste::paste! {
            $(#[$meta])*
            ///
            /// BEWARE: This struct has no automatic destructor, and must be manually dropped or otherwise consumed!
            #[must_use = "This struct has to be manually dropped via an associated `drop()` method."]
            #[doc(alias = "" $sdl "")]
            pub struct $owned {
                pub(crate) inner: [<$owned Handle>],
            }

            $crate::resource_new_impl!($sdl, $owned);
        }
    };
}

/// Define a resource and implement shared traits and member functions.
#[macro_export]
macro_rules! resource_new {
    ($(#[$meta:meta])* $sdl:ident, $owned:ident, $dtor:ident) => {
        paste::paste! {
            $(#[$meta])*
            #[doc(alias = "" $sdl "")]
            pub struct $owned {
                pub(crate) inner: [<$owned Handle>],
            }
        }

        paste::paste! {
            $crate::resource_new_impl!($sdl, $owned);

            impl Drop for $owned {
                #[doc(alias = "" $sdl "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! resource_new_tied {
    ($(#[$meta:meta])* $sdl:ident, $owned:ident, $dtor:ident, $tied:ident) => {
        paste::paste! {
            $(#[$meta])*
            #[doc(alias = "" $sdl "")]
            pub struct $owned<'a> {
                pub(crate) inner: [<$owned Handle>],
                marker: PhantomData<&'a $tied>,
            }

            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>] {
                pub(crate) handle: ::std::ptr::NonNull<$sdl>,
            }

            impl [<$owned Handle>] {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    std::ptr::NonNull::new(handle).map(|handle| Self { handle })
                }

                /// Convenience method to directly access the underlying pointer.
                pub(crate) fn as_ptr(&self) -> *mut $sdl {
                    self.handle.as_ptr()
                }
            }

            impl<'a> $owned<'a> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> $crate::Result<Self> {
                    match std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] { handle },
                            marker: PhantomData,
                        }),
                        None => Err($crate::error::Error::current()),
                    }
                }
            }

            impl std::ops::Deref for $owned<'_> {
                type Target = [<$owned Handle>];
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            impl std::ops::DerefMut for $owned<'_> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl $crate::resource::Handle for [<$owned Handle>] {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl $crate::resource::Resource for $owned<'_> {
                type Handle = [<$owned Handle>];

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }

            impl Drop for $owned<'_> {
                #[doc(alias = "" $dtor "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    };
}
