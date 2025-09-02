fn main() {
    // Only compile Slint UI when the `gui` feature is enabled.
    if std::env::var("CARGO_FEATURE_GUI").is_ok() {
        slint_build::compile("ui/appwindow.slint").unwrap();
    }
}
