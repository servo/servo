/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::vec::Vec;

use msg::constellation_msg::PipelineId;
use serde::{Deserialize, Serialize};

use crate::wgc::id::DeviceId;

#[derive(Eq, Hash, PartialEq)]
pub(crate) struct DeviceScope {
    pub device_id: DeviceId,
    pub pipeline_id: PipelineId,
    /// <https://www.w3.org/TR/webgpu/#dom-gpudevice-errorscopestack-slot>
    pub error_scope_stack: Vec<ErrorScope>,
    // TODO:
    // Queue for this device (to remove transmutes)
    // queue_id: QueueId,
    // Poller for this device
    // poller: Poller,
}

impl DeviceScope {
    pub fn new(device_id: DeviceId, pipeline_id: PipelineId) -> Self {
        Self {
            device_id,
            pipeline_id,
            error_scope_stack: Vec::new(),
        }
    }
}

/// <https://www.w3.org/TR/webgpu/#gpu-error-scope>
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ErrorScope {
    // we only store first error
    pub errors: Option<Error>,
    pub filter: ErrorFilter,
}

impl ErrorScope {
    pub fn new(filter: ErrorFilter) -> Self {
        Self {
            filter,
            errors: None,
        }
    }
}

/// <https://www.w3.org/TR/webgpu/#enumdef-gpuerrorfilter>
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ErrorFilter {
    Validation,
    OutOfMemory,
    Internal,
}

/// <https://www.w3.org/TR/webgpu/#gpuerror>
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Error {
    Validation(String),
    OutOfMemory(String),
    Internal(String),
}

impl Error {
    pub fn filter(&self) -> ErrorFilter {
        match self {
            Error::Validation(_) => ErrorFilter::Validation,
            Error::OutOfMemory(_) => ErrorFilter::OutOfMemory,
            Error::Internal(_) => ErrorFilter::Internal,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Error::Validation(m) => m,
            Error::OutOfMemory(m) => m,
            Error::Internal(m) => m,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PopError {
    Lost,
    Empty,
}
