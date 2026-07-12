/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::FileSystemBinding::FileSystemMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystem {
    reflector_: Reflector,
    name: USVString,
    root: Dom<FileSystemDirectoryEntry>,
}

impl FileSystem {
    pub(crate) fn new_inherited(name: USVString, root: &FileSystemDirectoryEntry) -> FileSystem {
        FileSystem {
            reflector_: Reflector::new(),
            name,
            root: Dom::from_ref(root),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        name: USVString,
        root: &FileSystemDirectoryEntry,
    ) -> DomRoot<FileSystem> {
        reflect_dom_object_with_cx(Box::new(FileSystem::new_inherited(name, root)), global, cx)
    }
}

impl FileSystemMethods<crate::DomTypeHolder> for FileSystem {
    /// <https://wicg.github.io/entries-api/#dom-filesystem-name>
    fn Name(&self) -> USVString {
        self.name.clone()
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystem-root>
    fn Root(&self) -> DomRoot<FileSystemDirectoryEntry> {
        DomRoot::from_ref(&self.root)
    }
}
