# What's new!

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
