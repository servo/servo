use std::cell::Cell;

use crate::bidi::{connection::ConnectionId, session::common::SessionId};

// partial model of a resume wait queue

thread_local! {
    static RESUME_ID: Cell<u64> = const { Cell::new(0) };
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ResumeId(u64);

impl ResumeId {
    pub(crate) fn next() -> Self {
        RESUME_ID.with(|c| Self(c.replace(c.get() + 1)))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ResumeEvent {
    SessionResponded(SessionId),
}

/// <https://www.w3.org/TR/webdriver-bidi/#wait-queue>
pub(crate) struct WaitQueue {}

impl WaitQueue {
    pub(crate) fn awaits<T, Fut>(&self, events: &[ResumeEvent], fut: Fut)
    where
        Fut: Future<Output = T>,
    {
        // TODO:
        // task::spawn_local({ recv.await; algo() })
    }

    pub(crate) fn resume(&self, event: ResumeEvent) {
        // TODO:
    }
}
