#[cfg(any(
    all(
        target_os = "android",
        any(target_arch = "arm", target_arch = "aarch64")
    ),
    target_arch = "x86_64",
    target_arch = "aarch64",
))]
mod platform {
    pub use servo_media_gstreamer::GStreamerBackend as Backend;
}

#[cfg(not(any(
    all(
        target_os = "android",
        any(target_arch = "arm", target_arch = "aarch64")
    ),
    target_arch = "x86_64",
    target_arch = "aarch64",
)))]
mod platform {
    pub use servo_media_dummy::DummyBackend as Backend;
}

pub type Backend = platform::Backend;
