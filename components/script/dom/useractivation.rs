/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Add;

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::UserActivationBinding::UserActivationMethods;
use time::Duration;

use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::{
    Document, SameOriginDescendantNavigablesIterator, SameoriginAncestorNavigablesIterator,
};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#the-useractivation-interface>
#[dom_struct]
pub(crate) struct UserActivation {
    reflector_: Reflector,
}

impl UserActivation {
    fn new_inherited() -> UserActivation {
        UserActivation {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<UserActivation> {
        reflect_dom_object(Box::new(UserActivation::new_inherited()), global, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#activation-notification>
    pub(crate) fn handle_user_activation_notification(document: &Document) {
        // Step 1.
        // > Assert: document is fully active.
        debug_assert!(
            document.is_fully_active(),
            "Document should be fully active at this moment"
        );

        // Step 2.
        // > Let windows be « document's relevant global object ».
        let owner_window = document.window();
        rooted_vec!(let mut windows <- vec![Dom::from_ref(owner_window)].into_iter());

        // Step 3.
        // > Extend windows with the active window of each of document's ancestor navigables.
        // TODO: this would not work for disimilar origin ancestor, since we don't store the document in this script thread.
        for document in SameoriginAncestorNavigablesIterator::new(DomRoot::from_ref(document)) {
            windows.push(Dom::from_ref(document.window()));
        }

        // Step 4.
        // > Extend windows with the active window of each of document's descendant navigables, filtered to include only
        // > those navigables whose active document's origin is same origin with document's origin.
        for document in SameOriginDescendantNavigablesIterator::new(DomRoot::from_ref(document)) {
            windows.push(Dom::from_ref(document.window()));
        }

        // Step 5.
        // > For each window in windows:
        let current_timestamp = CrossProcessInstant::now();
        for window in windows.iter() {
            // Step 5.1.
            // > Set window's last activation timestamp to the current high resolution time.
            window.set_last_activation_timestamp(UserActivationTimestamp::TimeStamp(
                current_timestamp,
            ));

            // Step 5.2.
            // > Notify the close watcher manager about user activation given window.
            // TODO: impl close watcher
        }
    }
}

impl UserActivationMethods<crate::DomTypeHolder> for UserActivation {
    /// <https://html.spec.whatwg.org/multipage/#dom-useractivation-hasbeenactive>
    fn HasBeenActive(&self) -> bool {
        // > The hasBeenActive getter steps are to return true if this's relevant global object has sticky activation, and false otherwise.
        self.global().as_window().has_sticky_activation()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-useractivation-isactive>
    fn IsActive(&self) -> bool {
        // > The isActive getter steps are to return true if this's relevant global object has transient activation, and false otherwise.
        self.global().as_window().has_transient_activation()
    }
}

/// Timestamp definition specific to [`UserActivation`].
/// > ... which is either a DOMHighResTimeStamp, positive infinity (indicating that W has never been activated), or negative infinity
/// > (indicating that the activation has been consumed). Initially positive infinity.
/// > <https://html.spec.whatwg.org/multipage/#user-activation-data-model>
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, MallocSizeOf)]
pub(crate) enum UserActivationTimestamp {
    NegativeInfinity,
    TimeStamp(CrossProcessInstant),
    #[default]
    PositiveInfinity,
}

impl Add<i64> for UserActivationTimestamp {
    type Output = UserActivationTimestamp;

    fn add(self, rhs: i64) -> Self::Output {
        match self {
            UserActivationTimestamp::TimeStamp(timestamp) => {
                UserActivationTimestamp::TimeStamp(timestamp + Duration::milliseconds(rhs))
            },
            _ => self,
        }
    }
}
