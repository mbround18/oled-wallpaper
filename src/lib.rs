pub mod config;
pub mod configurator;
/// Interactive OLED Wallpaper Application
///
/// Core library module setup with logging/tracing support.
/// Application entry point and foundational infrastructure.
pub mod error;
pub mod input;
pub mod math;
pub mod perf;
pub mod physics;
pub mod renderer;
pub mod runtime;
pub mod wallpaper;
pub mod weather;
pub mod widgets;

use tracing_subscriber::filter::EnvFilter;

/// Initialize the tracing/logging system with RUST_LOG environment variable support
pub fn init_tracing() {
    let env_filter = EnvFilter::from_default_env().add_directive(
        "oled_wallpaper=debug"
            .parse()
            .expect("Valid tracing directive"),
    );

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_succeeds() {
        // init_tracing should not panic
        // Note: Only call once in test suite due to static initialization
        init_tracing();
    }
}
