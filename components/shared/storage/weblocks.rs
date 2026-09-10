use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{GenericCallback, GenericSender};

#[derive(Debug, Deserialize, Serialize)]
pub enum WebLocksThreadMsg {
    Request(GenericSender<LockMsg>, String), // TODO: options
    Query(GenericCallback<LockManagerSnapshotMsg>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockManagerSnapshotMsg {
    held: Vec<LockInfoMsg>,
    pending: Vec<LockInfoMsg>,
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
