/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserBinding::DOMParserMethods;
use dom::bindings::conversions::IDLInterface;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::MutDomObject;
use dom::bindings::root::DomRoot;
use dom::bindings::trace::JSTraceable;
use dom::window::Window;
use malloc_size_of::MallocSizeOf;
use typeholder::TypeHolderTrait;

pub trait DOMParserTrait<TH: TypeHolderTrait>:
    MutDomObject +
    IDLInterface +
    MallocSizeOf +
    JSTraceable +
    DOMParserMethods<TH> {
    fn Constructor(window: &Window<TH>) -> Fallible<DomRoot<TH::DOMParser>>;
}

