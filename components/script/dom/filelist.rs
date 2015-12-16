/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileListBinding;
use dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::file::File;
use dom::window::Window;

// https://w3c.github.io/FileAPI/#dfn-filelist
#[dom_struct]
pub struct FileList {
    reflector_: Reflector,
    list: Vec<JS<File>>
}

impl FileList {
    #[allow(unrooted_must_root)]
    fn new_inherited(files: Vec<JS<File>>) -> FileList {
        FileList {
            reflector_: Reflector::new(),
            list: files
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, files: Vec<JS<File>>) -> Root<FileList> {
        reflect_dom_object(box FileList::new_inherited(files), GlobalRef::Window(window), FileListBinding::Wrap)
    }
}

impl FileListMethods for FileList {
    // https://w3c.github.io/FileAPI/#dfn-length
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/FileAPI/#dfn-item
    fn Item(&self, index: u32) -> Option<Root<File>> {
        Some(Root::from_ref(&*(self.list[index as usize])))
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<File>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}
