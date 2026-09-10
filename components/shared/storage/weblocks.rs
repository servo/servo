use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;
use servo_url::ImmutableOrigin;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLocksThreadMsg {
    Request(GenericCallback<LockMsg>, String, ImmutableOrigin), // TODO: options
    Query(GenericCallback<LockManagerSnapshotMsg>, ImmutableOrigin),
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
    Exclusive,
    Shared,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockMsg {
    pub name: String,
    pub mode: LockModeMsg,
}
