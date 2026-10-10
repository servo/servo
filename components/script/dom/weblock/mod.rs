use script_bindings::codegen::GenericBindings::WebLockBinding::LockMode;
use storage_traits::weblocks::{LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebLockBinding::{LockInfo, LockManagerSnapshot};

pub(crate) mod lock;
pub(crate) mod lockmanager;

impl Convert<LockManagerSnapshot> for LockManagerSnapshotMsg {
    fn convert(self) -> LockManagerSnapshot {
        let mut snapshot = LockManagerSnapshot::empty();
        snapshot.held = Some(self.held.into_iter().map(Convert::convert).collect());
        snapshot.pending = Some(self.pending.into_iter().map(Convert::convert).collect());
        snapshot
    }
}

impl Convert<LockInfo> for LockInfoMsg {
    fn convert(self) -> LockInfo {
        let mut lock_info = LockInfo::empty();
        lock_info.name = Some(self.name.into());
        lock_info.mode = Some(self.mode.convert());
        lock_info.clientId = Some(self.client_id.into());
        lock_info
    }
}

impl Convert<LockMode> for LockModeMsg {
    fn convert(self) -> LockMode {
        match self {
            LockModeMsg::Shared => LockMode::Shared,
            LockModeMsg::Exclusive => LockMode::Exclusive,
        }
    }
}

impl Convert<LockModeMsg> for LockMode {
    fn convert(self) -> LockModeMsg {
        match self {
            LockMode::Shared => LockModeMsg::Shared,
            LockMode::Exclusive => LockModeMsg::Exclusive,
        }
    }
}
