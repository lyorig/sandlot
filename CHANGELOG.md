# What's new!

## v0.1.3
- Lifetimes!
  - several structs now have lifetimes representing the initialization hierarchy
  - this adds a LOT of lifetimes to the source code (see `RenderStateCreateInfo` for the motherlode), but also shouldn't break **well-formed** existing code.
    - for example, examples required no changes aside from function arguments
    - structs which contained multiple tied objects have become self-referential and will require changes
- Subsystems!
  - now backed by proper types, they're required to initialize relevant objects and ensure lifetime safety
  - created via `init` (as opposed to the previous `new`)

```rust
let wnd;
{
    let video = Video::init(&ctx)?;
    wnd = Window::new(video.as_ref(), /* ... */) // won't compile
}
```
  
- Usability!
  - `as_ref` and friends are now implemented directly on the type (no `trait Resource` import necessary)
- Documentation!
  - existing boolenums now have documentation
  - module doc headers are more concise to look good at [docs.rs](https://docs.rs)
  - `Drop` code has added SDL docs
  - and too many small legibility/correctness changes to count
- SDL enums
  - `SDL_FlashOperation` (`window::FlashOp`)

## v0.1.2

- Text input functions moved from `mod keyboard` to `Event`:
  - `keyboard::text_input_start` -> `Event::enable_text_input`
  - `keyboard::text_input_stop` -> `Event::disable_text_input`
  - `keyboard::is_text_input_active` -> `Event::is_text_input_active`
- `EventIter::new` moved to `Event::iter`
- Documentation for:
  - boolenums
  - resources
  - modules
- SDL enums
  - `SDL_BlendFactor` (`pixels::BlendFactor`)
  - `SDL_BlendOperation` (`pixels::BlendOperation`)
- SDL functions
  - `SDL_ComposeCustomBlendMode` (`BlendMode::compose`)
  - `SDL_PumpEvents` (`Event::pump`)
  - `SDL_WaitEvent` (`Event::wait`)
  - `SDL_GetRGB` (`RgbU8::from_pixel`)
  - `SDL_GetRGBA` (`RgbaU8::from_pixel`)
  - `SDL_MapRGB` (`RgbU8::map`)
  - `SDL_MapRGBA` (`RgbaU8::map`)
  - `SDL_GetPixelFormatForMasks` (`PixelFormat::from_mask`)
  - `SDL_GetMasksForPixelFormat` (`PixelFormat::mask`)
  - `SDL_GetPixelFormatName` (`PixelFormat::name`)
- New functions
  - `GraphicsPipeline::with`
- New structs
  - `PixelFormatMask`

## v0.1.1

- Boolenums now have `From<bool>`
- Documentation additions & fixes
