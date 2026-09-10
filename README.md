# Sandlot

An SDL (3.4) & SDL_ttf (3.2) Rust wrapper.

## How much of a wrapper is it really?

It's somewhere between raw sdl3-sys bindings, and a higher-level graphics framework with a radically different API.
The general plan is:
1. expose the functionality of SDL,
2. minimize ways to shoot yourself in the foot.

> [!IMPORTANT]
> This crate isn't concerned with how you find, and link with, SDL and its
> satellite libraries on your system. These topics are better covered by the
> [sdl3-sys docs](https://docs.rs/sdl3-sys/latest/sdl3_sys/).

> [!NOTE]
> Functions and methods usually map 1:1 to their SDL counterparts in terms of functionality
> (in such cases, there is always a corresponding `#[doc(alias = "...")]` attribute).
> There is an effort to "pluck" documentation from the SDL wiki and map it to Rust abstractions,
> while preserving the meaning. The first pass has been done via LLMs, so there may be some slop.

## Usage

### Initialization

Most, but not all, functionality requires two things to be in scope:
- `sandlot::Context`
- a relevant subsystem (see `sandlot::subsystem`)
  - for example, `Window::new` may return `Err` if `subsystem::Video` isn't in scope.

### Objects

Since SDL works with opaque pointers, using Rust references would cause unnecessary double indirection.
To avoid this, Sandlot uses custom types with matching properties. For example:
- a `Texture` is an owned object which will `Drop` the underlying handle upon going out of scope
- `Ref<'a, Texture>` and `RefMut<'a, Texture>` mimic `&Texture` and `&mut Texture`
  - these can be obtained from an owned object via the `as_ref()` method (requires the `sandlot::resource::Resource` trait to be in scope)
  - the only difference between these two is that `Ref` only implements `Deref` for its handle, while `RefMut` also implements `DerefMut`
- a `TextureHandle` is analogous to a pointer. It exposes all methods, but is not lifetime-bound to anything

Allocations originating from SDL are wrapped in a custom implementation of `Box` and `String`.
These might not have exact 1:1 semantics with their Rust counterparts; check documentation for specifics.

## Enhancements

In an attempt to justify the time spent on this project, here is a list of things that,
in my eyes, make Sandlot much neater to use over raw SDL bindings:

- `Drop` impl'd where applicable[^1]
- `Ref` for borrowing opaque handles without extra indirection
- `sandlot::Result<T>` instead of `SDL_GetError()`
- `Box` and `String` for mapping SDL allocations to Rust
- Descriptive bool-enums instead of bool parameters
- Builders wrapping the [Properties API](https://wiki.libsdl.org/SDL3/CategoryProperties)
- Encapsulation of reserved struct parameters

[^1]: Certain SDL_gpu objects are an exception, since their destructors require an extra parameter. Such objects have a "manual" `drop()` method, and have proper documentation of this fact.
