/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::rc::Rc;
use std::cell::RefCell;

static mut next_tid: AtomicUsize = ATOMIC_USIZE_INIT;

thread_local!(static TASK_LOCAL_TID: Rc<RefCell<Option<uint>>> = Rc::new(RefCell::new(None)));

/// Every task gets one, that's unique.
pub fn tid() -> uint {
    TASK_LOCAL_TID.with(|ref k| {
        let ret =
            match *k.borrow() {
                None => unsafe { next_tid.fetch_add(1, Ordering::SeqCst) },
                Some(x) => x,
            };

        *k.borrow_mut() = Some(ret);

        ret
    })
}
