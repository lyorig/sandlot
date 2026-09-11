/// Hack around `#[warn(unused_parens)]`.
macro_rules! expand_parens {
    ($t:ty) => {
        $t
    };
    ($($t:ty),+) => {
        ($($t),*)
    };
}

macro_rules! resource_new {
    (
        $(#[$meta:meta])*
        pub struct $owned:ident<$($lt:lifetime),*> : $sdl:ty, $dtor:ty {
            marker: PhantomData<($($t:ty),*)>,
        }
    ) => {
        ::paste::paste! {
            $(#[$meta])*
            #[derive(Clone, Copy)]
            pub struct [<$owned Handle>]<$($lt),*> {
                handle: ::std::ptr::NonNull<$sdl>,
                marker: ::std::marker::PhantomData<expand_parens!($($t),*)>
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
            /// TODO: Add doc alias.
            pub struct $owned<$($lt),*> {
                pub(crate) inner: [<$owned Handle>]<$($lt),*>
            }

            impl<$($lt),*> $owned<$($lt),*> {
                pub(crate) fn from_ptr(handle: *mut $sdl) -> crate::Result<Self> {
                    match ::std::ptr::NonNull::new(handle) {
                        Some(handle) => Ok(Self {
                            inner: [<$owned Handle>] {
                                handle,
                                marker: ::std::marker::PhantomData,
                            },
                        }),
                        None => Err(crate::error::Error::current()),
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

            impl<$($lt),*> crate::resource::Handle for [<$owned Handle>]<$($lt),*> {
                type Raw = *mut $sdl;
                type Inner = ::std::ptr::NonNull<$sdl>;

                fn as_raw(&self) -> Self::Raw {
                    self.handle.as_ptr()
                }

                fn as_inner(&self) -> Self::Inner {
                    self.handle
                }
            }

            impl<$($lt),*> crate::resource::Resource for $owned<$($lt),*> {
                type Handle = [<$owned Handle>]<$($lt),*>;

                unsafe fn as_handle(&self) -> Self::Handle {
                    self.inner
                }
            }

            impl<$($lt),*> ::std::ops::Drop for $owned<$($lt),*> {
                /// TODO: Add doc alias.
                fn drop(&mut self) {
                    unsafe { $dtor(self.inner.handle.as_ptr()) }
                }
            }
        }
    };
}

resource_new! {
    pub struct Test<'ctx, 'vid> : sdl3_sys::video::SDL_Window, sdl3_sys::video::SDL_DestroyWindow {
        marker: PhantomData<(crate::init::Ref<'vid, crate::init::Video<'ctx>>)>,
    }
}
