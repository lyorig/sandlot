//! Audio functionality.
//!
//! All audio in SDL3 revolves around [`AudioStream`].
//! Whether you want to play or record audio, convert it, stream it, buffer it,
//! or mix it, you pass it through an audio stream.
//!
//! Audio streams are flexible: they accept any amount of data at a time, in any
//! supported format, and output it as needed in any other format, even if the
//! data format changes on either side halfway through.
//!
//! An app opens an audio device and binds any number of audio streams to it,
//! feeding more data to the streams as available. When the device needs more
//! data, it pulls it from all bound streams and mixes them together for playback.
//!
//! Audio streams can also use an app-provided callback to supply data on demand,
//! which maps closely to the SDL2 audio model.
//!
//! SDL also provides a simple WAV loader in
//! [`SDL_LoadWAV`](https://wiki.libsdl.org/SDL3/SDL_LoadWAV) and
//! [`SDL_LoadWAV_IO`](https://wiki.libsdl.org/SDL3/SDL_LoadWAV_IO) if the data
//! is not being read from a file.
//!
//! ## Logical audio devices
//!
//! In SDL3, opening a physical device gives you a logical device ID to which
//! audio streams can be bound. In almost all cases, logical devices can be used
//! anywhere in the API that a physical device is normally used. Each device
//! opening generates a new logical device, so different parts of a program can
//! open devices independently without interfering with each other. Each logical
//! device mixes its separate audio down to a single buffer, which is fed to the
//! physical device behind the scenes.
//!
//! Logical devices also allow SDL to automatically migrate devices to different
//! hardware when the system default changes, such as when headphones are
//! connected. Playback can continue without the app needing to handle the
//! migration explicitly.
//!
//! ## Simplified audio
//!
//! For a single source of audio, an app can use
//! [`SDL_OpenAudioDeviceStream`](https://wiki.libsdl.org/SDL3/SDL_OpenAudioDeviceStream)
//! to open an audio device, create an audio stream, bind that stream to the
//! device, and optionally provide a callback for obtaining audio data. The
//! stream is the primary interface and the device handle is mostly hidden.
//!
//! Destroying a stream created through this function also closes the device,
//! and stream bindings cannot be changed. The device starts paused and must be
//! explicitly resumed. In the non-simplified form, playback starts unpaused
//! once a stream is bound to a device.
//!
//! ## Channel layouts
//!
//! Audio data passing through SDL is uncompressed, interleaved PCM data. SDL
//! does not provide decompression for formats such as MP3; an app can provide
//! its own decoder. Each interleaved channel is expected to follow a specific
//! order:
//!
//! - `FRONT`: single mono speaker
//! - `FL`: front left speaker
//! - `FR`: front right speaker
//! - `FC`: front center speaker
//! - `BL`: back left speaker
//! - `BR`: back right speaker
//! - `SR`: surround right speaker
//! - `SL`: surround left speaker
//! - `BC`: back center speaker
//! - `LFE`: low-frequency speaker
//!
//! These channels are listed in the order in which they are laid out in
//! memory. For example, `FL, FR` means that the front-left speaker is laid out
//! first, followed by the front-right speaker, and then the sequence repeats
//! for the next audio frame.
//!
//! - 1 channel (mono): `FRONT`
//! - 2 channels (stereo): `FL, FR`
//! - 3 channels (2.1): `FL, FR, LFE`
//! - 4 channels (quad): `FL, FR, BL, BR`
//! - 5 channels (4.1): `FL, FR, LFE, BL, BR`
//! - 6 channels (5.1): `FL, FR, FC, LFE, BL, BR` (the last two can also be `SL, SR`)
//! - 7 channels (6.1): `FL, FR, FC, LFE, BC, SL, SR`
//! - 8 channels (7.1): `FL, FR, FC, LFE, BL, BR, SL, SR`
//!
//! This is the same order expected by DirectSound, applied across all
//! platforms. SDL swizzles channels as necessary when a platform expects a
//! different order. [`SDL_AudioStream`](https://wiki.libsdl.org/SDL3/SDL_AudioStream)
//! can also be given channel maps to change the ordering when needed.
//!
//! # API coverage
//!
//! - [ ] SDL_AudioDevicePaused
//! - [ ] SDL_AudioStreamDevicePaused
//! - [ ] SDL_BindAudioStream
//! - [ ] SDL_BindAudioStreams
//! - [ ] SDL_ClearAudioStream
//! - [ ] SDL_CloseAudioDevice
//! - [ ] SDL_ConvertAudioSamples
//! - [ ] SDL_CreateAudioStream
//! - [x] SDL_DestroyAudioStream
//! - [x] SDL_FlushAudioStream
//! - [ ] SDL_GetAudioDeviceChannelMap
//! - [ ] SDL_GetAudioDeviceFormat
//! - [ ] SDL_GetAudioDeviceGain
//! - [ ] SDL_GetAudioDeviceName
//! - [ ] SDL_GetAudioDeviceProperties
//! - [x] SDL_GetAudioDriver
//! - [ ] SDL_GetAudioFormatName
//! - [ ] SDL_GetAudioPlaybackDevices
//! - [ ] SDL_GetAudioRecordingDevices
//! - [x] SDL_GetAudioStreamAvailable
//! - [ ] SDL_GetAudioStreamData
//! - [ ] SDL_GetAudioStreamDevice
//! - [ ] SDL_GetAudioStreamFormat
//! - [ ] SDL_GetAudioStreamFrequencyRatio
//! - [x] SDL_GetAudioStreamGain
//! - [ ] SDL_GetAudioStreamInputChannelMap
//! - [ ] SDL_GetAudioStreamOutputChannelMap
//! - [ ] SDL_GetAudioStreamProperties
//! - [ ] SDL_GetAudioStreamQueued
//! - [ ] SDL_GetCurrentAudioDriver
//! - [x] SDL_GetNumAudioDrivers
//! - [ ] SDL_GetSilenceValueForFormat
//! - [ ] SDL_IsAudioDevicePhysical
//! - [ ] SDL_IsAudioDevicePlayback
//! - [ ] SDL_LoadWAV
//! - [ ] SDL_LoadWAV_IO
//! - [x] SDL_LockAudioStream
//! - [ ] SDL_MixAudio
//! - [ ] SDL_OpenAudioDevice
//! - [ ] SDL_OpenAudioDeviceStream
//! - [ ] SDL_PauseAudioDevice
//! - [ ] SDL_PauseAudioStreamDevice
//! - [ ] SDL_PutAudioStreamData
//! - [ ] SDL_PutAudioStreamDataNoCopy
//! - [ ] SDL_PutAudioStreamPlanarData
//! - [ ] SDL_ResumeAudioDevice
//! - [ ] SDL_ResumeAudioStreamDevice
//! - [ ] SDL_SetAudioDeviceGain
//! - [ ] SDL_SetAudioPostmixCallback
//! - [ ] SDL_SetAudioStreamFormat
//! - [ ] SDL_SetAudioStreamFrequencyRatio
//! - [x] SDL_SetAudioStreamGain
//! - [ ] SDL_SetAudioStreamGetCallback
//! - [ ] SDL_SetAudioStreamInputChannelMap
//! - [ ] SDL_SetAudioStreamOutputChannelMap
//! - [ ] SDL_SetAudioStreamPutCallback
//! - [ ] SDL_UnbindAudioStream
//! - [ ] SDL_UnbindAudioStreams
//! - [x] SDL_UnlockAudioStream
//!

use std::ptr::NonNull;

use sdl3_sys::audio::*;

use crate::{Result, error::Error, resource::resource_new, str::Str, util::to_result};

/// Get the amount of built-in audio drivers.
///
/// This function returns a hardcoded number. This never returns a negative value;
/// if there are no drivers compiled into this build of SDL, this function returns zero.
/// The presence of a driver in this list does not mean it will function, it just means
/// SDL is capable of interacting with that interface. For example, a build of SDL might
/// have esound support, but if there’s no esound server available, SDL’s esound driver
/// would fail if used.
///
/// By default, SDL tries all drivers, in its preferred order, until one is found to be usable.
#[doc(alias = "SDL_GetNumAudioDrivers")]
pub fn num_drivers() -> i32 {
    unsafe { SDL_GetNumAudioDrivers() }
}

/// Get the name of a built-in audio driver.
///
/// The list of audio drivers is given in the order that they are normally initialized by default;
/// the drivers that seem more reasonable to choose first (as far as the SDL developers believe)
/// are earlier in the list.
///
/// The names of drivers are all simple, low-ASCII identifiers, like “alsa”, “coreaudio” or “wasapi”.
/// These never have Unicode characters, and are not meant to be proper names.
///
/// # Parameters
///
/// - `index`: the index of the audio driver; the value ranges from 0..[`num_drivers`] - 1.
#[doc(alias = "SDL_GetAudioDriver")]
pub fn driver(index: i32) -> Option<Str<'static>> {
    unsafe { NonNull::new(SDL_GetAudioDriver(index).cast_mut()).map(|nn| Str::from_non_null(nn)) }
}

resource_new! {
    /// An audio conversion interface.
    ///
    /// It can handle:
    /// - resampling data in chunks without generating artifacts, when it doesn’t have the complete buffer available.
    /// - incoming data in any variable size.
    /// - input/output format changes on the fly.
    /// - remapping audio channels between inputs and outputs.
    ///
    /// You push data as you have it, and pull it when you need it; the stream will
    /// buffer data as needed.
    ///
    /// It can also function as a basic audio data queue even if you just have sound
    /// that needs to pass from one place to another.
    ///
    /// You can hook callbacks up to them when more data is added or requested, to
    /// manage data on-the-fly.
    ///
    /// Audio streams are the core of the SDL3 audio interface. You create one or more of them,
    /// bind them to an opened audio device, and feed data to them (or for recording, consume data from them).
    pub struct AudioStream<> : SDL_AudioStream {
        raw: *mut SDL_AudioStream,
        inner: NonNull<SDL_AudioStream>,
        marker: PhantomData<()>,
    }

    /// Free an audio stream.
    ///
    /// This will release all allocated data, including any audio that is still queued.
    /// You do not need to manually clear the stream first.
    ///
    /// If this stream was bound to an audio device, it is unbound during this call.
    /// If this stream was created with [`SDL_OpenAudioDeviceStream`], the audio device
    /// that was opened alongside this stream’s creation will be closed, too.
    ~SDL_DestroyAudioStream
}

impl AudioStream {
    // #[doc(alias = "SDL_OpenAudioDeviceStream")]
    // pub fn open() -> Result<Self> {
    //     Self::from_raw(SDL_OpenAudioDeviceStream())
    // }
}

impl AudioStreamHandle {
    #[doc(alias = "SDL_FlushAudioStream")]
    pub fn flush(self) -> Result<()> {
        to_result(unsafe { SDL_FlushAudioStream(self.as_raw()) })
    }

    #[doc(alias = "SDL_GetAudioStreamGain")]
    pub fn gain(self) -> Result<f32> {
        let gain = unsafe { SDL_GetAudioStreamGain(self.as_raw()) };
        if gain == -1. {
            Err(Error::current())
        } else {
            Ok(gain)
        }
    }

    #[doc(alias = "SDL_GetAudioStreamAvailable")]
    pub fn available_bytes(self) -> Result<i32> {
        let av = unsafe { SDL_GetAudioStreamAvailable(self.as_raw()) };
        if av == -1 {
            Err(Error::current())
        } else {
            Ok(av)
        }
    }

    #[doc(alias = "SDL_SetAudioStreamGain")]
    pub fn set_gain(self, gain: f32) -> Result<()> {
        to_result(unsafe { SDL_SetAudioStreamGain(self.as_raw(), gain) })
    }

    #[doc(alias = "SDL_LockAudioStream")]
    fn lock(self) -> Result<()> {
        to_result(unsafe { SDL_LockAudioStream(self.as_raw()) })
    }

    #[doc(alias = "SDL_UnlockAudioStream")]
    fn unlock(self) -> Result<()> {
        to_result(unsafe { SDL_UnlockAudioStream(self.as_raw()) })
    }
}
