/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use selector_impl::{Animation, GeckoSelectorImpl, SharedStyleContext};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use style::context::{LocalStyleContext, StyleContext};
use style::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};

thread_local!(static LOCAL_CONTEXT_KEY:
                RefCell<Option<Rc<LocalStyleContext<GeckoSelectorImpl>>>> = RefCell::new(None));

// Keep this implementation in sync with the one in components/layout/context.rs.
fn create_or_get_local_context(shared: &SharedStyleData)
                               -> Rc<LocalStyleContext<GeckoSelectorImpl>> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            if shared.shared_style_context.screen_size_changed {
                context.applicable_declarations_cache.borrow_mut().evict_all();
            }
            context
        } else {
            let new_animations_sender = shared.new_animations_sender.lock().unwrap().clone();
            let context = Rc::new(LocalStyleContext {
                applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
                style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
                new_animations_sender: new_animations_sender,
            });
            *r = Some(context.clone());
            context
        }
    })
}

/// A shared style context, plus data that we need to create a local one.
pub struct SharedStyleData {
    pub shared_style_context: SharedStyleContext,
    pub new_animations_sender: Mutex<Sender<Animation>>,
}

pub struct StandaloneStyleContext<'a> {
    pub shared_data: &'a SharedStyleData,
    cached_local_context: Rc<LocalStyleContext<GeckoSelectorImpl>>,
}

impl<'a> StandaloneStyleContext<'a> {
    pub fn new(shared: &'a SharedStyleData) -> Self {
        let local_context = create_or_get_local_context(shared);
        StandaloneStyleContext {
            shared_data: shared,
            cached_local_context: local_context,
        }
    }
}

impl<'a> StyleContext<'a, GeckoSelectorImpl> for StandaloneStyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared_data.shared_style_context
    }

    fn local_context(&self) -> &LocalStyleContext<GeckoSelectorImpl> {
        &self.cached_local_context
    }
}
