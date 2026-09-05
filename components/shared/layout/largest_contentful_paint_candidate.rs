/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for the largest-contentful-paint candidate.

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::id::LCPCandidateID;
use servo_url::ServoUrl;
use style::dom::OpaqueNode;

/// A largest-contentful-paint candidate
///
/// <https://www.w3.org/TR/largest-contentful-paint/#largest-contentful-paint-candidate>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct LCPCandidate {
    /// A unique identifier for this candidate.
    pub id: LCPCandidateID,
    /// The size of the visual area.
    pub area: usize,
    /// The candidate's request URL.
    pub url: Option<ServoUrl>,
    /// The DOM node of the candidate's element, if any.
    pub node: Option<OpaqueNode>,
}

impl LCPCandidate {
    pub fn new(
        id: LCPCandidateID,
        area: usize,
        url: Option<ServoUrl>,
        node: Option<OpaqueNode>,
    ) -> Self {
        Self {
            id,
            area,
            url,
            node,
        }
    }
}
