/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use cfg_if::cfg_if;

static RES: LazyLock<RwLock<Option<Box<dyn ResourceReaderMethods + Sync + Send>>>> =
    LazyLock::new(|| {
        cfg_if! {
            if #[cfg(servo_production)] {
                RwLock::new(None)
            } else {
                // Static assert that this is really a non-production build, rather
                // than a failure of the build script’s production check.
                const _: () = assert!(cfg!(servo_do_not_use_in_production));

                RwLock::new(Some(resources_for_tests()))
            }
        }
    });

pub fn set(reader: Box<dyn ResourceReaderMethods + Sync + Send>) {
    *RES.write().unwrap() = Some(reader);
}

pub fn read_bytes(res: Resource) -> Vec<u8> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .read(res)
}

pub fn read_string(res: Resource) -> String {
    String::from_utf8(read_bytes(res)).unwrap()
}

pub fn sandbox_access_files() -> Vec<PathBuf> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .sandbox_access_files()
}

pub fn sandbox_access_files_dirs() -> Vec<PathBuf> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .sandbox_access_files_dirs()
}

pub enum Resource {
    /// A json file of [`Preferences`](servo_config::pref_util::Preferences) configuration.
    /// It can be empty but lots of features will be disabled.
    Preferences,
    /// A list of GATT services that are blocked from being used by web bluetooth.
    /// The format of the file is a list of UUIDs, one per line, with an optional second word to specify the
    /// type of blocklist.
    /// It can be empty but then all GATT services will be allowed.
    BluetoothBlocklist,
    /// A list of domain names that are considered public suffixes, typically obtained from <https://publicsuffix.org/list/>.
    /// The Public Suffix List is a cross-vendor initiative to provide an accurate list of domain name suffixes
    /// that are under the control of a registry. This is used to prevent cookies from being set for top-level
    /// domains that are not controlled by the same entity as the website.
    /// It can be empty but all domain names will be considered not public suffixes.
    DomainList,
    /// A preloaded list of HTTP Strict Transport Security. It can be an empty list and
    /// [`HstsList::default()`](net::hsts::HstsList) will be called.
    HstsPreloadList,
    /// A HTML page to display when [`SslValidation`](net_traits::NetworkError::SslValidation) network error is
    /// reported.
    /// The page contains placeholder `${reason}` for the error code and `${bytes}` for the certificate bytes,
    /// and also `${secret}` for the privileged secret.
    /// It can be empty but then nothing will be displayed when a certificate error occurs.
    BadCertHTML,
    /// A HTML page to display when any network error occurs that is not related to SSL validation.
    /// The message can contain a placeholder `${reason}` for the error code.
    /// It can be empty but then nothing will be displayed when an internal error occurs.
    NetErrorHTML,
    /// A CSS file to style the user agent stylesheet.
    /// It can be empty but then there's simply no user agent stylesheet.
    UserAgentCSS,
    /// A CSS file to style the Servo browser.
    /// It can be empty but several features might not work as expected.
    ServoCSS,
    /// A CSS file to style the presentational hints.
    /// It can be empty but then presentational hints will not be styled.
    PresentationalHintsCSS,
    /// A CSS file to style the quirks mode.
    /// It can be empty but then quirks mode will not be styled.
    QuirksModeCSS,
    /// A placeholder image to display if we couldn't get the requested image.
    ///
    /// ## Safety
    ///
    /// Servo will crash if this is an invalid image. Check `resources/rippy.png` in Servo codebase to see what
    /// a default rippy png should look like.
    RippyPNG,
    /// A CSS file to style the media controls.
    /// It can be empty but then media controls will not be styled.
    MediaControlsCSS,
    /// A JS file to control the media controls.
    /// It can be empty but then media controls will not work.
    MediaControlsJS,
    /// A placeholder HTML page to display when the code responsible for rendering a page panics and the original
    /// page can no longer be displayed.
    /// The message can contain a placeholder `${details}` for the error details.
    /// It can be empty but then nothing will be displayed when a crash occurs.
    CrashHTML,
    /// A HTML page to display when a directory listing is requested.
    /// The page contains a js function `setData` that will then be used to build the list of directory.
    /// It can be empty but then nothing will be displayed when a directory listing is requested.
    DirectoryListingHTML,
}

impl Resource {
    pub fn filename(&self) -> &'static str {
        match self {
            Resource::Preferences => "prefs.json",
            Resource::BluetoothBlocklist => "gatt_blocklist.txt",
            Resource::DomainList => "public_domains.txt",
            Resource::HstsPreloadList => "hsts_preload.json",
            Resource::BadCertHTML => "badcert.html",
            Resource::NetErrorHTML => "neterror.html",
            Resource::UserAgentCSS => "user-agent.css",
            Resource::ServoCSS => "servo.css",
            Resource::PresentationalHintsCSS => "presentational-hints.css",
            Resource::QuirksModeCSS => "quirks-mode.css",
            Resource::RippyPNG => "rippy.png",
            Resource::MediaControlsCSS => "media-controls.css",
            Resource::MediaControlsJS => "media-controls.js",
            Resource::CrashHTML => "crash.html",
            Resource::DirectoryListingHTML => "directory-listing.html",
        }
    }
}

pub trait ResourceReaderMethods {
    fn read(&self, res: Resource) -> Vec<u8>;
    fn sandbox_access_files(&self) -> Vec<PathBuf>;
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf>;
}

/// Bake all of our resources into this crate for tests, unless we are `cfg!(servo_production)`.
///
/// Local non-production embedder builds (e.g. servoshell) can still override these with [`set`],
/// if runtime loading of prefs.json and other resources is needed.
///
/// In theory this can be `#[cfg(servo_production)]`, but omitting the attribute ensures that the
/// code is always checked by the compiler, even if it later gets optimised out as dead code.
fn resources_for_tests() -> Box<dyn ResourceReaderMethods + Sync + Send> {
    struct ResourceReader;
    impl ResourceReaderMethods for ResourceReader {
        fn sandbox_access_files(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn read(&self, file: Resource) -> Vec<u8> {
            match file {
                Resource::Preferences => &include_bytes!("../../../resources/prefs.json")[..],
                Resource::BluetoothBlocklist => {
                    &include_bytes!("../../../resources/gatt_blocklist.txt")[..]
                },
                Resource::DomainList => {
                    &include_bytes!("../../../resources/public_domains.txt")[..]
                },
                Resource::HstsPreloadList => {
                    &include_bytes!("../../../resources/hsts_preload.json")[..]
                },
                Resource::BadCertHTML => &include_bytes!("../../../resources/badcert.html")[..],
                Resource::NetErrorHTML => &include_bytes!("../../../resources/neterror.html")[..],
                Resource::UserAgentCSS => &include_bytes!("../../../resources/user-agent.css")[..],
                Resource::ServoCSS => &include_bytes!("../../../resources/servo.css")[..],
                Resource::PresentationalHintsCSS => {
                    &include_bytes!("../../../resources/presentational-hints.css")[..]
                },
                Resource::QuirksModeCSS => {
                    &include_bytes!("../../../resources/quirks-mode.css")[..]
                },
                Resource::RippyPNG => &include_bytes!("../../../resources/rippy.png")[..],
                Resource::MediaControlsCSS => {
                    &include_bytes!("../../../resources/media-controls.css")[..]
                },
                Resource::MediaControlsJS => {
                    &include_bytes!("../../../resources/media-controls.js")[..]
                },
                Resource::CrashHTML => &include_bytes!("../../../resources/crash.html")[..],
                Resource::DirectoryListingHTML => {
                    &include_bytes!("../../../resources/directory-listing.html")[..]
                },
            }
            .to_owned()
        }
    }
    Box::new(ResourceReader)
}
