/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use content_security_policy::Violation;
use fonts_traits::SystemFontServiceProxySender;
use net_traits::ResourceThreads;
use net_traits::blob_url_store::TransferableUrlWithBlobClaim;
use net_traits::policy_container::PolicyContainer;
use net_traits::pub_domains::Site;
use net_traits::request::{CredentialsMode, InsecureRequestsPolicy, Referrer};
use paint_api::CrossProcessPaintApi;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::{GenericCallback, GenericReceiver, GenericSender};
use servo_base::id::{
    BrowsingContextId, MessagePortId, PipelineId, PipelineNamespaceRequest, WebViewId,
};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct SharedWorkerCreationData {
    pub init: WorkerGlobalScopeInit,
    pub webview_id: WebViewId,
    pub browsing_context_id: BrowsingContextId,
    pub top_level_site: Option<Site>,
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
pub enum SharedWorkerManagerMessage {
    Connect(Box<SharedWorkerConnect>),
    UpdateOwners {
        fully_active: HashMap<PipelineId, bool>,
        has_loading_document: bool,
    },
    RemoveOwner(PipelineId),
    SetNetworkOnlineState(bool),
    Exit,
}

#[derive(Deserialize, Serialize)]
pub struct SharedWorkerManagerSenders {
    pub resource_threads: ResourceThreads,
    pub paint_api: CrossProcessPaintApi,
    pub system_font_service_sender: SystemFontServiceProxySender,
    pub receiver: GenericReceiver<SharedWorkerManagerMessage>,
    pub namespace_request_sender: GenericSender<PipelineNamespaceRequest>,
    pub lifeline_sender: Option<GenericSender<()>>,
}
