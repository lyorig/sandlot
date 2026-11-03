# Performance considerations

This is a compendium of things worth knowing about before coding your app using Sandlot.

## Builder property cleanup

When building an object, you can finish the process using one of two methods, `build` or `build_cleanup`.

`build` maps 1:1 to SDL's `CreateXyzWithProperties`, so it **doesn't clear the properties**.
This is fine when building with a scoped property group which'll be destroyed not long after the object is built:

```rust
fn create_window<'ctx, 'vid>(vid: Ref<'vid, Video<'ctx>>) -> Result<Window<'ctx, 'vid>> {
    let props = Properties::new()?;
    Window::builder(props.as_ref())
        /* options... */
        .build(vid)

    // `props` is destroyed here, along with its keys
}
```

However, if you have a longer-lived property group, using `build` will leave lingering
key-value pairs set inside of it. This will:

- waste memory
- affect future builds using the same group

Remember, under the hood, builder methods are **setting key-value pairs in a hash map**.

## Global properties

Each `Properties` object's construction and destruction involves a non-zero amount of work.
The usual thing to do is use a `Properties` for every object builder, but we can do better!

The thing is, SDL initializes a global property group upon startup (`Context::global_properties`),
whether you like it or not. Various platform-specific settings are stored there by SDL, but never
any builder options (`SDL_PROP_CREATE_*`). You might as well use it for building objects,
saving yourself a property creation/destruction call.

I eagerly recommend the aforementioned `build_cleanup` here, since each builder's options will otherwise
remain in the global property group until the `Context` is dropped.

```rust
let gp = ctx.global_properties()?; // Ref<'ctx, Properties>
let wnd = Window::builder(gp)
    /* options... */
    .build_cleanup()?;
```
