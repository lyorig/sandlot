//! [`Box`](std::boxed::Box), but using SDL's memory allocation API.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryStdinc)):
//! - [x] SDL_free
//! - [x] SDL_malloc

use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use sdl3_sys::stdinc::SDL_free;

use crate::{Result, util::opt2res_map};

/// Mirror of [`std::boxed::Box`], with deallocation performed
/// via [`SDL_free`] instead of the global allocator.
pub struct Box<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T: ?Sized> Box<T> {
    /// Consume the [`Box`], returning the underlying pointer without freeing
    /// the allocation.
    pub(crate) fn into_raw(self) -> *mut T {
        self.into_raw_non_null().as_ptr()
    }

    /// Like [`Self::into_raw`], but preserves the non-null invariant.
    pub(crate) fn into_raw_non_null(self) -> NonNull<T> {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }

    /// Reconstruct a [`Box`] from a pointer obtained via [`Self::into_raw`]
    /// (or [`Self::into_raw_non_null`]).
    ///
    /// # Safety
    /// `raw` must have been obtained from a [`Box`] of this module and must
    /// not have been freed.
    pub(crate) unsafe fn from_raw(raw: *mut T) -> Self {
        // SAFETY: Per the contract, `raw` is a valid, aligned, non-null
        // pointer obtained from `into_raw`.
        unsafe { Self::from_raw_non_null(NonNull::new_unchecked(raw)) }
    }

    /// Reconstruct a [`Box`] from a pointer obtained via [`Self::into_raw`]
    /// (or [`Self::into_raw_non_null`]).
    ///
    /// # Safety
    /// `raw` must have been obtained from a [`Box`] of this module and must
    /// not have been freed.
    pub(crate) unsafe fn from_raw_non_null(raw: NonNull<T>) -> Self {
        // SAFETY: Per the contract, `raw` is a valid, aligned
        // pointer obtained from `into_raw`.
        Self { ptr: raw }
    }

    /// Create a [`Box`] from an SDL allocation.
    /// Returns the current error if `raw` is null.
    ///
    /// # Safety
    /// `raw` must be allocated via SDL.
    pub(crate) unsafe fn from_raw_nullck(raw: *mut T) -> Result<Self> {
        opt2res_map(NonNull::new(raw), |ptr| Self { ptr })
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Box<[T]> {
    /// Create a [`Box`] from a pointer and a length.
    ///
    /// # Safety
    /// `ptr` must be:
    /// - obtained from an SDL allocation
    /// - valid for `len` * `size_of::<T>()` bytes.
    pub(crate) unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
        unsafe {
            let ptr = NonNull::new_unchecked(ptr);
            Self::from_raw_parts_non_null(ptr, len)
        }
    }

    /// Create a [`Box`] from a pointer and a length.
    ///
    /// # Safety
    /// `ptr` must be:
    /// - obtained from an SDL allocation
    /// - valid for `len` * `size_of::<T>()` bytes.
    pub(crate) unsafe fn from_raw_parts_non_null(ptr: NonNull<T>, len: usize) -> Self {
        let slice = NonNull::slice_from_raw_parts(ptr, len);
        Self { ptr: slice }
    }

    /// Create a [`Box`] from an SDL allocation of `len` elements.
    /// Returns the current error if `ptr` is null.
    ///
    /// # Safety
    /// `ptr` must have been obtained from an SDL allocation and must be valid
    /// for `len` * `size_of::<T>()` bytes.
    pub(crate) unsafe fn from_raw_parts_nullck(ptr: *mut T, len: usize) -> Result<Self> {
        opt2res_map(NonNull::new(ptr), |nn| unsafe {
            Self::from_raw_parts_non_null(nn, len)
        })
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The `Box` owns a valid allocation of `T`.
        unsafe { self.ptr.as_ptr().as_ref_unchecked() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The `Box` owns a valid allocation of `T`.
        unsafe { self.ptr.as_ptr().as_mut_unchecked() }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    /// Free allocated memory.
    ///
    /// The pointer is no longer valid after this call and cannot be
    /// dereferenced anymore.
    #[doc(alias = "SDL_free")]
    fn drop(&mut self) {
        unsafe { SDL_free(self.ptr.as_ptr().cast()) };
    }
}

impl<T: ?Sized> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> Borrow<T> for Box<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> fmt::Pointer for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

impl<T: Debug + ?Sized> Debug for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: Display + ?Sized> Display for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: PartialEq + ?Sized> PartialEq for Box<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq + ?Sized> Eq for Box<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Box<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    fn lt(&self, other: &Self) -> bool {
        **self < **other
    }

    fn le(&self, other: &Self) -> bool {
        **self <= **other
    }

    fn gt(&self, other: &Self) -> bool {
        **self > **other
    }

    fn ge(&self, other: &Self) -> bool {
        **self >= **other
    }
}

impl<T: Ord + ?Sized> Ord for Box<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: Hash + ?Sized> Hash for Box<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

/// Owned iterator over the elements of a `Box<[T]>`, moving them out by
/// value. Yields each element exactly once; elements not yet yielded when the
/// iterator is dropped still get their destructors run.
pub struct IntoIter<T> {
    arr: Box<[T]>,
    index: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.index == self.arr.len() {
            return None;
        }
        // SAFETY: The allocation holds `len` initialized elements and `index`
        // is below `len`, so the read is in-bounds. Each element is yielded
        // at most once, since `index` only increases.
        let ptr = self.arr.ptr.as_ptr().cast::<T>();
        let elem = unsafe { ptr.add(self.index).read() };
        self.index += 1;
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.arr.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Elements that were never yielded still need their destructors run;
        // the allocation itself is freed by the inner [`Box`].
        let ptr = self.arr.ptr.as_ptr().cast::<T>();
        for i in self.index..self.arr.len() {
            // SAFETY: `i` is within the allocation and the element at `i` has
            // not been moved out, so it is still initialized.
            unsafe { std::ptr::drop_in_place(ptr.add(i)) }
        }
    }
}

impl<T> IntoIterator for Box<[T]> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            arr: self,
            index: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a Box<[T]> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Box<[T]> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: The fat pointer's metadata is the slice length, and `self`
        // borrows the allocation mutably for `'a`.
        unsafe { &mut *self.ptr.as_ptr() }.iter_mut()
    }
}
