use std::sync::atomics::{AtomicUint, INIT_ATOMIC_UINT, SeqCst};

static mut next_tid: AtomicUint = INIT_ATOMIC_UINT;

local_data_key!(task_local_tid: uint)

/// Every task gets one, that's unique.
pub fn tid() -> uint {
    let ret =
        match task_local_tid.replace(None) {
            None => unsafe { next_tid.fetch_add(1, SeqCst) },
            Some(x) => x,
        };

    task_local_tid.replace(Some(ret));

    ret
}
