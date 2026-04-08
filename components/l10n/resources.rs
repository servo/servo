/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use parking_lot::RwLock;

pub trait LocaleReaderMethods {
    fn load_bundle(&self, bundle: &str, locale: &str) -> Option<String>;
}

pub type LocaleReader = Box<dyn LocaleReaderMethods + Sync + Send>;

static LOCALE_READER: OnceLock<RwLock<Option<LocaleReader>>> = OnceLock::new();

fn get_or_init() -> &'static RwLock<Option<LocaleReader>> {
    LOCALE_READER.get_or_init(|| RwLock::new(None))
}

pub fn set_locale_reader(reader: LocaleReader) {
    let lock = get_or_init();
    let mut guard = lock.write();
    *guard = Some(reader);
}

pub fn load_bundle(bundle: &str, locale: &str) -> Option<String> {
    let guard = get_or_init().read();
    let reader = (*guard).as_ref()?;
    reader.load_bundle(bundle, locale)
}

/// A locale reader that will read locales from the file system.
/// The expected layout is <root>/<bundle>/<locale>.ftl
pub struct FileLocaleReader {
    root: PathBuf,
}

impl FileLocaleReader {
    pub fn new(root: &Path) -> Self {
        Self { root: root.into() }
    }
}

impl LocaleReaderMethods for FileLocaleReader {
    fn load_bundle(&self, bundle: &str, locale: &str) -> Option<String> {
        let path = self.root.join(locale).join(format!("{bundle}.ftl"));
        let bytes = std::fs::read(path).ok()?;
        String::from_utf8(bytes).ok()
    }
}
