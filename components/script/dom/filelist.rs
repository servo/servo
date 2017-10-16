/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileListBinding;
use dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::file::File;
use dom::window::Window;
use dom_struct::dom_struct;
use std::slice::Iter;

// https://w3c.github.io/FileAPI/#dfn-filelist
#[dom_struct]
pub struct FileList {
    reflector_: Reflector,
    list: Vec<Dom<File>>
}

impl FileList {
    #[allow(unrooted_must_root)]
    fn new_inherited(files: Vec<Dom<File>>) -> FileList {
        FileList {
            reflector_: Reflector::new(),
            list: files
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, files: Vec<DomRoot<File>>) -> DomRoot<FileList> {
        reflect_dom_object(Box::new(FileList::new_inherited(files.iter().map(|r| Dom::from_ref(&**r)).collect())),
                           window,
                           FileListBinding::Wrap)
    }

    pub fn iter_files(&self) -> Iter<Dom<File>> {
        self.list.iter()
    }
}

impl FileListMethods for FileList {
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
