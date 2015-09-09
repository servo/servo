/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

static NEXT_TID: AtomicUsize = ATOMIC_USIZE_INIT;

thread_local!(static TASK_LOCAL_TID: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None)));

/// Every task gets one, that's unique.
pub fn tid() -> usize {
    TASK_LOCAL_TID.with(|ref k| {
        let ret =
            match *k.borrow() {
                None => NEXT_TID.fetch_add(1, Ordering::SeqCst),
                Some(x) => x,
            };

        *k.borrow_mut() = Some(ret);

        ret
    })
}
