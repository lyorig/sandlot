use rustest::{Result, test};

#[test]
fn string_into_boxed_str() -> Result {
    let pp = sandlot::fs::pref_path(c"Foo", c"Bar")?;
    let s = pp.into_boxed_str();
    let s_copy = s.to_owned();

    assert_eq!(s.as_ref(), &s_copy);

    Ok(())
}
