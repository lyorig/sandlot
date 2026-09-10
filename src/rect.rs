use std::{fmt::Display, ops::Mul, ptr};

use sdl3_sys::rect::*;

/// Wrapper around `SDL_(F)Point`, can be transmuted.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn map<U: Copy, F: Fn(T) -> U>(self, f: F) -> Point<U> {
        Point::new(f(self.x), f(self.y))
    }
}

pub type PointI32 = Point<i32>;
impl PointI32 {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn to_f32(self) -> PointF32 {
        Point::new(self.x as f32, self.y as f32)
    }

    pub const fn as_sdl_ptr(&self) -> *const SDL_Point {
        ptr::from_ref(self).cast()
    }

    pub const fn from_sdl(rect: SDL_Point) -> Self {
        unsafe { std::mem::transmute(rect) }
    }
}

pub type PointF32 = Point<f32>;
impl PointF32 {
    pub const ZERO: Self = Self::new(0., 0.);

    pub const fn to_i32(self) -> PointI32 {
        Point::new(self.x as i32, self.y as i32)
    }

    pub const fn as_sdl_ptr(&self) -> *const SDL_FPoint {
        ptr::from_ref(self).cast()
    }

    pub const fn from_sdl(rect: SDL_FPoint) -> Self {
        unsafe { std::mem::transmute(rect) }
    }
}

impl From<PointI32> for PointF32 {
    fn from(value: PointI32) -> Self {
        value.to_f32()
    }
}

impl From<PointF32> for PointI32 {
    fn from(value: PointF32) -> Self {
        value.to_i32()
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Point<T> {
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{},{}]", self.x, self.y)
    }
}

/// Wrapper around `SDL_(F)Rect`, can be transmuted.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rect<T> {
    pub pos: Point<T>,
    pub size: Point<T>,
}

impl<T: Copy> Rect<T> {
    pub const fn new(pos: Point<T>, size: Point<T>) -> Self {
        Self { pos, size }
    }

    /// Create a [`Rect`] with all fields specified.
    pub const fn xywh(x: T, y: T, w: T, h: T) -> Self {
        Self::new(Point::new(x, y), Point::new(w, h))
    }

    pub fn map<U: Copy, F: Fn(T) -> U + Copy>(self, f: F) -> Rect<U> {
        Rect::new(self.pos.map(f), self.size.map(f))
    }
}

impl<T: Copy + Default> Rect<T> {
    /// Convenience function, identical to `Rect::xywh(x, y, T::default(), T::default())`.
    pub fn xy(x: T, y: T) -> Self {
        Self::xywh(x, y, T::default(), T::default())
    }

    /// Convenience function, identical to `Rect::xywh(T::default(), T::default(), w, h)`.
    pub fn wh(w: T, h: T) -> Self {
        Self::xywh(T::default(), T::default(), w, h)
    }
}

pub type RectI32 = Rect<i32>;
impl RectI32 {
    pub const ZEROED: Self = Self::xywh(0, 0, 0, 0);

    pub const fn to_f32(self) -> RectF32 {
        RectF32::new(self.pos.to_f32(), self.size.to_f32())
    }

    pub const fn as_sdl_ptr(&self) -> *const SDL_Rect {
        ptr::from_ref(self).cast()
    }

    pub const fn from_sdl(rect: SDL_Rect) -> Self {
        unsafe { std::mem::transmute(rect) }
    }
}

pub type RectF32 = Rect<f32>;
impl RectF32 {
    pub const ZEROED: Self = Self::xywh(0., 0., 0., 0.);

    pub const fn to_i32(self) -> RectI32 {
        RectI32::new(self.pos.to_i32(), self.size.to_i32())
    }

    pub const fn as_sdl_ptr(&self) -> *const SDL_FRect {
        ptr::from_ref(self).cast()
    }

    pub const fn from_sdl(rect: SDL_FRect) -> Self {
        unsafe { std::mem::transmute(rect) }
    }
}

impl From<RectI32> for RectF32 {
    fn from(value: RectI32) -> Self {
        value.to_f32()
    }
}

impl From<RectF32> for RectI32 {
    fn from(value: RectF32) -> Self {
        value.to_i32()
    }
}

impl<T: Display> Display for Rect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {}x{})", self.pos, self.size.x, self.size.y)
    }
}
