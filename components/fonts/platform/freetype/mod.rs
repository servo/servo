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
    mod iso_values_converter;
    mod json;
}
#[cfg(any(target_env = "ohos", ohos_mock))]
pub use self::ohos::font_list;

mod freetype_errors;
mod freetype_truetype_unicode_ranges;
mod library_handle;

/// An identifier for a local font on systems using Freetype.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
    /// This field holds two different values. Bits 0-15 are the index of the face in the font file (starting with
    /// value 0). Set it to 0 if there is only one face in the font file.
    /// [Since 2.6.1] Bits 16-30 are relevant to GX and OpenType variation fonts only, specifying the named instance
    /// index for the current face index (starting with value 1; value 0 makes FreeType ignore named instances).
    /// For non-variation fonts, bits 16-30 are ignored. Assuming that you want to access the third named instance in
    /// face 4, face_index should be set to 0x00030004. If you want to access face 4 without variation handling, simply
    /// set face_index to value 4.
    /// FT_Open_Face and its siblings can be used to quickly check whether the font format of a given font resource is
    /// supported by FreeType. In general, if the face_index argument is negative, the function's return value is 0 if
    /// the font format is recognized, or non-zero otherwise. The function allocates a more or less empty face handle
    /// in *aface (if aface isn't NULL); the only two useful fields in this special case are face->num_faces and
    /// face->style_flags. For any negative value of face_index, face->num_faces gives the number of faces within
    /// the font file. For the negative value ‘-(N+1)’ (with ‘N’ a non-negative 16-bit value), bits 16-30 in
    /// face->style_flags give the number of named instances in face ‘N’ if we have a variation font (or zero
    /// otherwise). After examination, the returned FT_Face structure should be deallocated with a call to FT_Done_Face.
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
