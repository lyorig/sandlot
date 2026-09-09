# Sandlot

An SDL & SDL_ttf (3.x) wrapper. Aims for as close to 100% API coverage, while making it neater & safer to use via various Rust mechanisms (see _Enhancements_).
As I'm primarily a C++ developer, this library is probably unsound in various places. These ought to be weeded out over time after reaching full API coverage.

> [!IMPORTANT]
> This is purely an API wrapper. It isn't concerned with how you find, and link with,
> SDL and its satellite libraries on your system; these topics are covered by the
> [sdl3-sys docs](https://docs.rs/sdl3-sys/latest/sdl3_sys/).

> [!NOTE]
> Functions and methods usually map 1:1 to their SDL counterparts in terms of functionality.
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
