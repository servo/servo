/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderSyncBinding;
use dom::bindings::error::Fallible;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct FileReaderSync {
    eventtarget: EventTarget
}

impl FileReaderSync {
    pub fn new_inherited() -> FileReaderSync {
        FileReaderSync {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<FileReaderSync> {
        reflect_dom_object(box FileReaderSync::new_inherited(),
                           global, FileReaderSyncBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope) -> Fallible<Root<FileReaderSync>> {
        Ok(FileReaderSync::new(global))
    }
}
