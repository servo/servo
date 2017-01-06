/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Supports dynamic assertions in debug builds about what sort of thread is
//! running and what state it's in.
//!
//! In release builds, `get` returns 0.  All of the other functions inline
//! away to nothing.

#![deny(missing_docs)]

pub use self::imp::{enter, exit, get, initialize};

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
            #[cfg(debug_assertions)]
            #[allow(missing_docs)]
            pub fn $fun(self) -> bool {
                self.contains($flag)
            }
            #[cfg(not(debug_assertions))]
            #[allow(missing_docs)]
            pub fn $fun(self) -> bool {
                true
            }
        )*
    }

    #[cfg(debug_assertions)]
    static TYPES: &'static [ThreadState] =
        &[ $( $flag ),* ];
));

thread_types! {
    is_script = SCRIPT;
    is_layout = LAYOUT;
}

#[cfg(debug_assertions)]
mod imp {
    use std::cell::RefCell;
    use super::{TYPES, ThreadState};

    thread_local!(static STATE: RefCell<Option<ThreadState>> = RefCell::new(None));

    /// Initialize the current thread state.
    pub fn initialize(x: ThreadState) {
        STATE.with(|ref k| {
            if let Some(ref s) = *k.borrow() {
                panic!("Thread state already initialized as {:?}", s);
            }
            *k.borrow_mut() = Some(x);
        });
        get(); // check the assertion below
    }

    /// Get the current thread state.
    pub fn get() -> ThreadState {
        let state = STATE.with(|ref k| {
            match *k.borrow() {
                // This is one of the layout threads, that use rayon.
                None => super::LAYOUT | super::IN_WORKER,
                Some(s) => s,
            }
        });

        // Exactly one of the thread type flags should be set.
        assert_eq!(1, TYPES.iter().filter(|&&ty| state.contains(ty)).count());
        state
    }

    /// Enter into a given temporary state. Panics if re-entring.
    pub fn enter(x: ThreadState) {
        let state = get();
        assert!(!state.intersects(x));
        STATE.with(|ref k| {
            *k.borrow_mut() = Some(state | x);
        })
    }

    /// Exit a given temporary state.
    pub fn exit(x: ThreadState) {
        let state = get();
        assert!(state.contains(x));
        STATE.with(|ref k| {
            *k.borrow_mut() = Some(state & !x);
        })
    }
}

#[cfg(not(debug_assertions))]
#[allow(missing_docs)]
mod imp {
    use super::ThreadState;
    #[inline(always)] pub fn initialize(_: ThreadState) { }
    #[inline(always)] pub fn get() -> ThreadState { ThreadState::empty() }
    #[inline(always)] pub fn enter(_: ThreadState) { }
    #[inline(always)] pub fn exit(_: ThreadState) { }
}
