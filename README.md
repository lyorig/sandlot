# Sandlot

An SDL (3.4) & SDL_ttf (3.2) Rust wrapper.

## How much of a wrapper is it really?

It's somewhere between raw sdl3-sys bindings, and a higher-level graphics framework with a radically different API.
The general plan is:
1. expose the functionality of SDL,
2. minimize ways to shoot yourself in the foot.

So it's definitely quite a [leaky abstraction](https://www.joelonsoftware.com/2002/11/11/the-law-of-leaky-abstractions/).

> [!IMPORTANT]
> This crate isn't concerned with how you find, and link with, SDL and its
> satellite libraries on your system; these topics are covered by the
> [sdl3-sys docs](https://docs.rs/sdl3-sys/latest/sdl3_sys/).

> [!NOTE]
> Functions and methods usually map 1:1 to their SDL counterparts in terms of functionality.
> In such cases, there is always a corresponding `#[doc(alias = "...")]` attribute.
> There is an effort to "pluck" documentation from the SDL wiki and map it to Rust abstractions,
> while preserving the meaning. The first pass has been done via LLMs, so there may be some slop.

## Concepts

### Initialization

As with many modern APIs, `sandlot::Context` is the first struct you'll want to create for a proper application.
Afterwards, you can initialize subsystems (see the `sandlot::subsystem` module), whose existence permits creation
of relevant objects etc.

### Objects

SDL works with raw pointers and ownership rules are mostly described via function documentation.
Sandlot maps this to Rust terms with _handles_, _owned objects_ and _references_. For an arbitrary type `Foo`:
- `FooHandle` is where the API is actually implemented. Since it isn't tied to anything, it's usually unsafe to obtain and use.
- `Foo` is an owned object containing a handle, being responsible for `Drop`ping it.
- `Ref<'a, Foo>` and `RefMut<'a, Foo>` contain a handle, are lifetime-bound to an owned object, and don't drop anything.
  - These can be obtained from an owned object via the `as_ref()` method (requires the `sandlot::resource::Resource` trait to be in scope).
  - The only difference between these two is that `Ref` only implements `Deref` for its handle, while `RefMut` also implements `DerefMut`.

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
