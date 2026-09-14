use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// A non-owning handle to a resource.
///
/// These exist to enable Sandlot's custom references. They're analogous to a Rust pointer,
/// as they aren't lifetime-bound to any particular resource, so obtaining them directly is unsafe.
///
/// Both the owned type and [`Ref`]/[`RefMut`] contain this handle and [`Deref`] to it, the difference being:
/// - the owned type [`Drop`]s the handle
/// - the references are tied to the owned type via [`PhantomData`]
///
/// # Getting into specifics
///
/// This design prevents double indirection, which using "standard" Rust references would incur, because
/// SDL uses the [PImpl](https://en.cppreference.com/cpp/language/pimpl) idiom for most of its structures.
/// This way, SDL only exposes pointers (often called "handles" in many APIs) to these objects, and manipulation
/// is only possible via API functions themselves. As such, taking a reference to an SDL object would be a reference
/// to a pointer, incurring an unnecessary double indirection.
pub trait Handle: Copy {
    /// The "raw" type, i.e. `*mut SDL_Surface`.
    type Raw: Copy;

    /// The actual type contained within the handle, i.e. `NonZero<SDL_Surface>`.
    type Inner: Copy;

    /// Get this type's "raw" representation,
    /// i.e. `*mut SDL_Surface` for [`Surface`](crate::surface::Surface),
    /// or `SDL_PropertiesID` for [`Properties`](crate::properties::Properties).
    fn as_raw(self) -> Self::Raw;

    /// Get this type's "inner" representation,
    /// i.e. `NonNull<SDL_Surface>` for [`Surface`](crate::surface::Surface),
    /// or `NonZero<u32>` for [`Properties`](crate::properties::Properties).
    fn as_inner(self) -> Self::Inner;
}

/// An owning handle to a resource.
///
///
pub trait Resource: Sized {
    type Handle: Handle;

    /// Get this type's underlying handle. See the [`Handle`] trait's
    /// documentation for what this represents.
    ///
    /// # Safety
    ///
    /// The caller must only use the returned handle within
    /// the lifetime of the backing resource.
    unsafe fn as_handle(&self) -> Self::Handle;

    /// Create a new reference tied to this object.
    fn as_ref(&self) -> Ref<'_, Self> {
        unsafe { Ref::from_handle(self.as_handle()) }
    }

    /// Create a new mutable reference tied to this object.
    fn as_mut(&mut self) -> RefMut<'_, Self> {
        unsafe { RefMut::from_handle(self.as_handle()) }
    }
}

pub struct Ref<'a, T: Resource> {
    pub(crate) handle: T::Handle,
    _marker: PhantomData<&'a T>,
}

impl<T: Resource> Ref<'_, T> {
    /// # Safety
    ///
    /// The returned reference's lifetime is inferred.
    /// Functions which build on this one should tie it to the owned resource.
    pub unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
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
    /// # Safety
    ///
    /// The returned reference's lifetime is inferred.
    /// Functions which build on this one should tie it to the owned resource.
    pub unsafe fn from_handle(handle: T::Handle) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
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
///     /// Represents an OS window.
///     pub struct Window<'ctx, 'vid> : SDL_Window {
///         marker: PhantomData<(Ref<'vid, Video<'ctx>>)>,
///     }
///
///     /// Destroys a window.
///     ~SDL_DestroyWindow
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
///     /// Represents a GPU texture.
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
        $(#[$doc:meta])*
        pub struct $owned:ident<$($lt:lifetime),*> : $sdl:ty {
            marker: PhantomData<($($t:ty),*)>,
        }
    ) => {
        ::paste::paste! {
            $(#[$doc])*
            #[derive(Clone, Copy)]
            #[doc(alias = "" $sdl "")]
            pub struct [<$owned Handle>]<$($lt),*> {
                handle: ::std::ptr::NonNull<$sdl>,
                marker: ::std::marker::PhantomData<$crate::resource::expand_parens!($($t),*)>
            }


            $(#[$doc])*
            impl<$($lt),*> [<$owned Handle>]<$($lt),*> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> Option<Self> {
                    ::std::ptr::NonNull::new(handle).map(|handle| Self {
                        handle,
                        marker: ::std::marker::PhantomData,
                    })
                }

                /// Get this type's "raw" representation,
                /// i.e. `*mut SDL_Surface` for [`Surface`](crate::surface::Surface),
                /// or `SDL_PropertiesID` for [`Properties`](crate::properties::Properties).
                pub fn as_raw(self) -> *mut $sdl {
                    $crate::resource::Handle::as_raw(self)
                }

                /// Get this type's "inner" representation,
                /// i.e. `NonNull<SDL_Surface>` for [`Surface`](crate::surface::Surface),
                /// or `NonZero<u32>` for [`Properties`](crate::properties::Properties).
                pub fn as_inner(self) -> ::std::ptr::NonNull<$sdl> {
                    $crate::resource::Handle::as_inner(self)
                }
            }

            $(#[$doc])*
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

                pub fn as_ref(&self) -> $crate::resource::Ref<'_, $owned<$($lt),*>> {
                    unsafe { $crate::resource::Ref::from_handle(self.inner) }
                }

                /// # Safety
                ///
                /// The caller must only use the returned handle within the lifetime
                /// of the backing resource.
                pub unsafe fn as_handle(&self) -> [<$owned Handle>]<$($lt),*> {
                    unsafe { $crate::resource::Resource::as_handle(self) }
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

                fn as_raw(self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(self) -> Self::Inner {
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
        $(#[$doc:meta])*
        pub struct $owned:ident<$($lt:lifetime),*> : $sdl:ty {
            marker: PhantomData<($($t:ty),*)>,
        }

        $(#[$doc_dtor:meta])*
        ~$dtor:ident
    ) => {
        $crate::resource::resource_new! {
            $(#[$doc])*
            pub struct $owned<$($lt),*> : $sdl {
                marker: PhantomData<($($t),*)>,
            }
        }

        ::paste::paste! {
            impl<$($lt),*> ::std::ops::Drop for $owned<$($lt),*> {
                $(#[$doc_dtor])*
                #[doc(alias = "" $dtor "")]
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    }
}

pub(crate) use {expand_parens, resource_new};
