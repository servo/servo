/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::MediaStreamBinding;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MediaStream {
    eventtarget: EventTarget,
}

impl MediaStream {
    pub fn new_inherited() -> MediaStream {
        MediaStream {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<MediaStream> {
        reflect_dom_object(
            Box::new(MediaStream::new_inherited()),
            global,
            MediaStreamBinding::Wrap,
        )
    }
}
