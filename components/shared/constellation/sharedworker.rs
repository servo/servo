/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use content_security_policy::Violation;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::blob_url_store::TransferableUrlWithBlobClaim;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{CredentialsMode, InsecureRequestsPolicy, Referrer};
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;
use servo_base::id::{BrowsingContextId, MessagePortId, ScriptEventLoopId, WebViewId, WorkerId};

use crate::{MessagePortImpl, WorkerGlobalScopeInit};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SharedWorkerType {
    Classic,
    Module,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SharedWorkerOptions {
    pub name: String,
    pub worker_type: SharedWorkerType,
    pub credentials: CredentialsMode,
    pub extended_lifetime: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct SharedWorkerSource {
    pub worker_id: WorkerId,
    pub global_id: WorkerId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SharedWorkerCreationData {
    pub init: WorkerGlobalScopeInit,
    pub webview_id: WebViewId,
    pub browsing_context_id: BrowsingContextId,
    pub options: SharedWorkerOptions,
    pub url: TransferableUrlWithBlobClaim,
    pub referrer: Referrer,
    pub policy_container: Arc<PolicyContainer>,
    pub insecure_requests_policy: InsecureRequestsPolicy,
    pub has_trustworthy_ancestor_origin: bool,
    pub is_nested_browsing_context: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SharedWorkerConnect {
    pub connection_id: MessagePortId,
    pub creation: SharedWorkerCreationData,
    pub port: MessagePortImpl,
    pub reply: GenericCallback<SharedWorkerClientMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SharedWorkerClientMessage {
    Connected { enable_port: bool },
    Error,
    CspViolations(Vec<Violation>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SharedWorkerStart {
    pub creation: SharedWorkerCreationData,
    pub event_loop_id: ScriptEventLoopId,
    pub events: GenericCallback<SharedWorkerHostEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SharedWorkerCommand {
    Connect {
        connection_id: MessagePortId,
        port: MessagePortImpl,
    },
    Terminate,
    SetSuspended(bool),
    SetNetworkOnlineState(bool),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SharedWorkerHostEvent {
    Started(GenericCallback<SharedWorkerCommand>),
    ScopeCreated { secure_context: bool },
    ScriptLoaded,
    ScriptLoadFailed,
    PortReceived(MessagePortId),
    CspViolations(Vec<Violation>),
    Closing,
    Exited,
}
