//! `SEVERIN_TRACE=1` bridge transcript support.
//!
//! This intentionally writes directly to stderr instead of relying on a global
//! Rust logger. A CPython extension is hosted by somebody else's process, so
//! there may be no logger installed and we must not install one on its behalf.

pub(crate) const ENV: &str = "SEVERIN_TRACE";

pub(crate) fn enabled() -> bool {
    match std::env::var(ENV) {
        Ok(value) => !matches!(value.as_str(), "" | "0" | "false" | "FALSE" | "off" | "OFF"),
        Err(_) => false,
    }
}

pub(crate) fn emit(enabled: bool, message: std::fmt::Arguments<'_>) {
    if enabled {
        eprintln!("SEVERIN_BRIDGE: {message}");
    }
}
