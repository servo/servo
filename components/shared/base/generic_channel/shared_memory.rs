/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use ipc_channel::ipc::IpcSharedMemory;
use malloc_size_of::MallocSizeOf;
use serde::de::VariantAccess;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_config::opts;

#[derive(Clone)]
pub struct GenericSharedMemory(GenericSharedMemoryVariant);

#[derive(Clone)]
enum GenericSharedMemoryVariant {
    Ipc(IpcSharedMemory),
    InProcess(Arc<Vec<u8>>),
}

impl Deref for GenericSharedMemory {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        match &self.0 {
            GenericSharedMemoryVariant::Ipc(ipc_shared_memory) => ipc_shared_memory,
            GenericSharedMemoryVariant::InProcess(items) => items.as_slice(),
        }
    }
}

impl MallocSizeOf for GenericSharedMemory {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        match &self.0 {
            GenericSharedMemoryVariant::Ipc(_) => 0,
            GenericSharedMemoryVariant::InProcess(items) => items.size_of(ops),
        }
    }
}

impl GenericSharedMemory {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
            GenericSharedMemory(GenericSharedMemoryVariant::Ipc(
                IpcSharedMemory::from_bytes(bytes),
            ))
        } else {
            GenericSharedMemory(GenericSharedMemoryVariant::InProcess(Arc::new(
                bytes.to_owned(),
            )))
        }
    }

    pub fn from_byte(data: u8, length: usize) -> Self {
        if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
            GenericSharedMemory(GenericSharedMemoryVariant::Ipc(IpcSharedMemory::from_byte(
                data, length,
            )))
        } else {
            GenericSharedMemory(GenericSharedMemoryVariant::InProcess(Arc::new(vec![
                data;
                length
            ])))
        }
    }
}

impl fmt::Debug for GenericSharedMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GenericSharedMemory").finish()
    }
}

impl Serialize for GenericSharedMemory {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            GenericSharedMemoryVariant::Ipc(memory) => {
                s.serialize_newtype_variant("GenericSharedMemory", 0, "Ipc", memory)
            },
            GenericSharedMemoryVariant::InProcess(arc) => {
                if opts::get().multiprocess {
                    return Err(serde::ser::Error::custom(
                        "Arc<Vec<u8>> found in multiprocess mode!",
                    ));
                } // We know everything is in one address-space, so we can "serialize" the receiver by
                // sending a leaked Box pointer.
                let address = Arc::into_raw(arc.clone()) as *mut Vec<u8> as usize;
                s.serialize_newtype_variant("GenericSharedMemory", 1, "InProcess", &address)
            },
        }
    }
}

struct GenericSharedMemoryVisitor {}

impl<'de> serde::de::Visitor<'de> for GenericSharedMemoryVisitor {
    type Value = GenericSharedMemory;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a GenericReceiver variant")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        #[derive(Deserialize)]
        enum GenericSharedMemoryVariantNames {
            Ipc,
            InProcess,
        }

        let (variant_name, variant_data): (GenericSharedMemoryVariantNames, _) = data.variant()?;

        match variant_name {
            GenericSharedMemoryVariantNames::Ipc => variant_data
                .newtype_variant::<IpcSharedMemory>()
                .map(|receiver| GenericSharedMemory(GenericSharedMemoryVariant::Ipc(receiver))),
            GenericSharedMemoryVariantNames::InProcess => {
                if opts::get().multiprocess {
                    return Err(serde::de::Error::custom(
                        "Crossbeam channel found in multiprocess mode!",
                    ));
                }
                let addr = variant_data.newtype_variant::<usize>()?;
                let ptr = addr as *mut Vec<u8>;
                // SAFETY: We know we are in the same address space as the sender, so we can safely
                // reconstruct the Box.
                #[expect(unsafe_code)]
                let arc = unsafe { Arc::from_raw(ptr) };
                Ok(GenericSharedMemory(GenericSharedMemoryVariant::InProcess(
                    arc,
                )))
            },
        }
    }
}

impl<'a> Deserialize<'a> for GenericSharedMemory {
    fn deserialize<D>(d: D) -> Result<GenericSharedMemory, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericSharedMemory",
            &["Ipc", "InProcess"],
            GenericSharedMemoryVisitor {},
        )
    }
}

#[cfg(test)]
mod single_process_callback_test {
    use std::sync::Arc;

    use ipc_channel::ipc::IpcSharedMemory;

    use super::GenericSharedMemory;
    use crate::generic_channel::{self};

    #[test]
    fn test_ipc() {
        let bytes = vec![0xba; 10];
        let bytes_copy = bytes.clone();
        let shared_memory = GenericSharedMemory(super::GenericSharedMemoryVariant::Ipc(
            IpcSharedMemory::from_bytes(&bytes),
        ));

        let (send, recv) = generic_channel::channel().unwrap();
        send.send(shared_memory).expect("Could not send");
        assert_eq!(recv.recv().unwrap().to_vec(), bytes_copy);
    }

    #[test]
    fn test_inprocess() {
        let bytes = vec![0xba; 10];
        let bytes_copy = bytes.clone();
        let shared_memory = GenericSharedMemory(super::GenericSharedMemoryVariant::InProcess(
            Arc::new(bytes.clone()),
        ));

        let (send, recv) = generic_channel::channel().unwrap();
        send.send(shared_memory).expect("Could not send");
        assert_eq!(recv.recv().unwrap().to_vec(), bytes_copy);
    }
}
