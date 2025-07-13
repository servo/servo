/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use serde::{Deserialize, Serialize};
use webrender_api::Epoch;

/// Largest Contentful Paint Candidate, include image and block-level element containing text
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct LCPCandidate {
    /// The identity of the element.
    pub id: LCPCandidateID,
    /// The size of the visual area
    pub area: usize,
    /// The epoch when the image is drawn.
    pub epoch: Epoch,
}

impl LCPCandidate {
    pub fn new(id: LCPCandidateID, area: usize, epoch: Epoch) -> Self {
        Self { id, area, epoch }
    }
}

impl Default for LCPCandidate {
    fn default() -> Self {
        Self {
            id: LCPCandidateID::INVALID,
            area: 0,
            epoch: Epoch::invalid(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LCPCandidateID(pub usize);

impl LCPCandidateID {
    pub const INVALID: LCPCandidateID = LCPCandidateID(0);
}

impl Default for LCPCandidateID {
    fn default() -> Self {
        Self::INVALID
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LargestContentfulPaint {
    pub id: LCPCandidateID,
    pub area: usize,
    pub paint_time: CrossProcessInstant,
}

impl LargestContentfulPaint {
    pub fn from(lcp_candidate: LCPCandidate, paint_time: CrossProcessInstant) -> Self {
        Self {
            id: lcp_candidate.id,
            area: lcp_candidate.area,
            paint_time,
        }
    }
}
