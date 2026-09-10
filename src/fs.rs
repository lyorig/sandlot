//! Examination and manipulation of the filesystem. Functionally overlaps with [`std::fs`].
//!
//! This covers most things one would need to do with directories, except for
//! actual file I/O (which is covered by [CategoryIOStream](https://wiki.libsdl.org/SDL3/CategoryIOStream)
//! and [CategoryAsyncIO](https://wiki.libsdl.org/SDL3/CategoryAsyncIO) instead).
//!
//! There are functions to answer necessary path questions:
//!
//! - Where is my app's data? [`Context::base_path`](crate::Context::base_path).
//! - Where can I safely write files? [`fs::pref_path`](pref_path).
//! - Where are paths like Downloads, Desktop, Music? [`Context::user_folder`](crate::Context::user_folder).
//! - What is this thing at this location? [`fs::path_info`](path_info).
//! - What items live in this folder? [`fs::enumerate_directory`](enumerate_directory).
//! - What items live in this folder by wildcard? [`fs::glob_directory`](glob_directory).
//! - What is my current working directory? [`fs::current_directory`](current_directory).
//!
//! SDL also offers functions to manipulate the directory tree: renaming,
//! removing, copying files.
//!
//! Implementation checklist ([source](https://wiki.libsdl.org/SDL3/CategoryFilesystem)):
//! - [x] SDL_CopyFile
//! - [x] SDL_CreateDirectory
//! - [x] SDL_EnumerateDirectory
//! - [x] SDL_GetBasePath (impl'd as [`Context::base_path`](crate::Context::base_path))
//! - [x] SDL_GetCurrentDirectory
//! - [x] SDL_GetPathInfo
//! - [x] SDL_GetPrefPath
//! - [x] SDL_GetUserFolder (impl'd as [`Context::user_folder`](crate::Context::user_folder))
//! - [x] SDL_GlobDirectory
//! - [x] SDL_RemovePath
//! - [x] SDL_RenamePath

use std::{
    ffi::{CStr, c_char},
    mem::MaybeUninit,
    ptr::{NonNull, from_mut},
};

use bitflags::bitflags;
use sdl3_sys::filesystem::*;

use crate::{Result, boxed::Box, impl_enum_transmute, string::String, util::to_result};

/// The type of the OS-provided default folder for a specific purpose.
///
/// The Trash folder is not included because moving files to the trash usually
/// requires extra OS-specific functionality to remember their original locations.
///
/// The folders supported per platform are:
///
/// | Folder | Windows | macOS/iOS | tvOS | Unix (XDG) | Haiku | Emscripten |
/// | ------ | ------- | --------- | ---- | ---------- | ----- | ---------- |
/// | [`Self::Home`] | yes | yes | | yes | yes | yes |
/// | [`Self::Desktop`] | yes | yes | | yes | yes | |
/// | [`Self::Documents`] | yes | yes | | yes | | |
/// | [`Self::Downloads`] | Vista and later | yes | | yes | | |
/// | [`Self::Music`] | yes | yes | | yes | | |
/// | [`Self::Pictures`] | yes | yes | | yes | | |
/// | [`Self::PublicShare`] | | yes | | yes | | |
/// | [`Self::SavedGames`] | Vista and later | | | | | |
/// | [`Self::Screenshots`] | Vista and later | | | | | |
/// | [`Self::Templates`] | yes | | | yes | | |
/// | [`Self::Videos`] | yes | yes | | yes | | |
///
/// On macOS and iOS, [`Self::Videos`] refers to the “Movies” folder.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum Folder {
    /// The folder containing the current user's data, preferences, and documents.
    /// It is a safe fallback for storing user documents when another requested
    /// folder does not exist.
    Home = SDL_Folder::HOME.0,
    /// The folder containing files displayed on the desktop.
    Desktop = SDL_Folder::DESKTOP.0,
    /// The folder for user documents, possibly application-specific.
    Documents = SDL_Folder::DOCUMENTS.0,
    /// The standard folder for files downloaded from the internet.
    Downloads = SDL_Folder::DOWNLOADS.0,
    /// The folder for music files playable by a standard music player.
    Music = SDL_Folder::MUSIC.0,
    /// The folder for image files displayable by a standard image viewer.
    Pictures = SDL_Folder::PICTURES.0,
    /// The folder for files intended to be shared with other users of the computer.
    PublicShare = SDL_Folder::PUBLICSHARE.0,
    /// The folder for game save files.
    SavedGames = SDL_Folder::SAVEDGAMES.0,
    /// The folder for application screenshots.
    Screenshots = SDL_Folder::SCREENSHOTS.0,
    /// The folder for desktop-environment file templates.
    Templates = SDL_Folder::TEMPLATES.0,
    /// The folder for video files playable by a standard video player.
    Videos = SDL_Folder::VIDEOS.0,
}

impl_enum_transmute!(SDL_Folder, Folder);

/// The type of a filesystem entry.
///
/// Filesystem entries such as devices and named pipes are reported as
/// [`Self::Other`]. Symlinks are followed when determining the type.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PathType {
    /// The path does not exist.
    #[default]
    None = SDL_PathType::NONE.0,
    /// The path is a normal file.
    File = SDL_PathType::FILE.0,
    /// The path is a directory.
    Directory = SDL_PathType::DIRECTORY.0,
    /// The path is a filesystem entry of another kind, such as a device node.
    Other = SDL_PathType::OTHER.0,
}

impl_enum_transmute!(SDL_PathType, PathType);

/// Information about a path on the filesystem.
///
/// This mirrors `SDL_PathInfo`. The C field `type` is named [`Self::path_type`]
/// to avoid clashing with the Rust keyword.
#[doc(alias = "SDL_PathInfo")]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathInfo {
    /// The type of the path.
    pub path_type: PathType,
    /// The file size in bytes.
    pub size: u64,
    /// The time when the path was created.
    pub create_time: i64,
    /// The last time the path was modified.
    pub modify_time: i64,
    /// The last time the path was read.
    pub access_time: i64,
}

bitflags! {
    /// Flags for path matching.
    #[derive(Clone, Copy, Debug)]
    #[doc(alias = "SDL_GlobFlags")]
    pub struct GlobFlags: u32 {
        /// Match the pattern without considering letter case.
        const CASE_INSENSITIVE = SDL_GlobFlags::CASEINSENSITIVE.0;
    }
}

/// Possible results from a directory-enumeration callback.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum EnumerationResult {
    /// Continue enumerating directory entries.
    Continue = SDL_EnumerationResult::CONTINUE.0,
    /// Stop enumerating successfully.
    Success = SDL_EnumerationResult::SUCCESS.0,
    /// Stop enumerating and report failure.
    Failure = SDL_EnumerationResult::FAILURE.0,
}

impl_enum_transmute!(SDL_EnumerationResult, EnumerationResult);

/// Get the user- and application-specific directory where files can be written.
///
/// The directory is specific to the current user and application. It is created
/// if necessary, and the returned path is absolute, UTF-8 encoded, and guaranteed
/// to end with a path separator (`\\` on Windows and `/` on most other platforms).
///
/// * `org` is the organization name. Use the same, case-sensitive value for all
///   applications that use this function.
/// * `app` is the application name. Use a unique, stable value for each application.
///
/// Both names may become part of a directory name. Unicode is allowed when encoded
/// as UTF-8, but letters, numbers, and spaces are recommended instead of punctuation.
/// An empty `org` omits the organization subdirectory; new applications should
/// provide a real organization name.
///
/// Returns [`Err`] if the directory cannot be created or another problem occurs.
///
/// This is the only safe location returned by SDL for application-managed files;
/// [`crate::Context::base_path`] may not be writable.
#[doc(alias = "SDL_GetPrefPath")]
pub fn pref_path(org: &CStr, app: &CStr) -> Result<String> {
    unsafe {
        let ptr = SDL_GetPrefPath(org.as_ptr(), app.as_ptr());
        String::from_raw_nullck(ptr)
    }
}

/// Create a directory and any missing parent directories.
///
/// `path` is the path of the directory to create. The operation succeeds if
/// `path` already exists as a directory. If creation fails, parent directories
/// already created by this call are not removed.
///
/// Returns [`Err`] on failure; the current SDL error describes the failure.
#[doc(alias = "SDL_CreateDirectory")]
pub fn create_directory(path: &CStr) -> Result<()> {
    to_result(unsafe { SDL_CreateDirectory(path.as_ptr()) })
}

/// Remove a file or an empty directory.
///
/// `path` is the path to remove. Non-empty directories cannot be removed;
/// this function does not recursively delete directory trees.
///
/// Returns [`Err`] on failure; the current SDL error describes the failure.
#[doc(alias = "SDL_RemovePath")]
pub fn remove_path(path: &CStr) -> Result<()> {
    to_result(unsafe { SDL_RemovePath(path.as_ptr()) })
}

/// Rename a file or directory.
///
/// * `old_path` is the current path.
/// * `new_path` is the replacement path. An existing file at this path is replaced.
///
/// Renaming does not copy files across filesystems, drives, or volumes. Use a
/// copy to a temporary file on the destination filesystem followed by a rename
/// when that behavior is required.
///
/// Returns [`Err`] on failure; the current SDL error describes the failure.
#[doc(alias = "SDL_RenamePath")]
pub fn rename_path(old_path: &CStr, new_path: &CStr) -> Result<()> {
    to_result(unsafe { SDL_RenamePath(old_path.as_ptr(), new_path.as_ptr()) })
}

/// Copy a file.
///
/// * `old_path` is the source path.
/// * `new_path` is the destination path. An existing file at this path is
///   overwritten.
///
/// This operation blocks until the copy is complete and is not atomic. Readers
/// can observe an incomplete destination, and a failed copy leaves the destination
/// in an undefined state. To avoid this, copy to a temporary file in the same
/// directory and rename it into place after a successful copy.
///
/// SDL attempts to synchronize the copied data to disk before returning when the
/// platform allows it.
///
/// Returns [`Err`] on failure; the current SDL error describes the failure.
#[doc(alias = "SDL_CopyFile")]
pub fn copy_file(old_path: &CStr, new_path: &CStr) -> Result<()> {
    to_result(unsafe { SDL_CopyFile(old_path.as_ptr(), new_path.as_ptr()) })
}

/// Get information about a filesystem path.
///
/// `path` is the path to query. Symlinks are followed, so the returned
/// information describes the target rather than the symlink itself.
///
/// Returns [`Err`] if the path does not exist or another failure occurs;
/// the current SDL error describes the failure.
#[doc(alias = "SDL_GetPathInfo")]
pub fn path_info(path: &CStr) -> Result<PathInfo> {
    let mut info = MaybeUninit::<PathInfo>::uninit();

    to_result(unsafe { SDL_GetPathInfo(path.as_ptr(), info.as_mut_ptr().cast()) })?;

    // SAFETY: SDL fully initializes the struct on success.
    // The layouts of both types match field for field.
    Ok(unsafe { info.assume_init() })
}

/// Enumerate a directory through a callback function.
///
/// `path` is the path of the directory to enumerate. `callback` is called once
/// for each entry until all entries are provided or it returns
/// [`EnumerationResult::Success`] or [`EnumerationResult::Failure`].
///
/// The callback receives the directory currently being enumerated and the name
/// of the current entry without a path prefix. Both references are valid only
/// for the duration of the callback.
///
/// Return [`EnumerationResult::Continue`] to keep enumerating,
/// [`EnumerationResult::Success`] to stop early while succeeding, or
/// [`EnumerationResult::Failure`] to stop and fail.
///
/// Returns [`Err`] if there is a system error or the callback returns
/// [`EnumerationResult::Failure`].
#[doc(alias = "SDL_EnumerateDirectory")]
pub fn enumerate_directory<F>(path: &CStr, mut callback: F) -> Result<()>
where
    F: FnMut(&CStr, &CStr) -> EnumerationResult,
{
    unsafe extern "C" fn trampoline<F>(
        userdata: *mut std::ffi::c_void,
        dirname: *const c_char,
        fname: *const c_char,
    ) -> SDL_EnumerationResult
    where
        F: FnMut(&CStr, &CStr) -> EnumerationResult,
    {
        // SAFETY: SDL passes valid, nul-terminated strings,
        // valid for the duration of the call.
        unsafe {
            let callback = (userdata as *mut F).as_mut_unchecked();
            callback(CStr::from_ptr(dirname), CStr::from_ptr(fname)).into()
        }
    }

    to_result(unsafe {
        SDL_EnumerateDirectory(
            path.as_ptr(),
            Some(trampoline::<F>),
            from_mut(&mut callback).cast(),
        )
    })
}

/// Enumerate a directory tree, filter entries by pattern, and return a list.
///
/// * `path` is the path of the directory to enumerate.
/// * `pattern` filters entries by the wildcards `*` (any sequence) and `?` (one
///   character). `None` returns all entries.
/// * `flags` controls pattern matching behavior.
///
/// Subdirectories are permitted in patterns and are separated with `/`.
/// Wildcards never match a path separator. [`GlobFlags::CASE_INSENSITIVE`]
/// enables case-insensitive matching.
///
/// Entries are returned as an SDL-allocated array of pointers. The array and
/// its strings are one allocation; the individual strings must not be freed.
/// The array is freed automatically when the returned [`Box`] is dropped.
///
/// Returns [`Err`] on failure; the current SDL error describes the failure.
#[doc(alias = "SDL_GlobDirectory")]
pub fn glob_directory(
    path: &CStr,
    pattern: Option<&CStr>,
    flags: GlobFlags,
) -> Result<Box<[NonNull<c_char>]>> {
    let pattern = pattern.map_or(std::ptr::null(), CStr::as_ptr);

    // Initialize to zero, so no uninitialized memory is read in case of failure.
    let mut count: i32 = 0;
    let ptr = unsafe {
        SDL_GlobDirectory(
            path.as_ptr(),
            pattern,
            SDL_GlobFlags::new(flags.bits()),
            &mut count,
        )
    };

    // SAFETY: On success, SDL allocates a single array of `count` string pointers.
    unsafe { Box::from_raw_parts_nullck(ptr.cast(), count as usize) }
}

/// Get the system's current working directory.
///
/// The returned path is UTF-8 encoded, uses platform-dependent notation, and
/// is guaranteed to end with a path separator (`\\` on Windows and `/` on most
/// other platforms). On systems without a current-working-directory concept,
/// SDL still attempts to return a reasonable path.
///
/// SDL does not provide a way to change the current working directory through
/// this API.
///
/// Returns [`Err`] if the current directory cannot be obtained.
#[doc(alias = "SDL_GetCurrentDirectory")]
pub fn current_directory() -> Result<String> {
    unsafe {
        let ptr = SDL_GetCurrentDirectory();
        String::from_raw_nullck(ptr)
    }
}
