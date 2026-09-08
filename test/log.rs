use rustest::test;
use sandlot::log::Category;

// Tests that all log priority macro overloads work.
// Only tests `log_trace!` since every other priority macro works exactly the same.
// This also avoids extraneous logs in tests.
#[test]
fn log_macro_overloads() {
    sandlot::log_trace!("without any arguments");
    sandlot::log_trace!("with an argument {}", 1);
    sandlot::log_trace!(Category::Audio, "with a category");
    sandlot::log_trace!(Category::Audio, "with a category and an argument {}", 2);
}
