use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLocksThreadMsg {
    Request(GenericCallback<LockMsg>, String), // TODO: options
    Query(GenericCallback<LockManagerSnapshotMsg>),
    CollectMemoryReport(ReportsChan),
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub enum LockModeMsg {
    Exclusive,
    Shared,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockMsg {
    pub name: String,
    pub mode: LockModeMsg,
}
