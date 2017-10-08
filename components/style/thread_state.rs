/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Supports dynamic assertions in about what sort of thread is running and
//! what state it's in.

#![deny(missing_docs)]

use std::cell::RefCell;

bitflags! {
    /// A thread state flag, used for multiple assertions.
    pub flags ThreadState: u32 {
        /// Whether we're in a script thread.
        const SCRIPT          = 0x01,
        /// Whether we're in a layout thread.
        const LAYOUT          = 0x02,

        /// Whether we're in a script worker thread (actual web workers), or in
        /// a layout worker thread.
        const IN_WORKER       = 0x0100,

        /// Whether the current thread is going through a GC.
        const IN_GC           = 0x0200,
    }
}

macro_rules! thread_types ( ( $( $fun:ident = $flag:ident ; )* ) => (
    impl ThreadState {
        /// Whether the current thread is a worker thread.
        pub fn is_worker(self) -> bool {
            self.contains(IN_WORKER)
        }

        $(
            #[allow(missing_docs)]
            pub fn $fun(self) -> bool {
                self.contains($flag)
            }
        )*
    }
));

thread_types! {
    is_script = SCRIPT;
    is_layout = LAYOUT;
}

thread_local!(static STATE: RefCell<Option<ThreadState>> = RefCell::new(None));

/// Initializes the current thread state.
pub fn initialize(x: ThreadState) {
    STATE.with(|ref k| {
        if let Some(ref s) = *k.borrow() {
            if x != *s {
                panic!("Thread state already initialized as {:?}", s);
            }
        }
        *k.borrow_mut() = Some(x);
    });
}

/// Initializes the current thread as a layout worker thread.
pub fn initialize_layout_worker_thread() {
    initialize(LAYOUT | IN_WORKER);
}

/// Gets the current thread state.
pub fn get() -> ThreadState {
    let state = STATE.with(|ref k| {
        match *k.borrow() {
            None => ThreadState::empty(), // Unknown thread.
            Some(s) => s,
        }
    });

    state
}

/// Enters into a given temporary state. Panics if re-entring.
pub fn enter(x: ThreadState) {
    let state = get();
    debug_assert!(!state.intersects(x));
    STATE.with(|ref k| {
        *k.borrow_mut() = Some(state | x);
    })
}

/// Exits a given temporary state.
pub fn exit(x: ThreadState) {
    let state = get();
    debug_assert!(state.contains(x));
    STATE.with(|ref k| {
        *k.borrow_mut() = Some(state & !x);
    })
}
