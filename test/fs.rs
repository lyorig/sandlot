use std::{
    ffi::{CStr, CString},
    fs as std_fs,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

use sandlot::{
    fs,
    fs::{EnumerationResult, Folder, GlobFlags, PathType},
    init::Context,
};

use rustest::test;

/// Directory names must be unique per test run,
/// since rustest may run tests in parallel.
fn scratch_dir(name: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("sandlot-fs-test-{name}-{id}"));

    // Reruns of a failing test can leave leftovers behind.
    let _ = std_fs::remove_dir_all(&dir);

    dir
}

fn cstr(path: &std::path::Path) -> CString {
    CString::new(path.to_str().unwrap()).unwrap()
}

#[test]
fn fs_base_path_ends_with_separator() {
    let ctx = Context::new();

    let path = ctx.base_path().unwrap();
    assert!(path.ends_with('/') || path.ends_with('\\'));
}

#[test]
fn fs_pref_path_ends_with_separator() -> rustest::Result {
    let path = fs::pref_path(c"sandlot", c"fs-test")?;
    let path = path.to_str();

    assert!(path.ends_with('/') || path.ends_with('\\'));

    Ok(())
}

#[test]
fn fs_user_folder_home() -> rustest::Result {
    let ctx = Context::new();

    let home = ctx.user_folder(Folder::Home)?;
    assert!(home.ends_with('/') || home.ends_with('\\'));

    Ok(())
}

#[test]
fn fs_directory_roundtrip() -> rustest::Result {
    let dir = scratch_dir("roundtrip");
    let nested = dir.join("nested");

    // Missing parents are created recursively.
    fs::create_directory(&cstr(&nested))?;

    let file = nested.join("file.txt");
    std_fs::write(&file, b"contents")?;

    // Enumeration sees the nested directory and its file,
    // with entry names relative to the enumerated directory.
    let mut names = Vec::new();
    fs::enumerate_directory(&cstr(&dir), |_, fname| {
        names.push(CString::from(fname));
        EnumerationResult::Continue
    })?;

    assert!(names.iter().any(|name| name.as_c_str() == c"nested"));

    let mut names = Vec::new();
    fs::enumerate_directory(&cstr(&nested), |_, fname| {
        names.push(CString::from(fname));
        EnumerationResult::Continue
    })?;

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].as_c_str(), c"file.txt");

    // Globbing by wildcard.
    let results = fs::glob_directory(&cstr(&nested), Some(c"*.txt"), GlobFlags::empty())?;
    assert_eq!(results.len(), 1);
    // SAFETY: The array holds `len` valid, nul-terminated strings.
    let entry = unsafe { CStr::from_ptr(results[0].as_ptr()) };
    assert_eq!(entry, c"file.txt");
    drop(results);

    // Path info reflects the file.
    let info = fs::path_info(&cstr(&file))?;
    assert_eq!(info.path_type, PathType::File);
    assert_eq!(info.size, "contents".len() as u64);

    // Renaming and removal.
    let renamed = nested.join("renamed.txt");
    fs::rename_path(&cstr(&file), &cstr(&renamed))?;
    // A nonexistent path is an error, not `PathType::None`.
    assert!(fs::path_info(&cstr(&file)).is_err());

    fs::remove_path(&cstr(&renamed))?;
    fs::remove_path(&cstr(&nested))?;
    fs::remove_path(&cstr(&dir))?;
    // A nonexistent path is an error, not `PathType::None`.
    assert!(fs::path_info(&cstr(&dir)).is_err());

    Ok(())
}

#[test]
fn fs_enumeration_stops_early() -> rustest::Result {
    let dir = scratch_dir("stop-early");
    fs::create_directory(&cstr(&dir))?;

    for i in 0..4 {
        let file = dir.join(format!("{i}.txt"));
        std_fs::write(&file, b"")?;
    }

    let mut names = Vec::new();
    fs::enumerate_directory(&cstr(&dir), |_, fname| {
        names.push(CString::from(fname));
        EnumerationResult::Success
    })?;

    assert_eq!(names.len(), 1);

    for i in 0..4 {
        fs::remove_path(&cstr(&dir.join(format!("{i}.txt"))))?;
    }
    fs::remove_path(&cstr(&dir))?;

    Ok(())
}

#[test]
fn fs_copy_file() -> rustest::Result {
    let dir = scratch_dir("copy");
    fs::create_directory(&cstr(&dir))?;

    let src = dir.join("src.txt");
    std_fs::write(&src, b"payload")?;

    let dst = dir.join("dst.txt");
    fs::copy_file(&cstr(&src), &cstr(&dst))?;

    let info = fs::path_info(&cstr(&dst))?;
    assert_eq!(info.path_type, PathType::File);
    assert_eq!(info.size, "payload".len() as u64);

    fs::remove_path(&cstr(&dst))?;
    fs::remove_path(&cstr(&src))?;
    fs::remove_path(&cstr(&dir))?;

    Ok(())
}
