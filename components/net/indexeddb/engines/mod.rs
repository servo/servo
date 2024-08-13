/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::future::Future;

use ipc_channel::ipc::IpcSender;
use net_traits::indexeddb_thread::{AsyncOperation, IndexedDBTxnMode};
use tokio::sync::oneshot::error::RecvError;

pub use self::rkv::RkvEngine;

// mod heed;
mod rkv;

#[derive(Eq, Hash, PartialEq)]
pub struct SanitizedName {
    name: String,
}

impl SanitizedName {
    pub fn new(name: String) -> SanitizedName {
        let name = name.replace("https://", "");
        let name = name.replace("http://", "");
        // FIXME:(rasviitanen) Disallowing special characters might be a big problem,
        // but better safe than sorry. E.g. the db name '../other_origin/db',
        // would let us access databases from another origin.
        let name = name
            .chars()
            .map(|c| match c {
                'A'..='Z' => c,
                'a'..='z' => c,
                '0'..='9' => c,
                '-' => c,
                '_' => c,
                _ => '-',
            })
            .collect();
        SanitizedName { name }
    }

    pub fn to_string(&self) -> String {
        self.name.clone()
    }
}

pub struct KvsOperation {
    pub sender: IpcSender<Option<Vec<u8>>>,
    pub store_name: SanitizedName,
    pub operation: AsyncOperation,
}

pub struct KvsTransaction {
    pub mode: IndexedDBTxnMode,
    pub requests: VecDeque<KvsOperation>,
}

pub trait KvsEngine {
    fn create_store(&self, store_name: SanitizedName, auto_increment: bool);

    fn process_transaction<'a>(
        &self,
        transaction: KvsTransaction,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>, RecvError>> + Send + 'a>>
    where
        Self: Sync + 'a;

    fn has_key_generator(&self, store_name: SanitizedName) -> bool;
}
