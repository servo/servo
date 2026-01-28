/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use crate::dom::closewatcher::InternalCloseWatcher;

/// <https://html.spec.whatwg.org/multipage/#close-watcher-manager>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct CloseWatcherManager {
    #[no_trace]
    #[ignore_malloc_size_of = "Idk"]
    groups: Vec<Vec<NonNull<InternalCloseWatcher>>>,
    allowed_number_of_groups: Cell<usize>,
    next_user_interaction_allows_a_new_group: Cell<bool>,
}

impl CloseWatcherManager {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            allowed_number_of_groups: Cell::new(1),
            next_user_interaction_allows_a_new_group: Cell::new(true),
        }
    }

    pub fn number_of_groups(&self) -> usize {
        self.groups.len()
    }

    pub fn allowed_number_of_groups(&self) -> usize {
        self.allowed_number_of_groups.get()
    }

    pub fn increment_allowed_number_of_groups(&self) {
        self.allowed_number_of_groups
            .set(self.allowed_number_of_groups.get() + 1);
    }

    pub fn next_user_interaction_allows_a_new_group(&self) -> bool {
        self.next_user_interaction_allows_a_new_group.get()
    }

    pub fn set_next_user_interaction_allows_a_new_group(&self, value: bool) {
        self.next_user_interaction_allows_a_new_group.set(value);
    }

    pub fn append_group(&mut self, group: Vec<NonNull<InternalCloseWatcher>>) {
        self.groups.push(group)
    }

    pub fn append_close_watcher(&mut self, watcher: NonNull<InternalCloseWatcher>) {
        if let Some(last_item) = self.groups.last_mut() {
            last_item.push(watcher)
        }
    }

    pub fn remove_close_watcher(&mut self, watcher: &InternalCloseWatcher) {
        // Step 2. For each group of manager's groups: remove closeWatcher from group.
        for group in &mut self.groups {
            group.retain(|w| *w != NonNull::from(watcher))
        }

        // Step 3. Remove any item from manager's groups that is empty.
        self.groups.retain(|group| !group.is_empty())
    }
}
