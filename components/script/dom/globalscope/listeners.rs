/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, VecDeque};

use net_traits::filemanager_thread::{FileManagerResult, ReadFileProgress};
use rustc_hash::{FxBuildHasher, FxHashMap};
use script_bindings::refcounted::Trusted;
use script_bindings::root::Dom;
use servo_base::id::{BroadcastChannelRouterId, MessagePortId, MessagePortRouterId};
use servo_constellation_traits::{
    BlobImpl, BroadcastChannelMsg, MessagePortImpl, MessagePortMsg, ScriptToConstellationMessage,
};
use uuid::Uuid;

use crate::dom::GlobalScope;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::globalscope::broadcastchannel::BroadcastChannel;
use crate::dom::messageport::MessagePort;
use crate::dom::promise::RootedPromise;
use crate::dom::readablestream::ReadableStream;
use crate::dom::transformstream::CrossRealmTransform;
use crate::realms::enter_auto_realm;
use crate::tasks::task_source::SendableTaskSource;
use crate::test::TrustedPromise;

/// A wrapper for glue-code between the ipc router and the event-loop.
pub(super) struct MessageListener {
    pub(super) task_source: SendableTaskSource,
    pub(super) context: Trusted<GlobalScope>,
}

/// A wrapper for broadcasts coming in over IPC, and the event-loop.
pub(super) struct BroadcastListener {
    pub(super) task_source: SendableTaskSource,
    pub(super) context: Trusted<GlobalScope>,
}

pub(super) type FileListenerCallback =
    Box<dyn Fn(&mut js::context::JSContext, &RootedPromise, Fallible<Vec<u8>>) + Send>;

/// A wrapper for the handling of file data received by the ipc router
pub(super) struct FileListener {
    /// State should progress as either of:
    /// - Some(Empty) => Some(Receiving) => None
    /// - Some(Empty) => None
    pub(super) state: Option<FileListenerState>,
    pub(super) task_source: SendableTaskSource,
}

pub(super) enum FileListenerTarget {
    Promise(TrustedPromise, FileListenerCallback),
    Stream(Trusted<ReadableStream>),
}

pub(super) enum FileListenerState {
    Empty(FileListenerTarget),
    Receiving(Vec<u8>, FileListenerTarget),
}

#[derive(JSTraceable, MallocSizeOf)]
/// A holder of a weak reference for a DOM blob or file.
pub(crate) enum BlobTracker {
    /// A weak ref to a DOM file.
    File(WeakRef<File>),
    /// A weak ref to a DOM blob.
    Blob(WeakRef<Blob>),
}

#[derive(JSTraceable, MallocSizeOf)]
/// The info pertaining to a blob managed by this global.
pub(crate) struct BlobInfo {
    /// The weak ref to the corresponding DOM object.
    pub(super) tracker: BlobTracker,
    /// The data and logic backing the DOM object.
    #[no_trace]
    pub(super) blob_impl: BlobImpl,
    /// Whether this blob has an outstanding URL,
    /// <https://w3c.github.io/FileAPI/#url>.
    pub(super) has_url: bool,
}

/// The result of looking-up the data for a Blob,
/// containing either the in-memory bytes,
/// or the file-id.
pub(super) enum BlobResult {
    Bytes(Vec<u8>),
    File(Uuid, usize),
}

/// Data representing a message-port managed by this global.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ManagedMessagePort {
    /// The DOM port.
    pub(super) dom_port: Dom<MessagePort>,
    /// The logic and data backing the DOM port.
    /// The option is needed to take out the port-impl
    /// as part of its transferring steps,
    /// without having to worry about rooting the dom-port.
    #[no_trace]
    pub(super) port_impl: Option<MessagePortImpl>,
    /// We keep ports pending when they are first transfer-received,
    /// and only add them, and ask the constellation to complete the transfer,
    /// in a subsequent task if the port hasn't been re-transfered.
    pub(super) pending: bool,
    /// Whether the port has been closed by script in this global,
    /// so it can be removed.
    pub(super) explicitly_closed: bool,
    /// The handler for `message` or `messageerror` used in the cross realm transform,
    /// if any was setup with this port.
    pub(super) cross_realm_transform: Option<CrossRealmTransform>,
}

/// State representing whether this global is currently managing broadcast channels.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(super) enum BroadcastChannelState {
    /// The broadcast-channel router id for this global, and a queue of managed channels.
    /// Step 9, "sort destinations"
    /// of <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// requires keeping track of creation order, hence the queue.
    Managed(
        #[no_trace] BroadcastChannelRouterId,
        /// The map of channel-name to queue of channels, in order of creation.
        HashMap<DOMString, VecDeque<Dom<BroadcastChannel>>>,
    ),
    /// This global is not managing any broadcast channels at this time.
    UnManaged,
}

/// State representing whether this global is currently managing messageports.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(super) enum MessagePortState {
    /// The message-port router id for this global, and a map of managed ports.
    Managed(
        #[no_trace] MessagePortRouterId,
        HashMapTracedValues<MessagePortId, ManagedMessagePort, FxBuildHasher>,
    ),
    /// This global is not managing any ports at this time.
    UnManaged,
}

impl BroadcastListener {
    /// Handle a broadcast coming in over IPC,
    /// by queueing the appropriate task on the relevant event-loop.
    pub(super) fn handle(&self, event: BroadcastChannelMsg) {
        let context = self.context.clone();

        // Note: strictly speaking we should just queue the message event tasks,
        // not queue a task that then queues more tasks.
        // This however seems to be hard to avoid in the light of the IPC.
        // One can imagine queueing tasks directly,
        // for channels that would be in the same script-thread.
        self.task_source
            .queue(task!(broadcast_message_event: move || {
                let global = context.root();
                // Step 10 of https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage,
                // For each BroadcastChannel object destination in destinations, queue a task.
                global.broadcast_message_event(event, None);
            }));
    }
}

impl MessageListener {
    /// A new message came in, handle it via a task enqueued on the event-loop.
    /// A task is required, since we are using a trusted globalscope,
    /// and we can only access the root from the event-loop.
    pub(super) fn notify(&self, msg: MessagePortMsg) {
        match msg {
            MessagePortMsg::CompleteTransfer(ports) => {
                let context = self.context.clone();
                self.task_source.queue(
                    task!(process_complete_transfer: move |cx| {
                        let global = context.root();

                        let router_id = match global.port_router_id() {
                            Some(router_id) => router_id,
                            None => {
                                // If not managing any ports, no transfer can succeed,
                                // so just send back everything.
                                let _ = global.script_to_constellation_chan().send(
                                    ScriptToConstellationMessage::MessagePortTransferResult(None, vec![], ports),
                                );
                                return;
                            }
                        };

                        let mut succeeded = vec![];
                        let mut failed = FxHashMap::default();

                        for (id, info) in ports.into_iter() {
                            if global.is_managing_port(&id) {
                                succeeded.push(id);
                                global.complete_port_transfer(
                                    cx,
                                    id,
                                    info.port_message_queue,
                                    info.disentangled,
                                );
                            } else {
                                failed.insert(id, info);
                            }
                        }
                        let _ = global.script_to_constellation_chan().send(
                            ScriptToConstellationMessage::MessagePortTransferResult(Some(router_id), succeeded, failed),
                        );
                    })
                );
            },
            MessagePortMsg::CompletePendingTransfer(port_id, info) => {
                let context = self.context.clone();
                self.task_source.queue(task!(complete_pending: move |cx| {
                    let global = context.root();
                    global.complete_port_transfer(cx, port_id, info.port_message_queue, info.disentangled);
                }));
            },
            MessagePortMsg::CompleteDisentanglement(port_id) => {
                let context = self.context.clone();
                self.task_source
                    .queue(task!(try_complete_disentanglement: move |cx| {
                        let global = context.root();
                        global.try_complete_disentanglement(cx, port_id);
                    }));
            },
            MessagePortMsg::NewTask(port_id, task) => {
                let context = self.context.clone();
                self.task_source.queue(task!(process_new_task: move |cx| {
                    let global = context.root();
                    global.route_task_to_port(cx, port_id, task);
                }));
            },
        }
    }
}

/// Callback used to enqueue file chunks to streams as part of FileListener.
fn stream_handle_incoming(
    cx: &mut js::context::JSContext,
    stream: &ReadableStream,
    bytes: Fallible<Vec<u8>>,
) {
    match bytes {
        Ok(b) => {
            stream.enqueue_native(cx, b);
        },
        Err(e) => {
            stream.error_native(cx, e);
        },
    }
}

/// Callback used to close streams as part of FileListener.
fn stream_handle_eof(cx: &mut js::context::JSContext, stream: &ReadableStream) {
    stream.controller_close_native(cx);
}

impl FileListener {
    pub(super) fn handle(&mut self, msg: FileManagerResult<ReadFileProgress>) {
        match msg {
            Ok(ReadFileProgress::Meta(blob_buf)) => match self.state.take() {
                Some(FileListenerState::Empty(target)) => {
                    let bytes = if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move |cx| {
                            let stream = trusted.root();
                            stream_handle_incoming(cx, &stream, Ok(blob_buf.bytes));
                        });
                        self.task_source.queue(task);

                        Vec::with_capacity(0)
                    } else {
                        blob_buf.bytes
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Meta msg."
                ),
            },
            Ok(ReadFileProgress::Partial(mut bytes_in)) => match self.state.take() {
                Some(FileListenerState::Receiving(mut bytes, target)) => {
                    if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move |cx| {
                            let stream = trusted.root();
                            stream_handle_incoming(cx, &stream, Ok(bytes_in));
                        });

                        self.task_source.queue(task);
                    } else {
                        bytes.append(&mut bytes_in);
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Partial msg."
                ),
            },
            Ok(ReadFileProgress::EOF) => match self.state.take() {
                Some(FileListenerState::Receiving(bytes, target)) => match target {
                    FileListenerTarget::Promise(trusted_promise, callback) => {
                        let task = task!(resolve_promise: move |cx| {
                            let promise = trusted_promise.root(cx);
                            let mut realm = enter_auto_realm(cx, &*promise.global());
                            callback(&mut realm, &promise, Ok(bytes));
                        });

                        self.task_source.queue(task);
                    },
                    FileListenerTarget::Stream(trusted_stream) => {
                        let task = task!(enqueue_stream_chunk: move |cx| {
                            let stream = trusted_stream.root();
                            stream_handle_eof(cx, &stream);
                        });

                        self.task_source.queue(task);
                    },
                },
                _ => {
                    panic!("Unexpected FileListenerState when receiving ReadFileProgress::EOF msg.")
                },
            },
            Err(_) => match self.state.take() {
                Some(FileListenerState::Receiving(_, target)) |
                Some(FileListenerState::Empty(target)) => {
                    let error = Err(Error::Network(None));

                    match target {
                        FileListenerTarget::Promise(trusted_promise, callback) => {
                            self.task_source.queue(task!(reject_promise: move |cx| {
                                let promise = trusted_promise.root(cx);
                                let mut realm = enter_auto_realm(cx, &*promise.global());
                                callback(&mut realm, &promise, error);
                            }));
                        },
                        FileListenerTarget::Stream(trusted_stream) => {
                            self.task_source.queue(task!(error_stream: move |cx| {
                                let stream = trusted_stream.root();
                                stream_handle_incoming(cx, &stream, error);
                            }));
                        },
                    }
                },
                _ => panic!("Unexpected FileListenerState when receiving Err msg."),
            },
        }
    }
}
