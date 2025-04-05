/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains data structures used for message ports and serializing
//! DOM objects to send across them as per
//! <https://html.spec.whatwg.org/multipage/#serializable-objects>.
//! The implementations are here instead of in `script``, because these
//! types can be sent through the Constellation to other ScriptThreads,
//! and Constellation cannot depend directly on `script`.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use base::id::{BlobId, DomPointId, MessagePortId};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::filemanager_thread::RelativePos;
use serde::{Deserialize, Serialize};
use servo_url::ImmutableOrigin;
use strum::{EnumIter, IntoEnumIterator};
use uuid::Uuid;

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
enum MessagePortState {
    /// <https://html.spec.whatwg.org/multipage/#detached>
    Detached,
    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    /// The message-queue of this port is enabled,
    /// the boolean represents awaiting completion of a transfer.
    Enabled(bool),
    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    /// The message-queue of this port is disabled,
    /// the boolean represents awaiting completion of a transfer.
    Disabled(bool),
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
/// The data and logic backing the DOM managed MessagePort.
pub struct MessagePortImpl {
    /// The current state of the port.
    state: MessagePortState,

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    entangled_port: Option<MessagePortId>,

    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    message_buffer: Option<VecDeque<PortMessageTask>>,

    /// The UUID of this port.
    message_port_id: MessagePortId,
}

impl MessagePortImpl {
    /// Create a new messageport impl.
    pub fn new(port_id: MessagePortId) -> MessagePortImpl {
        MessagePortImpl {
            state: MessagePortState::Disabled(false),
            entangled_port: None,
            message_buffer: None,
            message_port_id: port_id,
        }
    }

    /// Get the Id.
    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    /// Maybe get the Id of the entangled port.
    pub fn entangled_port_id(&self) -> Option<MessagePortId> {
        self.entangled_port
    }

    /// Entanged this port with another.
    pub fn entangle(&mut self, other_id: MessagePortId) {
        self.entangled_port = Some(other_id);
    }

    /// Is this port enabled?
    pub fn enabled(&self) -> bool {
        matches!(self.state, MessagePortState::Enabled(_))
    }

    /// Mark this port as having been shipped.
    /// <https://html.spec.whatwg.org/multipage/#has-been-shipped>
    pub fn set_has_been_shipped(&mut self) {
        match self.state {
            MessagePortState::Detached => {
                panic!("Messageport set_has_been_shipped called in detached state")
            },
            MessagePortState::Enabled(_) => self.state = MessagePortState::Enabled(true),
            MessagePortState::Disabled(_) => self.state = MessagePortState::Disabled(true),
        }
    }

    /// Handle the completion of the transfer,
    /// this is data received from the constellation.
    pub fn complete_transfer(&mut self, mut tasks: VecDeque<PortMessageTask>) {
        match self.state {
            MessagePortState::Detached => return,
            MessagePortState::Enabled(_) => self.state = MessagePortState::Enabled(false),
            MessagePortState::Disabled(_) => self.state = MessagePortState::Disabled(false),
        }

        // Note: these are the tasks that were buffered while the transfer was ongoing,
        // hence they need to execute first.
        // The global will call `start` if we are enabled,
        // which will add tasks on the event-loop to dispatch incoming messages.
        match self.message_buffer {
            Some(ref mut incoming_buffer) => {
                while let Some(task) = tasks.pop_back() {
                    incoming_buffer.push_front(task);
                }
            },
            None => self.message_buffer = Some(tasks),
        }
    }

    /// A message was received from our entangled port,
    /// returns an optional task to be dispatched.
    pub fn handle_incoming(&mut self, task: PortMessageTask) -> Option<PortMessageTask> {
        let should_dispatch = match self.state {
            MessagePortState::Detached => return None,
            MessagePortState::Enabled(in_transfer) => !in_transfer,
            MessagePortState::Disabled(_) => false,
        };

        if should_dispatch {
            Some(task)
        } else {
            match self.message_buffer {
                Some(ref mut buffer) => {
                    buffer.push_back(task);
                },
                None => {
                    let mut queue = VecDeque::new();
                    queue.push_back(task);
                    self.message_buffer = Some(queue);
                },
            }
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    /// returns an optional queue of tasks that were buffered while the port was disabled.
    pub fn start(&mut self) -> Option<VecDeque<PortMessageTask>> {
        match self.state {
            MessagePortState::Detached => return None,
            MessagePortState::Enabled(_) => {},
            MessagePortState::Disabled(in_transfer) => {
                self.state = MessagePortState::Enabled(in_transfer);
            },
        }
        if let MessagePortState::Enabled(true) = self.state {
            return None;
        }
        self.message_buffer.take()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close(&mut self) {
        // Step 1
        self.state = MessagePortState::Detached;
    }
}

/// A data-holder for serialized data and transferred objects.
/// <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
#[derive(Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct StructuredSerializedData {
    /// Data serialized by SpiderMonkey.
    pub serialized: Vec<u8>,
    /// Serialized in a structured callback,
    pub blobs: Option<HashMap<BlobId, BlobImpl>>,
    /// Serialized point objects.
    pub points: Option<HashMap<DomPointId, DomPoint>>,
    /// Transferred objects.
    pub ports: Option<HashMap<MessagePortId, MessagePortImpl>>,
}

pub(crate) trait BroadcastClone
where
    Self: Sized,
{
    /// The ID type that uniquely identify each value.
    type Id: Eq + std::hash::Hash + Copy;
    /// Clone this value so that it can be reused with a broadcast channel.
    /// Only return None if cloning is impossible.
    fn clone_for_broadcast(&self) -> Option<Self>;
    /// The field from which to clone values.
    fn source(data: &StructuredSerializedData) -> &Option<HashMap<Self::Id, Self>>;
    /// The field into which to place cloned values.
    fn destination(data: &mut StructuredSerializedData) -> &mut Option<HashMap<Self::Id, Self>>;
}

/// All the DOM interfaces that can be serialized.
#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Serializable {
    /// The `Blob` interface.
    Blob,
    /// The `DOMPoint` interface.
    DomPoint,
    /// The `DOMPointReadOnly` interface.
    DomPointReadOnly,
}

impl Serializable {
    fn clone_values(&self) -> fn(&StructuredSerializedData, &mut StructuredSerializedData) {
        match self {
            Serializable::Blob => StructuredSerializedData::clone_all_of_type::<BlobImpl>,
            Serializable::DomPointReadOnly => {
                StructuredSerializedData::clone_all_of_type::<DomPoint>
            },
            Serializable::DomPoint => StructuredSerializedData::clone_all_of_type::<DomPoint>,
        }
    }
}

/// All the DOM interfaces that can be transferred.
#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Transferrable {
    /// The `MessagePort` interface.
    MessagePort,
}

impl StructuredSerializedData {
    fn is_empty(&self, val: Transferrable) -> bool {
        fn is_field_empty<K, V>(field: &Option<HashMap<K, V>>) -> bool {
            field.as_ref().is_some_and(|h| h.is_empty())
        }
        match val {
            Transferrable::MessagePort => is_field_empty(&self.ports),
        }
    }

    /// Clone all values of the same type stored in this StructuredSerializedData
    /// into another instance.
    fn clone_all_of_type<T: BroadcastClone>(&self, cloned: &mut StructuredSerializedData) {
        let existing = T::source(self);
        let Some(existing) = existing else { return };
        let mut clones = HashMap::with_capacity(existing.len());

        for (original_id, obj) in existing.iter() {
            if let Some(clone) = obj.clone_for_broadcast() {
                clones.insert(*original_id, clone);
            }
        }

        *T::destination(cloned) = Some(clones);
    }

    /// Clone the serialized data for use with broadcast-channels.
    pub fn clone_for_broadcast(&self) -> StructuredSerializedData {
        for transferrable in Transferrable::iter() {
            if !self.is_empty(transferrable) {
                // Not panicking only because this is called from the constellation.
                warn!(
                    "Attempt to broadcast structured serialized data including {:?} (should never happen).",
                    transferrable,
                );
            }
        }

        let serialized = self.serialized.clone();

        let mut cloned = StructuredSerializedData {
            serialized,
            ..Default::default()
        };

        for serializable in Serializable::iter() {
            let clone_impl = serializable.clone_values();
            clone_impl(self, &mut cloned);
        }

        cloned
    }
}

/// A task on the <https://html.spec.whatwg.org/multipage/#port-message-queue>
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PortMessageTask {
    /// The origin of this task.
    pub origin: ImmutableOrigin,
    /// A data-holder for serialized data and transferred objects.
    pub data: StructuredSerializedData,
}

/// Messages for communication between the constellation and a global managing ports.
#[derive(Debug, Deserialize, Serialize)]
pub enum MessagePortMsg {
    /// Complete the transfer for a batch of ports.
    CompleteTransfer(HashMap<MessagePortId, VecDeque<PortMessageTask>>),
    /// Complete the transfer of a single port,
    /// whose transfer was pending because it had been requested
    /// while a previous failed transfer was being rolled-back.
    CompletePendingTransfer(MessagePortId, VecDeque<PortMessageTask>),
    /// Remove a port, the entangled one doesn't exists anymore.
    RemoveMessagePort(MessagePortId),
    /// Handle a new port-message-task.
    NewTask(MessagePortId, PortMessageTask),
}

/// Message for communication between the constellation and a global managing broadcast channels.
#[derive(Debug, Deserialize, Serialize)]
pub struct BroadcastMsg {
    /// The origin of this message.
    pub origin: ImmutableOrigin,
    /// The name of the channel.
    pub channel_name: String,
    /// A data-holder for serialized data.
    pub data: StructuredSerializedData,
}

impl Clone for BroadcastMsg {
    fn clone(&self) -> BroadcastMsg {
        BroadcastMsg {
            data: self.data.clone_for_broadcast(),
            origin: self.origin.clone(),
            channel_name: self.channel_name.clone(),
        }
    }
}

/// File-based blob
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct FileBlob {
    #[ignore_malloc_size_of = "Uuid are hard(not really)"]
    id: Uuid,
    #[ignore_malloc_size_of = "PathBuf are hard"]
    name: Option<PathBuf>,
    cache: RefCell<Option<Vec<u8>>>,
    size: u64,
}

impl FileBlob {
    /// Create a new file blob.
    pub fn new(id: Uuid, name: Option<PathBuf>, cache: Option<Vec<u8>>, size: u64) -> FileBlob {
        FileBlob {
            id,
            name,
            cache: RefCell::new(cache),
            size,
        }
    }

    /// Get the size of the file.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Get the cached file data, if any.
    pub fn get_cache(&self) -> Option<Vec<u8>> {
        self.cache.borrow().clone()
    }

    /// Cache data.
    pub fn cache_bytes(&self, bytes: Vec<u8>) {
        *self.cache.borrow_mut() = Some(bytes);
    }

    /// Get the file id.
    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

impl BroadcastClone for BlobImpl {
    type Id = BlobId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.blobs
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.blobs
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        let type_string = self.type_string();

        if let BlobData::Memory(bytes) = self.blob_data() {
            let blob_clone = BlobImpl::new_from_bytes(bytes.clone(), type_string);

            // Note: we insert the blob at the original id,
            // otherwise this will not match the storage key as serialized by SM in `serialized`.
            // The clone has it's own new Id however.
            return Some(blob_clone);
        } else {
            // Not panicking only because this is called from the constellation.
            log::warn!("Serialized blob not in memory format(should never happen).");
        }
        None
    }
}

/// The data backing a DOM Blob.
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct BlobImpl {
    /// UUID of the blob.
    blob_id: BlobId,
    /// Content-type string
    type_string: String,
    /// Blob data-type.
    blob_data: BlobData,
    /// Sliced blobs referring to this one.
    slices: Vec<BlobId>,
}

/// Different backends of Blob
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum BlobData {
    /// File-based blob, whose content lives in the net process
    File(FileBlob),
    /// Memory-based blob, whose content lives in the script process
    Memory(Vec<u8>),
    /// Sliced blob, including parent blob-id and
    /// relative positions of current slicing range,
    /// IMPORTANT: The depth of tree is only two, i.e. the parent Blob must be
    /// either File-based or Memory-based
    Sliced(BlobId, RelativePos),
}

impl BlobImpl {
    /// Construct memory-backed BlobImpl
    pub fn new_from_bytes(bytes: Vec<u8>, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::Memory(bytes);
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Construct file-backed BlobImpl from File ID
    pub fn new_from_file(file_id: Uuid, name: PathBuf, size: u64, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::File(FileBlob {
            id: file_id,
            name: Some(name),
            cache: RefCell::new(None),
            size,
        });
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Construct a BlobImpl from a slice of a parent.
    pub fn new_sliced(rel_pos: RelativePos, parent: BlobId, type_string: String) -> BlobImpl {
        let blob_id = BlobId::new();
        let blob_data = BlobData::Sliced(parent, rel_pos);
        BlobImpl {
            blob_id,
            type_string,
            blob_data,
            slices: vec![],
        }
    }

    /// Get a clone of the blob-id
    pub fn blob_id(&self) -> BlobId {
        self.blob_id
    }

    /// Get a clone of the type-string
    pub fn type_string(&self) -> String {
        self.type_string.clone()
    }

    /// Get a mutable ref to the data
    pub fn blob_data(&self) -> &BlobData {
        &self.blob_data
    }

    /// Get a mutable ref to the data
    pub fn blob_data_mut(&mut self) -> &mut BlobData {
        &mut self.blob_data
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
/// A serializable version of the DOMPoint/DOMPointReadOnly interface.
pub struct DomPoint {
    /// The x coordinate.
    pub x: f64,
    /// The y coordinate.
    pub y: f64,
    /// The z coordinate.
    pub z: f64,
    /// The w coordinate.
    pub w: f64,
}

impl BroadcastClone for DomPoint {
    type Id = DomPointId;

    fn source(
        data: &StructuredSerializedData,
    ) -> &Option<std::collections::HashMap<Self::Id, Self>> {
        &data.points
    }

    fn destination(
        data: &mut StructuredSerializedData,
    ) -> &mut Option<std::collections::HashMap<Self::Id, Self>> {
        &mut data.points
    }

    fn clone_for_broadcast(&self) -> Option<Self> {
        Some(self.clone())
    }
}
