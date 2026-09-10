use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;
use servo_url::ImmutableOrigin;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLocksThreadMsg {
    Request(LockRequest, ImmutableOrigin),
    Query(GenericCallback<LockManagerSnapshotMsg>, ImmutableOrigin),
    Release(),
    Abort(),
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
    pub client_id: String,
    pub name: String,
    pub mode: LockModeMsg,
    pub callback: GenericCallback<LockMsg>,
    pub if_available: bool,
    pub steal: bool,
}
