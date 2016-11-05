/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::{LocalStyleContext, StyleContext, SharedStyleContext};
use std::cell::RefCell;
use std::rc::Rc;

thread_local!(static LOCAL_CONTEXT_KEY: RefCell<Option<Rc<LocalStyleContext>>> = RefCell::new(None));

// Keep this implementation in sync with the one in components/layout/context.rs.
fn create_or_get_local_context(shared: &SharedStyleContext) -> Rc<LocalStyleContext> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            context
        } else {
            let context = Rc::new(LocalStyleContext::new(&shared.local_context_creation_data.lock().unwrap()));
            *r = Some(context.clone());
            context
        }
    })
}

pub struct StandaloneStyleContext<'a> {
    pub shared: &'a SharedStyleContext,
    cached_local_context: Rc<LocalStyleContext>,
}

impl<'a> StandaloneStyleContext<'a> {
    pub fn new(shared: &'a SharedStyleContext) -> Self {
        let local_context = create_or_get_local_context(shared);
        StandaloneStyleContext {
            shared: shared,
            cached_local_context: local_context,
        }
    }
}

impl<'a> StyleContext<'a> for StandaloneStyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared
    }

    fn local_context(&self) -> &LocalStyleContext {
        &self.cached_local_context
    }
}
