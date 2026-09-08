# SDL documentation import

Your task is to add SDL's documentation to sandlot wrappers in a manner that "maps" it to Rust terms while preserving *what the SDL function does*.
Specifically, for each function/method with a `#[doc(alias = ...)]` attribute, its docblock should contain _relevant_ documentation from the SDL wiki.
This operation should only affect docblocks of the items mentioned below; if any further modifications are necessary, ask for permission first.

## What this applies to

- Functions
- Enums
- Constants
- Structs

## What this does NOT apply to

- Macro-generated types (e.g. via `resource_new!`)
- Items with extensive documentation already present (try to "merge" smaller docblocks into the SDL docs)

## What "relevant" means

You should include, in this order:

- the function summary
- parameters' descriptions/meanings
- platform-specific behavior
- error conditions, if they're specific
- lifetime rules, if there are any that are not encapsulated via Rust (e.g. RAII)
- remarks, if they aren't something C-specific that Rust takes care of

You should NOT include:

- thread safety
- version ("available from")

Enums' variants should all get their individual docblock as well, if SDL provides them.

Constants' docs are located on their parent enum's page, e.g. `SDL_GlobFlags`.

## Where to fetch docs from

The following links provide raw Markdown. Substitute {item} for a function/enum/etc.
If a doc isn't available on the wiki, leave the docblock as `/// FIXME(doc): Not found on wiki.`

For SDL: https://wiki.libsdl.org/SDL3/{item}/raw
For SDL_ttf: https://wiki.libsdl.org/SDL3_ttf/{item}/raw

## Style rules
- Convert wiki links to plain-code formatting, so as to prevent `rustdoc::broken_intra_doc_links`.
- If a docblock references another SDL function, change it to the sandlot equivalent.
  - For example, `SDL_WaitAndAcquireGPUSwapchainTexture` would become `CommandBufferHandle::wait_for_swapchain_texture`.
- Use intra-doc links wherever possible. If the name isn't imported, import it with an `#[allow(dead_code)]` attribute.

## Testing

Run `cargo check --all-targets` and `cargo doc --no-deps` to verify your changes.

## Examples

```rust
/// Get the name of the platform.
///
/// ## Remarks
/// Here are the names returned for some (but not all) supported platforms:
/// - "Windows"
/// - "macOS"
/// - "Linux"
/// - "iOS"
/// - "Android"
#[doc(alias = "SDL_GetPlatform")]
pub fn platform() -> &'static str {
    // SAFETY: All SDL3 platform strings are UTF-8,
    // and are stored statically.
    unsafe { c_ptr_to_str(SDL_GetPlatform()) }
}
```

```rust
/// Query whether there is data in the clipboard for the provided MIME type.
#[doc(alias = "SDL_HasClipboardData")]
pub fn has_data(mime_type: &CStr) -> bool {
    unsafe { SDL_HasClipboardData(mime_type.as_ptr()) }
}
```

```rust
/// Blocks the thread until a swapchain texture is available to be acquired, and then acquires it.
/// Returns [`Ok(None)`] if the swapchain texture is unavailable, e.g. when the
/// window is minimized (this is not an error!).
/// 
/// ## Remarks
/// 
/// When a swapchain texture is acquired on a command buffer, it will automatically be submitted
/// for presentation when the command buffer is submitted. The swapchain texture should only be referenced
/// by the command buffer used to acquire it. It is an error to call this method after a swapchain texture is acquired.
///
/// The swapchain texture is:
/// - managed by the implementation and must not be freed by the user
/// - write-only and cannot be used as a sampler or for another reading operation
#[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
pub fn wait_for_swapchain_texture(
    &self,
    wnd: Ref<Window>,
    (tex_x, tex_y): (Option<&mut u32>, Option<&mut u32>),
) -> Result<Option<Ref<'_, Texture>>> {
    /* ... */
}
```
