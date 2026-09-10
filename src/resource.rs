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
