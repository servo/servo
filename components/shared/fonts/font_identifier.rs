/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
pub use platform::LocalFontIdentifier;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FontIdentifier {
    Local(LocalFontIdentifier),
    Web(ServoUrl),
}

impl FontIdentifier {
    pub fn index(&self) -> u32 {
        match *self {
            Self::Local(ref local_font_identifier) => local_font_identifier.index(),
            Self::Web(_) => 0,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod platform {
    use std::fs::File;
    use std::path::{Path, PathBuf};

    use malloc_size_of_derive::MallocSizeOf;
    use memmap2::Mmap;
    use serde::{Deserialize, Serialize};
    use style::Atom;
    use webrender_api::NativeFontHandle;

    use crate::{FontData, FontDataAndIndex};

    /// An identifier for a local font on systems using Freetype.
    #[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
    pub struct LocalFontIdentifier {
        /// The path to the font.
        pub path: Atom,
        /// The variation index within the font.
        pub face_index: u16,
        /// The index of the named instance within the font.
        ///
        /// For non-variable fonts, this is ignored.
        pub named_instance_index: u16,
    }

    impl LocalFontIdentifier {
        pub fn index(&self) -> u32 {
            self.face_index as u32
        }

        pub fn named_instance_index(&self) -> u32 {
            self.named_instance_index as u32
        }

        pub fn native_font_handle(&self) -> NativeFontHandle {
            NativeFontHandle {
                path: PathBuf::from(&*self.path),
                index: self.face_index as u32,
            }
        }

        #[expect(unsafe_code)]
        pub fn font_data_and_index(&self) -> Option<FontDataAndIndex> {
            let file = File::open(Path::new(&*self.path)).ok()?;
            let mmap = unsafe { Mmap::map(&file).ok()? };
            let data = FontData::from_bytes(&mmap);

            Some(FontDataAndIndex {
                data,
                index: self.face_index as u32,
            })
        }

        /// Fontconfig and FreeType use a packed format to represent face and
        /// named instance indexes in a single integer. The first 16 bits make
        /// up the named instance index and the second 16 bits make up the
        /// face index.
        ///
        /// See <https://freetype.org/freetype2/docs/reference/ft2-face_creation.html#ft_open_face>
        /// for more information.
        pub fn face_index_for_freetype(&self) -> u32 {
            ((self.named_instance_index()) << 16) | self.index()
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::fs::File;
    use std::path::Path;

    use log::warn;
    use malloc_size_of_derive::MallocSizeOf;
    use memmap2::Mmap;
    use read_fonts::types::NameId;
    use read_fonts::{FileRef, TableProvider};
    use serde::{Deserialize, Serialize};
    use style::Atom;
    use webrender_api::NativeFontHandle;

    use crate::{FontData, FontDataAndIndex};

    /// An identifier for a local font on a MacOS system. These values comes from the CoreText
    /// CTFontCollection. Note that `path` here is required. We do not load fonts that do not
    /// have paths.
    #[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
    pub struct LocalFontIdentifier {
        pub postscript_name: Atom,
        pub path: Atom,
    }

    impl LocalFontIdentifier {
        pub fn native_font_handle(&self) -> NativeFontHandle {
            NativeFontHandle {
                name: self.postscript_name.to_string(),
                path: self.path.to_string(),
            }
        }

        pub(crate) fn index(&self) -> u32 {
            0
        }

        #[expect(unsafe_code)]
        pub fn font_data_and_index(&self) -> Option<FontDataAndIndex> {
            let file = File::open(Path::new(&*self.path)).ok()?;
            let mmap = unsafe { Mmap::map(&file).ok()? };

            // Determine index
            let file_ref = FileRef::new(mmap.as_ref()).ok()?;
            let index = ttc_index_from_postscript_name(file_ref, &self.postscript_name);

            Some(FontDataAndIndex {
                data: FontData::from_bytes(&mmap),
                index,
            })
        }
    }

    /// CoreText font enumeration gives us a Postscript name rather than an index.
    /// This functions maps from a Postscript name to an index.
    ///
    /// This mapping works for single-font files and for simple TTC files, but may not work in all cases.
    /// We are not 100% sure which cases (if any) will not work. But we suspect that variable fonts may cause
    /// issues due to the Postscript names corresponding to instances not being straightforward, and the possibility
    /// that CoreText may return a non-standard in that scenerio.
    fn ttc_index_from_postscript_name(font_file: FileRef<'_>, postscript_name: &str) -> u32 {
        match font_file {
            // File only contains one font: simply return 0
            FileRef::Font(_) => 0,
            // File is a collection: iterate through each font in the collection and check
            // whether the name matches
            FileRef::Collection(collection) => {
                for i in 0..collection.len() {
                    let font = collection.get(i).unwrap();
                    let name_table = font.name().unwrap();
                    if name_table
                        .name_record()
                        .iter()
                        .filter(|record| record.name_id() == NameId::POSTSCRIPT_NAME)
                        .any(|record| {
                            record
                                .string(name_table.string_data())
                                .unwrap()
                                .chars()
                                .eq(postscript_name.chars())
                        })
                    {
                        return i;
                    }
                }

                // If we fail to find a font, just use the first font in the file.
                warn!(
                    "Font with postscript_name {} not found in collection",
                    postscript_name
                );
                0
            },
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::hash::Hash;
    use std::sync::Arc;

    use dwrote::{FontCollection, FontDescriptor};
    use malloc_size_of_derive::MallocSizeOf;
    use serde::{Deserialize, Serialize};
    use webrender_api::NativeFontHandle;

    use crate::{FontData, FontDataAndIndex};

    /// An identifier for a local font on a Windows system.
    #[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
    pub struct LocalFontIdentifier {
        /// The FontDescriptor of this font.
        #[ignore_malloc_size_of = "dwrote does not support MallocSizeOf"]
        pub font_descriptor: Arc<FontDescriptor>,
    }

    impl LocalFontIdentifier {
        pub fn index(&self) -> u32 {
            FontCollection::system()
                .font_from_descriptor(&self.font_descriptor)
                .ok()
                .flatten()
                .map_or(0, |font| font.create_font_face().get_index())
        }

        pub fn native_font_handle(&self) -> NativeFontHandle {
            let face = FontCollection::system()
                .font_from_descriptor(&self.font_descriptor)
                .ok()
                .flatten()
                .expect("Could not create Font from FontDescriptor")
                .create_font_face();
            let path = face
                .files()
                .ok()
                .and_then(|files| files.first().cloned())
                .expect("Could not get FontFace files")
                .font_file_path()
                .ok()
                .expect("Could not get FontFace files path");
            NativeFontHandle {
                path,
                index: face.get_index(),
            }
        }

        pub fn font_data_and_index(&self) -> Option<FontDataAndIndex> {
            let font = FontCollection::system()
                .font_from_descriptor(&self.font_descriptor)
                .ok()??;
            let face = font.create_font_face();
            let index = face.get_index();
            let files = face.files().ok()?;
            assert!(!files.is_empty());

            let data = files[0].font_file_bytes().ok()?;
            let data = FontData::from_bytes(&data);

            Some(FontDataAndIndex { data, index })
        }
    }

    impl Eq for LocalFontIdentifier {}

    impl Hash for LocalFontIdentifier {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.font_descriptor.family_name.hash(state);
            self.font_descriptor.weight.to_u32().hash(state);
            self.font_descriptor.stretch.to_u32().hash(state);
            self.font_descriptor.style.to_u32().hash(state);
        }
    }
}
