/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::{SharedStyleContext, ThreadLocalStyleContext};
use std::cell::RefCell;
use std::rc::Rc;

thread_local!(static LOCAL_CONTEXT_KEY: RefCell<Option<Rc<ThreadLocalStyleContext>>> = RefCell::new(None));

// Keep this implementation in sync with the one in components/layout/context.rs.
pub fn create_or_get_local_context(shared: &SharedStyleContext) -> Rc<ThreadLocalStyleContext> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            context
        } else {
            let context = Rc::new(ThreadLocalStyleContext::new(&shared.local_context_creation_data.lock().unwrap()));
            *r = Some(context.clone());
            context
        }
    })
}

pub fn clear_local_context() {
    LOCAL_CONTEXT_KEY.with(|r| *r.borrow_mut() = None);
}
