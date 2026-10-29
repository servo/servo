/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefMut};
use std::rc::Rc;

use js::context::NoGC;
use script_bindings::cell::DomRefCell;

use crate::dom::internalclosewatcher::InternalCloseWatcher;

/// <https://html.spec.whatwg.org/multipage/#close-watcher-manager>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct CloseWatcherManager {
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-manager-groups>
    #[conditional_malloc_size_of]
    groups: DomRefCell<Vec<Vec<Rc<InternalCloseWatcher>>>>,
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-manager-allowed-number-of-groups>
    allowed_number_of_groups: Cell<usize>,
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-manager-next>
    next_user_interaction_allows_a_new_group: Cell<bool>,
}

impl CloseWatcherManager {
    pub(crate) fn new() -> Self {
        Self {
            groups: Default::default(),
            allowed_number_of_groups: Cell::new(1),
            next_user_interaction_allows_a_new_group: Cell::new(true),
        }
    }

    pub(crate) fn groups(&self) -> Ref<'_, Vec<Vec<Rc<InternalCloseWatcher>>>> {
        self.groups.borrow()
    }

    pub(crate) fn groups_mut<'a: 'r, 'no_gc: 'r, 'r>(
        &'a self,
        no_gc: &'no_gc NoGC,
    ) -> RefMut<'r, Vec<Vec<Rc<InternalCloseWatcher>>>> {
        self.groups.safe_borrow_mut(no_gc)
    }

    pub(crate) fn allowed_number_of_groups(&self) -> usize {
        self.allowed_number_of_groups.get()
    }

    pub(crate) fn increment_allowed_number_of_groups(&self) {
        self.allowed_number_of_groups
            .update(|allowed_number_of_groups| allowed_number_of_groups + 1);
    }

    pub(crate) fn next_user_interaction_allows_a_new_group(&self) -> bool {
        self.next_user_interaction_allows_a_new_group.get()
    }

    pub(crate) fn set_next_user_interaction_allows_a_new_group(&self, value: bool) {
        self.next_user_interaction_allows_a_new_group.set(value);
    }
}
