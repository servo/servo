/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::slice::Iter;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::file::File;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://w3c.github.io/FileAPI/#dfn-filelist
#[dom_struct]
pub(crate) struct FileList {
    reflector_: Reflector,
    list: Vec<Dom<File>>,
}

impl FileList {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(files: Vec<Dom<File>>) -> FileList {
        FileList {
            reflector_: Reflector::new(),
            list: files,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        files: Vec<DomRoot<File>>,
        can_gc: CanGc,
    ) -> DomRoot<FileList> {
        reflect_dom_object(
            Box::new(FileList::new_inherited(
                files.iter().map(|r| Dom::from_ref(&**r)).collect(),
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn iter_files(&self) -> Iter<Dom<File>> {
        self.list.iter()
    }
}

impl FileListMethods<crate::DomTypeHolder> for FileList {
    // https://w3c.github.io/FileAPI/#dfn-length
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/FileAPI/#dfn-item
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
