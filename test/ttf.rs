use rustest::test;
use sandlot::ttf::RtStr;

#[test]
fn ttf_rt_str_new() {
    // Case 1: Non-empty string.
    let s = "Hello!";
    let rt = RtStr::new(s);
    assert_eq!(rt.as_ptr(), s.as_ptr().cast());
    assert_eq!(rt.len(), s.len());

    // Case 2: Empty string, converted to empty `&CStr`.
    let s = "";
    let rt = RtStr::new(s);
    assert_ne!(s.as_ptr().cast(), rt.as_ptr());
    assert_eq!(rt.len(), 0);
}

#[test]
fn ttf_rt_str_new_unchecked() {
    let s = "Hello!";
    let rt = unsafe { RtStr::new_unchecked(s) };
    assert_eq!(rt.as_ptr(), s.as_ptr().cast());
    assert_eq!(rt.len(), s.len());
}
