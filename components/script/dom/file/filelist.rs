/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::slice::Iter;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::id::{FileListId, FileListIndex};
use servo_constellation_traits::SerializableFileList;

use crate::dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

// https://w3c.github.io/FileAPI/#dfn-filelist
#[dom_struct]
pub(crate) struct FileList {
    reflector_: Reflector,
    list: Vec<Dom<File>>,
}

impl FileList {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_inherited(files: Vec<Dom<File>>) -> FileList {
        FileList {
            reflector_: Reflector::new(),
            list: files,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        files: Vec<DomRoot<File>>,
    ) -> DomRoot<FileList> {
        reflect_dom_object_with_cx(
            Box::new(FileList::new_inherited(
                files.iter().map(|r| Dom::from_ref(&**r)).collect(),
            )),
            window,
            cx,
        )
    }

    pub(crate) fn new_in_global(
        cx: &mut JSContext,
        global: &GlobalScope,
        files: Vec<DomRoot<File>>,
    ) -> DomRoot<FileList> {
        reflect_dom_object_with_cx(
            Box::new(FileList::new_inherited(
                files.iter().map(|r| Dom::from_ref(&**r)).collect(),
            )),
            global,
            cx,
        )
    }

    pub(crate) fn iter_files(&self) -> Iter<'_, Dom<File>> {
        self.list.iter()
    }
}

impl Serializable for FileList {
    type Index = FileListIndex;
    type Data = SerializableFileList;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self, no_gc: &NoGC) -> Result<(FileListId, SerializableFileList), ()> {
        let files = self
            .list
            .iter()
            .map(|file| file.serialized_data(no_gc))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((FileListId::new(), SerializableFileList { files }))
    }

    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        cx: &mut JSContext,
        owner: &GlobalScope,
        serialized: SerializableFileList,
    ) -> Result<DomRoot<Self>, ()> {
        let files = serialized
            .files
            .into_iter()
            .map(|file| File::deserialize(cx, owner, file))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FileList::new_in_global(cx, owner, files))
    }

    fn serialized_storage<'a>(
        reader: StructuredData<'a, '_>,
    ) -> &'a mut Option<rustc_hash::FxHashMap<FileListId, Self::Data>> {
        match reader {
            StructuredData::Reader(r) => &mut r.file_lists,
            StructuredData::Writer(w) => &mut w.file_lists,
        }
    }
}

impl FileListMethods<crate::DomTypeHolder> for FileList {
    /// <https://w3c.github.io/FileAPI/#dfn-length>
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    /// <https://w3c.github.io/FileAPI/#dfn-item>
    fn Item(&self, index: u32) -> Option<DomRoot<File>> {
        if (index as usize) < self.list.len() {
            Some(DomRoot::from_ref(&*(self.list[index as usize])))
        } else {
            None
        }
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<File>> {
        self.Item(index)
    }
}
