/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str;

use malloc_size_of_derive::MallocSizeOf;
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use style::Atom;
use webrender_api::NativeFontHandle;

pub mod font;

#[cfg(all(target_os = "linux", not(target_env = "ohos"), not(ohos_mock)))]
pub mod font_list;

#[cfg(target_os = "android")]
mod android {
    pub mod font_list;
    mod xml;
}
#[cfg(target_os = "android")]
pub use self::android::font_list;

#[cfg(any(target_env = "ohos", ohos_mock))]
mod ohos {
    pub mod font_list;
}
#[cfg(any(target_env = "ohos", ohos_mock))]
pub use self::ohos::font_list;

mod library_handle;

/// An identifier for a local font on systems using Freetype.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
    /// The variation index within the font.
    pub variation_index: i32,
}

impl LocalFontIdentifier {
    pub(crate) fn index(&self) -> u32 {
        self.variation_index.try_into().unwrap()
    }

    pub(crate) fn native_font_handle(&self) -> NativeFontHandle {
        NativeFontHandle {
            path: PathBuf::from(&*self.path),
            index: self.variation_index as u32,
        }
    }

    pub(crate) fn read_data_from_file(&self) -> Option<Vec<u8>> {
        let file = File::open(Path::new(&*self.path)).ok()?;
        let mmap = unsafe { Mmap::map(&file).ok()? };
        Some(mmap[..].to_vec())
    }
}
