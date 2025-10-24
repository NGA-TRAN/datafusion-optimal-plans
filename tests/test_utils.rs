/// Helper function to configure insta with path filters for snapshot tests
///
/// This function normalizes file paths in test snapshots to make them portable
/// across different development environments.
pub fn insta_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    // Filter out the absolute workspace path to make snapshots portable
    let cwd = std::env::current_dir().unwrap();
    let cwd_str = cwd.to_str().unwrap();
    // DataFusion outputs paths without leading slash on macOS, so we strip the leading / from cwd
    let cwd_no_slash = cwd_str.trim_start_matches('/');
    // Replace absolute paths with root-relative paths for portability
    settings.add_filter(&format!("{}/", cwd_no_slash), "/");
    settings
}
