//! Event polling, pushing, etc.
//!
//! Implementation checklist:
//! - [x] SDL_PollEvent
//! - [x] SDL_PushEvent
//! - [x] SDL_StartTextInput
//! - [ ] SDL_StartTextInputWithProperties
//! - [x] SDL_StopTextInput
//! - [x] SDL_TextInputActive
//! - [ ] SDL_SetTextInputArea

use std::{iter::FusedIterator, mem::MaybeUninit, ptr};

use sdl3_sys::{
    events::*,
    keyboard::{SDL_StartTextInput, SDL_StopTextInput, SDL_TextInputActive},
};

use crate::{Result, resource::Ref, util::to_result, window::Window};

/// NOTE: Documentation for variants is copied from SDL.
/// It might not make sense in the context of this crate.
///
/// Furthermore, inner event structures ([`SDL_WindowEvent`] etc.)
/// are currently not wrapped and exposed directly as-is.
#[repr(C, u32)]
#[doc(alias = "SDL_Event")]
pub enum Event {
    /// User-requested quit
    Quit = SDL_EventType::QUIT.0,

    /// These application events have special meaning on iOS and Android, see `README-ios.md` and `README-android.md` for details.
    /// The application is being terminated by the OS. This event must be handled in a callback set with `SDL_AddEventWatch()`.
    /// Called on iOS in `applicationWillTerminate()`
    /// Called on Android in `onDestroy()`
    Terminating = SDL_EventType::TERMINATING.0,

    /// The application is low on memory, free memory if possible. This event must be handled in a callback set with `SDL_AddEventWatch()`.
    /// Called on iOS in `applicationDidReceiveMemoryWarning()`
    /// Called on Android in `onTrimMemory()`
    LowMemory = SDL_EventType::LOW_MEMORY.0,

    /// The application is about to enter the background. This event must be handled in a callback set with `SDL_AddEventWatch()`.
    /// Called on iOS in `applicationWillResignActive()`
    /// Called on Android in `onPause()`
    WillEnterBackground = SDL_EventType::WILL_ENTER_BACKGROUND.0,

    /// The application did enter the background and may not get CPU for some time. This event must be handled in a callback set with `SDL_AddEventWatch()`.
    /// Called on iOS in `applicationDidEnterBackground()`
    /// Called on Android in `onPause()`
    DidEnterBackground = SDL_EventType::DID_ENTER_BACKGROUND.0,

    /// The application is about to enter the foreground. This event must be handled in a callback set with `SDL_AddEventWatch()`.
    /// Called on iOS in `applicationWillEnterForeground()`
    /// Called on Android in `onResume()`
    WillEnterForeground = SDL_EventType::WILL_ENTER_FOREGROUND.0,

    /// The application is now interactive. This event must be handled in a callback set with SDL_AddEventWatch().
    /// Called on iOS in applicationDidBecomeActive()
    /// Called on Android in onResume()
    DidEnterForeground = SDL_EventType::DID_ENTER_FOREGROUND.0,

    /// The user's locale preferences have changed.
    LocaleChanged = SDL_EventType::LOCALE_CHANGED.0,

    /* Display events */
    /// The system theme changed
    SystemThemeChanged = SDL_EventType::SYSTEM_THEME_CHANGED.0,

    /// Display orientation has changed to data1
    DisplayOrientationChanged(SDL_DisplayEvent) = SDL_EventType::DISPLAY_ORIENTATION.0,

    /// Display has been added to the system
    DisplayAdded(SDL_DisplayEvent) = SDL_EventType::DISPLAY_ADDED.0,

    /// Display has been removed from the system
    DisplayRemoved(SDL_DisplayEvent) = SDL_EventType::DISPLAY_REMOVED.0,

    /// Display has changed position
    DisplayMoved(SDL_DisplayEvent) = SDL_EventType::DISPLAY_MOVED.0,

    /// Display has changed desktop mode
    DisplayDesktopModeChanged(SDL_DisplayEvent) = SDL_EventType::DISPLAY_DESKTOP_MODE_CHANGED.0,

    /// Display has changed current mode
    DisplayCurrentModeChanged(SDL_DisplayEvent) = SDL_EventType::DISPLAY_CURRENT_MODE_CHANGED.0,

    /// Display has changed content scale
    DisplayContentScaleChanged(SDL_DisplayEvent) = SDL_EventType::DISPLAY_CONTENT_SCALE_CHANGED.0,

    /* Window events */
    /// Window has been shown
    WindowShown(SDL_WindowEvent) = SDL_EventType::WINDOW_SHOWN.0,

    /// Window has been hidden
    WindowHidden(SDL_WindowEvent) = SDL_EventType::WINDOW_HIDDEN.0,

    /// Window has been exposed and should be redrawn, and can be redrawn
    /// directly from event watchers for this event. data1 is 1 for
    /// live-resize expose events, 0 otherwise.
    WindowExposed(SDL_WindowEvent) = SDL_EventType::WINDOW_EXPOSED.0,

    /// Window has been moved to data1, data2
    WindowMoved(SDL_WindowEvent) = SDL_EventType::WINDOW_MOVED.0,

    /// Window has been resized to data1xdata2
    WindowResized(SDL_WindowEvent) = SDL_EventType::WINDOW_RESIZED.0,

    /// The pixel size of the window has changed to data1xdata2
    WindowPixelSizeChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_PIXEL_SIZE_CHANGED.0,

    /// The pixel size of a Metal view associated with the window has changed
    WindowMetalViewResized(SDL_WindowEvent) = SDL_EventType::WINDOW_METAL_VIEW_RESIZED.0,

    /// Window has been minimized
    WindowMinimized(SDL_WindowEvent) = SDL_EventType::WINDOW_MINIMIZED.0,

    /// Window has been maximized
    WindowMaximized(SDL_WindowEvent) = SDL_EventType::WINDOW_MAXIMIZED.0,

    /// Window has been restored to normal size and position
    WindowRestored(SDL_WindowEvent) = SDL_EventType::WINDOW_RESTORED.0,

    /// Window has gained mouse focus
    WindowMouseEnter(SDL_WindowEvent) = SDL_EventType::WINDOW_MOUSE_ENTER.0,

    /// Window has lost mouse focus
    WindowMouseLeave(SDL_WindowEvent) = SDL_EventType::WINDOW_MOUSE_LEAVE.0,

    /// Window has gained keyboard focus
    WindowFocusGained(SDL_WindowEvent) = SDL_EventType::WINDOW_FOCUS_GAINED.0,

    /// Window has lost keyboard focus
    WindowFocusLost(SDL_WindowEvent) = SDL_EventType::WINDOW_FOCUS_LOST.0,

    /// The window manager requests that the window be closed
    WindowCloseRequested(SDL_WindowEvent) = SDL_EventType::WINDOW_CLOSE_REQUESTED.0,

    /// Window had a hit test that wasn't SDL_HITTEST_NORMAL
    WindowHitTest(SDL_WindowEvent) = SDL_EventType::WINDOW_HIT_TEST.0,

    /// The ICC profile of the window's display has changed
    WindowIccProfileChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_ICCPROF_CHANGED.0,

    /// Window has been moved to display data1
    WindowDisplayChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_DISPLAY_CHANGED.0,

    /// Window display scale has been changed
    WindowDisplayScaleChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_DISPLAY_SCALE_CHANGED.0,

    /// The window safe area has been changed
    WindowSafeAreaChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_SAFE_AREA_CHANGED.0,

    /// The window has been occluded
    WindowOccluded(SDL_WindowEvent) = SDL_EventType::WINDOW_OCCLUDED.0,

    /// The window has entered fullscreen mode
    WindowEnteredFullscreen(SDL_WindowEvent) = SDL_EventType::WINDOW_ENTER_FULLSCREEN.0,

    /// The window has left fullscreen mode
    WindowLeftFullscreen(SDL_WindowEvent) = SDL_EventType::WINDOW_LEAVE_FULLSCREEN.0,

    /// The window with the associated ID is being or has been destroyed. If this message is being handled
    /// in an event watcher, the window handle is still valid and can still be used to retrieve any properties
    /// associated with the window. Otherwise, the handle has already been destroyed and all resources
    /// associated with it are invalid
    WindowDestroyed(SDL_WindowEvent) = SDL_EventType::WINDOW_DESTROYED.0,

    /// Window HDR properties have changed
    WindowHdrStateChanged(SDL_WindowEvent) = SDL_EventType::WINDOW_HDR_STATE_CHANGED.0,

    /* Keyboard events */
    /// Key pressed
    KeyDown(SDL_KeyboardEvent) = SDL_EventType::KEY_DOWN.0,

    /// Key released
    KeyUp(SDL_KeyboardEvent) = SDL_EventType::KEY_UP.0,

    /// Keyboard text editing (composition)
    TextEditing(SDL_TextEditingEvent) = SDL_EventType::TEXT_EDITING.0,

    /// Keyboard text input
    TextInput(SDL_TextInputEvent) = SDL_EventType::TEXT_INPUT.0,

    /// Keymap changed due to a system event such as an input language or keyboard layout change.
    KeymapChanged = SDL_EventType::KEYMAP_CHANGED.0,

    /// A new keyboard has been inserted into the system
    KeyboardAdded(SDL_KeyboardDeviceEvent) = SDL_EventType::KEYBOARD_ADDED.0,

    /// A keyboard has been removed
    KeyboardRemoved(SDL_KeyboardDeviceEvent) = SDL_EventType::KEYBOARD_REMOVED.0,

    /// Keyboard text editing candidates
    TextEditingCandidates(SDL_TextEditingCandidatesEvent) =
        SDL_EventType::TEXT_EDITING_CANDIDATES.0,

    /* Mouse events */
    /// Mouse moved
    MouseMotion(SDL_MouseMotionEvent) = SDL_EventType::MOUSE_MOTION.0,

    /// Mouse button pressed
    MouseButtonDown(SDL_MouseButtonEvent) = SDL_EventType::MOUSE_BUTTON_DOWN.0,

    /// Mouse button released
    MouseButtonUp(SDL_MouseButtonEvent) = SDL_EventType::MOUSE_BUTTON_UP.0,

    /// Mouse wheel motion
    MouseWheelMotion(SDL_MouseWheelEvent) = SDL_EventType::MOUSE_WHEEL.0,

    /// A new mouse has been inserted into the system
    MouseAdded(SDL_MouseDeviceEvent) = SDL_EventType::MOUSE_ADDED.0,

    /// A mouse has been removed
    MouseRemoved(SDL_MouseDeviceEvent) = SDL_EventType::MOUSE_REMOVED.0,

    /* Joystick events */
    /// Joystick axis motion
    JoyAxisMotion(SDL_JoyAxisEvent) = SDL_EventType::JOYSTICK_AXIS_MOTION.0,

    /// Joystick trackball motion
    JoyBallMotion(SDL_JoyBallEvent) = SDL_EventType::JOYSTICK_BALL_MOTION.0,

    /// Joystick hat position change
    JoyHatMotion(SDL_JoyHatEvent) = SDL_EventType::JOYSTICK_HAT_MOTION.0,

    /// Joystick button pressed
    JoyButtonDown(SDL_JoyButtonEvent) = SDL_EventType::JOYSTICK_BUTTON_DOWN.0,

    /// Joystick button released
    JoyButtonUp(SDL_JoyButtonEvent) = SDL_EventType::JOYSTICK_BUTTON_UP.0,

    /// A new joystick has been inserted into the system
    JoyAdded(SDL_JoyDeviceEvent) = SDL_EventType::JOYSTICK_ADDED.0,

    /// An opened joystick has been removed
    JoyRemoved(SDL_JoyDeviceEvent) = SDL_EventType::JOYSTICK_REMOVED.0,

    /// Joystick battery level change
    JoyBatteryUpdate(SDL_JoyBatteryEvent) = SDL_EventType::JOYSTICK_BATTERY_UPDATED.0,

    /// Joystick update is complete
    JoyUpdateComplete(SDL_JoyDeviceEvent) = SDL_EventType::JOYSTICK_UPDATE_COMPLETE.0,

    /* Gamepad events */
    /// Gamepad axis motion
    GamepadAxisMotion(SDL_GamepadAxisEvent) = SDL_EventType::GAMEPAD_AXIS_MOTION.0,

    /// Gamepad button pressed
    GamepadButtonDown(SDL_GamepadButtonEvent) = SDL_EventType::GAMEPAD_BUTTON_DOWN.0,

    /// Gamepad button released
    GamepadButtonUp(SDL_GamepadButtonEvent) = SDL_EventType::GAMEPAD_BUTTON_UP.0,

    /// A new gamepad has been inserted into the system
    GamepadAdded(SDL_GamepadDeviceEvent) = SDL_EventType::GAMEPAD_ADDED.0,

    /// A gamepad has been removed
    GamepadRemoved(SDL_GamepadDeviceEvent) = SDL_EventType::GAMEPAD_REMOVED.0,

    /// The gamepad mapping was updated
    GamepadRemapped(SDL_GamepadDeviceEvent) = SDL_EventType::GAMEPAD_REMAPPED.0,

    /// Gamepad touchpad was touched
    GamepadTouchpadDown(SDL_GamepadTouchpadEvent) = SDL_EventType::GAMEPAD_TOUCHPAD_DOWN.0,

    /// Gamepad touchpad finger was moved
    GamepadTouchpadMotion(SDL_GamepadTouchpadEvent) = SDL_EventType::GAMEPAD_TOUCHPAD_MOTION.0,

    /// Gamepad touchpad finger was lifted
    GamepadTouchpadUp(SDL_GamepadTouchpadEvent) = SDL_EventType::GAMEPAD_TOUCHPAD_UP.0,

    /// Gamepad sensor was updated
    GamepadSensorUpdate(SDL_GamepadSensorEvent) = SDL_EventType::GAMEPAD_SENSOR_UPDATE.0,

    /// Gamepad update is complete
    GamepadUpdateComplete(SDL_GamepadDeviceEvent) = SDL_EventType::GAMEPAD_UPDATE_COMPLETE.0,

    /// Gamepad Steam handle has changed
    GamepadSteamHandleUpdated(SDL_GamepadDeviceEvent) =
        SDL_EventType::GAMEPAD_STEAM_HANDLE_UPDATED.0,

    /* Touch events */
    FingerDown(SDL_TouchFingerEvent) = SDL_EventType::FINGER_DOWN.0,
    FingerUp(SDL_TouchFingerEvent) = SDL_EventType::FINGER_UP.0,
    FingerMotion(SDL_TouchFingerEvent) = SDL_EventType::FINGER_MOTION.0,
    FingerCancelled(SDL_TouchFingerEvent) = SDL_EventType::FINGER_CANCELED.0,

    /* Clipboard events */
    /// The clipboard or primary selection changed
    ClipboardUpdate(SDL_ClipboardEvent) = SDL_EventType::CLIPBOARD_UPDATE.0,

    /* Drag and drop events */
    /// The system requests a file open
    DropFile(SDL_DropEvent) = SDL_EventType::DROP_FILE.0,

    /// text/plain drag-and-drop event
    DropText(SDL_DropEvent) = SDL_EventType::DROP_TEXT.0,

    /// A new set of drops is beginning (NULL filename)
    DropBegin(SDL_DropEvent) = SDL_EventType::DROP_BEGIN.0,

    /// Current set of drops is now complete (NULL filename)
    DropComplete(SDL_DropEvent) = SDL_EventType::DROP_COMPLETE.0,

    /// Position while moving over the window
    DropPosition(SDL_DropEvent) = SDL_EventType::DROP_POSITION.0,

    /* Audio hotplug events */
    /// A new audio device is available
    AudioDeviceAdded(SDL_AudioDeviceEvent) = SDL_EventType::AUDIO_DEVICE_ADDED.0,

    /// An audio device has been removed.
    AudioDeviceRemoved(SDL_AudioDeviceEvent) = SDL_EventType::AUDIO_DEVICE_REMOVED.0,

    /// An audio device's format has been changed by the system.
    AudioDeviceFormatChanged(SDL_AudioDeviceEvent) = SDL_EventType::AUDIO_DEVICE_FORMAT_CHANGED.0,

    /* Sensor events */
    /// A sensor was updated
    SensorUpdate(SDL_SensorEvent) = SDL_EventType::SENSOR_UPDATE.0,

    /* Pressure-sensitive pen events */
    /// Pressure-sensitive pen has become available
    PenProximityIn(SDL_PenProximityEvent) = SDL_EventType::PEN_PROXIMITY_IN.0,

    /// Pressure-sensitive pen has become unavailable
    PenProximityOut(SDL_PenProximityEvent) = SDL_EventType::PEN_PROXIMITY_OUT.0,

    /// Pressure-sensitive pen touched drawing surface
    PenDown(SDL_PenTouchEvent) = SDL_EventType::PEN_DOWN.0,

    /// Pressure-sensitive pen stopped touching drawing surface
    PenUp(SDL_PenTouchEvent) = SDL_EventType::PEN_UP.0,

    /// Pressure-sensitive pen button pressed
    PenButtonDown(SDL_PenButtonEvent) = SDL_EventType::PEN_BUTTON_DOWN.0,

    /// Pressure-sensitive pen button released
    PenButtonUp(SDL_PenButtonEvent) = SDL_EventType::PEN_BUTTON_UP.0,

    /// Pressure-sensitive pen is moving on the tablet
    PenMotion(SDL_PenMotionEvent) = SDL_EventType::PEN_MOTION.0,

    /// Pressure-sensitive pen angle/pressure/etc changed
    PenAxis(SDL_PenAxisEvent) = SDL_EventType::PEN_AXIS.0,

    /* Camera hotplug events */
    /// A new camera device is available
    CameraDeviceAdded(SDL_CameraDeviceEvent) = SDL_EventType::CAMERA_DEVICE_ADDED.0,

    /// A camera device has been removed.
    CameraDeviceRemoved(SDL_CameraDeviceEvent) = SDL_EventType::CAMERA_DEVICE_REMOVED.0,

    /// A camera device has been approved for use by the user.
    CameraDeviceApproved(SDL_CameraDeviceEvent) = SDL_EventType::CAMERA_DEVICE_APPROVED.0,

    /// A camera device has been denied for use by the user.
    CameraDeviceDenied(SDL_CameraDeviceEvent) = SDL_EventType::CAMERA_DEVICE_DENIED.0,

    /* Render events */
    /// The render targets have been reset and their contents need to be updated
    RenderTargetsReset(SDL_RenderEvent) = SDL_EventType::RENDER_TARGETS_RESET.0,

    /// The device has been reset and all textures need to be recreated
    RenderDeviceReset(SDL_RenderEvent) = SDL_EventType::RENDER_DEVICE_RESET.0,

    /// The device has been lost and can't be recovered
    RenderDeviceLost(SDL_RenderEvent) = SDL_EventType::RENDER_DEVICE_LOST.0,
}

impl Event {
    /// Add an event to the event queue.
    ///
    /// The event is copied into the queue.
    ///
    /// Returns [`Err`] if the event was filtered or on failure; a common
    /// reason for error is the event queue being full.
    ///
    /// # Remarks
    ///
    /// The event queue can actually be used as a two way communication
    /// channel. Not only can events be read from the queue, but the user can
    /// also push their own events onto it.
    ///
    /// Note: Pushing device input events onto the queue doesn't modify the
    /// state of the device within SDL.
    ///
    /// Note: Events pushed onto the queue get passed through the event
    /// filter.
    ///
    /// For pushing application-specific events, please use
    /// `SDL_RegisterEvents` to get an event type that does not conflict with
    /// other code that also wants its own custom event types.
    #[doc(alias = "SDL_PushEvent")]
    pub fn push(&self) -> Result<()> {
        // NOTE: The timestamp is set internally in `SDL_PushEvent()`.
        let mut e = SDL_Event::from(self);
        to_result(unsafe { SDL_PushEvent(&raw mut e) })
    }

    /// Sets the event timestamp. SDL recommends obtaining the value via [`crate::ticks_ns`].
    /// If you do not set the timestamp yourself, [`Self::push`] sets it internally to [`crate::ticks_ns`].
    pub fn set_timestamp(&mut self, ts: u64) {
        // The fields in `SDL_CommonEvent` are shared by all variants,
        // so it's always safe to read/write.
        let mut ptr = ptr::from_mut(self).cast::<SDL_CommonEvent>();
        ptr = unsafe { ptr.byte_add(DISCRIMINANT_OFFSET) };
        let common = unsafe { ptr.as_mut_unchecked() };

        common.timestamp = ts;
    }

    /// Start accepting Unicode text input events in a window.
    ///
    /// # Remarks
    ///
    /// This function will enable text input ([`Event::TextInput`] and
    /// [`Event::TextEditing`] events) in the specified window. Please use
    /// this function paired with [`Event::disable_text_input`].
    ///
    /// Text input events are not received by default.
    ///
    /// On some platforms using this function shows the screen keyboard and/or
    /// activates an IME, which can prevent some key press events from being
    /// passed through.
    #[doc(alias = "SDL_StartTextInput")]
    pub fn enable_text_input(wnd: Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StartTextInput(wnd.handle.as_ptr()) })
    }

    /// Stop receiving any text input events in a window.
    ///
    /// # Remarks
    ///
    /// If [`Event::enable_text_input`] showed the screen keyboard,
    /// this function will hide it.
    #[doc(alias = "SDL_StopTextInput")]
    pub fn disable_text_input(wnd: Ref<Window>) -> Result<()> {
        to_result(unsafe { SDL_StopTextInput(wnd.handle.as_ptr()) })
    }

    /// Check whether or not Unicode text input events are enabled for a window.
    #[doc(alias = "SDL_TextInputActive")]
    pub fn is_text_input_enabled(wnd: Ref<Window>) -> bool {
        unsafe { SDL_TextInputActive(wnd.handle.as_ptr()) }
    }
}

// HACK: Probably not the correct way to get the offset.
// Works on my machine, though, so... womp womp.
const DISCRIMINANT_OFFSET: usize = align_of::<Event>();

impl From<&SDL_Event> for Event {
    fn from(value: &SDL_Event) -> Self {
        let mut ret = MaybeUninit::<Event>::uninit();

        unsafe {
            ptr::write(
                ret.as_mut_ptr().cast::<u32>(),
                *ptr::from_ref(value).cast::<u32>(),
            );

            ptr::copy_nonoverlapping(
                ptr::from_ref(value).cast::<u8>(),
                ret.as_mut_ptr().cast::<u8>().add(DISCRIMINANT_OFFSET),
                size_of::<Event>() - DISCRIMINANT_OFFSET,
            );

            ret.assume_init()
        }
    }
}

impl From<&Event> for SDL_Event {
    fn from(value: &Event) -> Self {
        let mut ret = MaybeUninit::<SDL_Event>::zeroed();
        let src = ptr::from_ref(value);

        // YOLO
        unsafe {
            ptr::write(ret.as_mut_ptr().cast::<u32>(), src.cast::<u32>().read());

            let src = src.cast::<u8>().add(DISCRIMINANT_OFFSET * 2);
            let dst = ret.as_mut_ptr().cast::<u8>().add(DISCRIMINANT_OFFSET);
            ptr::copy_nonoverlapping(src, dst, size_of::<Event>() - DISCRIMINANT_OFFSET * 2);

            ret.assume_init()
        }
    }
}

pub struct EventIter;

impl EventIter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EventIter {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for EventIter {
    type Item = Event;

    /// Incredibly cursed function that maps an [`SDL_Event`] to our own [`Event`]
    /// at minimum cost.
    ///
    /// I haven't found a true zero-cost wrapper so far; while Rust has a solution
    /// for wrapping C tagged unions via `#[repr(C, Int)]`, SDL does things in a
    /// different way -- [`SDL_Event`] isn't a (tag, union) struct, but just an union
    /// where the tag is the first field in all members. The next best solution is
    /// accepting there's gonna be a duplicate tag and performing a dual memcpy for
    /// both the enum tag and the rest of the event structure.
    #[doc(alias = "SDL_PollEvent")]
    fn next(&mut self) -> Option<Self::Item> {
        let mut current = MaybeUninit::<SDL_Event>::uninit();
        if unsafe { SDL_PollEvent(current.as_mut_ptr()) } {
            Some(unsafe { current.assume_init_ref() }.into())
        } else {
            None
        }
    }
}

impl FusedIterator for EventIter {}
