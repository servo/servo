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
use typeholder::TypeHolderTrait;

// https://w3c.github.io/FileAPI/#dfn-filelist
#[dom_struct]
pub struct FileList<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    list: Vec<Dom<File<TH>>>
}

impl<TH: TypeHolderTrait> FileList<TH> {
    #[allow(unrooted_must_root)]
    fn new_inherited(files: Vec<Dom<File<TH>>>) -> FileList<TH> {
        FileList {
            reflector_: Reflector::new(),
            list: files
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, files: Vec<DomRoot<File<TH>>>) -> DomRoot<FileList<TH>> {
        reflect_dom_object(Box::new(FileList::new_inherited(files.iter().map(|r| Dom::from_ref(&**r)).collect())),
                           window,
                           FileListBinding::Wrap)
    }

    pub fn iter_files(&self) -> Iter<Dom<File<TH>>> {
        self.list.iter()
    }
}

impl<TH: TypeHolderTrait> FileListMethods<TH> for FileList<TH> {
    // https://w3c.github.io/FileAPI/#dfn-length
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/FileAPI/#dfn-item
    fn Item(&self, index: u32) -> Option<DomRoot<File<TH>>> {
        if (index as usize) < self.list.len() {
            Some(DomRoot::from_ref(&*(self.list[index as usize])))
        } else {
            None
        }
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<File<TH>>> {
        self.Item(index)
    }
}
