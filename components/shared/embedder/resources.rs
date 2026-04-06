/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::LazyLock;

#[doc(hidden)]
pub use inventory as _inventory;

/// A static reference to a ResourceReader
///
/// If you need to initialize the resource reader at runtime, use interior mutability.
///
/// # Examples
///
/// ```
/// pub(crate) struct ResourceReaderImpl {
///     resource_dir: OnceLock<PathBuf>,
/// }
/// static RESOURCE_READER: ResourceReaderImpl = ResourceReaderImpl {
///     resource_dir: OnceLock::new(),
/// };
///
/// servo::submit_resource_reader!(&RESOURCE_READER);
///
/// /// This can be called during initialization, e.g. after parsing commandline flags.
/// pub(crate) fn set_resource_dir(resource_dir: PathBuf) {
///     RESOURCE_READER.resource_dir.set(resource_dir).expect("Already initialized.")
/// }
/// impl ResourceReaderMethods for ResourceReaderImpl {
///  //
/// }
/// ```
pub type ResourceReader = &'static (dyn ResourceReaderMethods + Sync + Send);

/// Register the [`ResourceReader`] implementation.
///
/// This should be added at most once in the whole project.
/// In particular this means you should make sure to disable the (default)
/// `baked-in-resources` feature of servo if you want to override the default reader.
///
/// # Examples
///
/// Put `submit_resource_reader` invocations **outside** any function body:
/// ```
/// servo_embedder_traits::submit_resource_reader!(my_resource_reader);
/// ```
#[macro_export]
macro_rules! submit_resource_reader {
    ($resource_reader:expr) => {
        $crate::resources::_inventory::submit! {
            $resource_reader as $crate::resources::ResourceReader
        }
    };
}

// The embedder may register a resource reader via `submit_resource_reader!()`
// Note: A weak symbol would perhaps be preferable, but that isn't available in stable rust yet.
inventory::collect!(ResourceReader);

static RESOURCE_READER: LazyLock<ResourceReader> = {
    LazyLock::new(|| {
        let mut resource_reader_iterator = inventory::iter::<ResourceReader>.into_iter();
        let Some(resource_reader) = resource_reader_iterator.next() else {
            panic!("No resource reader registered");
        };
        if resource_reader_iterator.next().is_some() {
            log::error!(
                "Multiple resource readers registered. Taking the first implementation \
                (random, non deterministic order). This is a bug! Check usages of \
                `submit_resource_reader!()`. Perhaps you meant to disable the default resource reader \
                (selected by depending on the `servo-default-resources` crate) ?"
            );
        }
        *resource_reader
    })
};

pub fn read_bytes(res: Resource) -> Vec<u8> {
    RESOURCE_READER.read(res)
}

pub fn read_string(res: Resource) -> String {
    String::from_utf8(read_bytes(res)).unwrap()
}

pub fn sandbox_access_files() -> Vec<PathBuf> {
    RESOURCE_READER.sandbox_access_files()
}

pub fn sandbox_access_files_dirs() -> Vec<PathBuf> {
    RESOURCE_READER.sandbox_access_files_dirs()
}

pub enum Resource {
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
    /// `HstsList::default()` will be called.
    HstsPreloadList,
    /// A HTML page to display when `net_traits::NetworkError::SslValidation` network error is
    /// reported.
    /// The page contains placeholder `${reason}` for the error code and `${bytes}` for the certificate bytes,
    /// and also `${secret}` for the privileged secret.
    /// It can be empty but then nothing will be displayed when a certificate error occurs.
    BadCertHTML,
    /// A HTML page to display when any network error occurs that is not related to SSL validation.
    /// The message can contain a placeholder `${reason}` for the error code.
    /// It can be empty but then nothing will be displayed when an internal error occurs.
    NetErrorHTML,
    /// A placeholder image to display if we couldn't get the requested image.
    ///
    /// ## Panic
    ///
    /// If the resource is not provided, servo will fallback to a baked in default (See resources/rippy.png).
    /// However, if the image is provided but invalid, Servo will crash.
    BrokenImageIcon,
    /// A placeholder HTML page to display when the code responsible for rendering a page panics and the original
    /// page can no longer be displayed.
    /// The message can contain a placeholder `${details}` for the error details.
    /// It can be empty but then nothing will be displayed when a crash occurs.
    CrashHTML,
    /// A HTML page to display when a directory listing is requested.
    /// The page contains a js function `setData` that will then be used to build the list of directory.
    /// It can be empty but then nothing will be displayed when a directory listing is requested.
    DirectoryListingHTML,
    /// A HTML page that is used for the about:memory url.
    AboutMemoryHTML,
    /// RPC script for the Debugger API on behalf of devtools.
    DebuggerJS,
    /// A HTML page to display a pretty printed view of a json document.
    JsonViewerHTML,
}

impl Resource {
    pub fn filename(&self) -> &'static str {
        match self {
            Resource::BluetoothBlocklist => "gatt_blocklist.txt",
            Resource::DomainList => "public_domains.txt",
            Resource::HstsPreloadList => "hsts_preload.fstmap",
            Resource::BadCertHTML => "badcert.html",
            Resource::NetErrorHTML => "neterror.html",
            Resource::BrokenImageIcon => "rippy.png",
            Resource::CrashHTML => "crash.html",
            Resource::DirectoryListingHTML => "directory-listing.html",
            Resource::AboutMemoryHTML => "about-memory.html",
            Resource::DebuggerJS => "debugger.js",
            Resource::JsonViewerHTML => "json-viewer.html",
        }
    }
}

pub trait ResourceReaderMethods {
    /// Read a named [`Resource`].
    ///
    /// The implementation must be functional in all Servo processes.
    fn read(&self, res: Resource) -> Vec<u8>;
    /// Files that should remain accessible after sandboxing the content process.
    ///
    /// If the resources are shipped as files, then the files should be listed here,
    /// or the parent directory in [sandbox_access_files_dirs].
    fn sandbox_access_files(&self) -> Vec<PathBuf>;
    /// Directories that should remain accessible after sandboxing the content process.
    ///
    /// If resources are shipped as files, then the directory containing them be listed
    /// here to ensure the content process can access the files.
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf>;
}
