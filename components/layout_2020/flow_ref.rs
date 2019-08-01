/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::flow::Flow;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct FlowRef(Arc<dyn Flow>);

impl Deref for FlowRef {
    type Target = dyn Flow;
    fn deref(&self) -> &dyn Flow {
        self.0.deref()
    }
}

impl FlowRef {
    pub fn new(mut r: Arc<dyn Flow>) -> Self {
        assert!(Arc::get_mut(&mut r).is_some());
        FlowRef(r)
    }

    #[allow(unsafe_code)]
    pub fn deref_mut(this: &mut FlowRef) -> &mut dyn Flow {
        let ptr: *const dyn Flow = &*this.0;
        unsafe { &mut *(ptr as *mut dyn Flow) }
    }
}
