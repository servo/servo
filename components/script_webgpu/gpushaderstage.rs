/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::reflector::Reflector;

#[dom_struct]
pub struct GPUShaderStage<D: DomTypes> {
    reflector_: Reflector,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}
