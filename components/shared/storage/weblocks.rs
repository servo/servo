use std::sync::atomic::{AtomicUsize, Ordering};

use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;
use servo_url::ImmutableOrigin;
use uuid::Uuid;

static LOCK_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LockId(usize);

impl LockId {
    pub fn next() -> Self {
        Self(LOCK_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// TODO: once servo implements client id, this can be changed to usize
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize,
)]
pub struct LockRequestId(Uuid);

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLocksThreadMsg {
    Request(LockRequest, ImmutableOrigin),
    Query(GenericCallback<LockManagerSnapshotMsg>, ImmutableOrigin),
    Release(String, ImmutableOrigin),
    Abort(LockRequestId, ImmutableOrigin, String),
    CollectMemoryReport(ReportsChan),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LockManagerSnapshotMsg {
    pub held: Vec<LockInfoMsg>,
    pub pending: Vec<LockInfoMsg>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockInfoMsg {
    pub name: String,
    pub mode: LockModeMsg,
    pub client_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LockModeMsg {
    Shared,
    Exclusive,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockMsg {
    pub name: String,
    pub mode: LockModeMsg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockRequest {
    pub id: LockRequestId,
    pub client_id: String,
    pub name: String,
    pub mode: LockModeMsg,
    pub callback: GenericCallback<Option<LockMsg>>,
    pub if_available: bool,
    pub steal: bool,
}
