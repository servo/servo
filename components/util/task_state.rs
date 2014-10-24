/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Supports dynamic assertions in debug builds about what sort of task is
//! running and what state it's in.
//!
//! In release builds, `get` is not available; calls must be inside
//! `debug_assert!` or similar.  All of the other functions inline away to
//! nothing.

pub use self::imp::{initialize, enter, exit};

#[cfg(not(ndebug))]
pub use self::imp::get;

bitflags! {
    #[deriving(Show)]
    flags TaskState: u32 {
        static Script        = 0x01,
        static Layout        = 0x02,
        static Render        = 0x04,

        static InWorker      = 0x0100,
        static InGC          = 0x0200,
        static InHTMLParser  = 0x0400,
    }
}

// Exactly one of these should be set.
static task_types: &'static [TaskState]
    = &[Script, Layout, Render];

macro_rules! predicates ( ( $( $f:ident = $c:ident ; )* ) => (
    impl TaskState {
        $(
            pub fn $f(self) -> bool {
                self.contains($c)
            }
        )*
    }
))

predicates! {
    is_script = Script;
    is_layout = Layout;
    is_render = Render;
}

#[cfg(not(ndebug))]
mod imp {
    use super::{TaskState, task_types};

    local_data_key!(STATE: TaskState)

    pub fn initialize(x: TaskState) {
        match STATE.replace(Some(x)) {
            None => (),
            Some(s) => fail!("Task state already initialized as {}", s),
        };
        get(); // check the assertion below
    }

    pub fn get() -> TaskState {
        let state = match STATE.get() {
            None => fail!("Task state not initialized"),
            Some(s) => *s,
        };

        // Exactly one of the task type flags should be set.
        assert_eq!(1, task_types.iter().filter(|&&ty| state.contains(ty)).count());
        state
    }

    pub fn enter(x: TaskState) {
        let state = get();
        assert!(!state.intersects(x));
        STATE.replace(Some(state | x));
    }

    pub fn exit(x: TaskState) {
        let state = get();
        assert!(state.contains(x));
        STATE.replace(Some(state & !x));
    }
}

#[cfg(ndebug)]
mod imp {
    use super::TaskState;
    #[inline(always)] pub fn initialize(_: TaskState) { }
    #[inline(always)] pub fn enter(_: TaskState) { }
    #[inline(always)] pub fn exit(_: TaskState) { }
}
