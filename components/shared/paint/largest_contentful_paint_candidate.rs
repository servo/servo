/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for Largest Contentful Paint Candidate and Largest Contentful Paint.

use base::cross_process_instant::CrossProcessInstant;
use serde::{Deserialize, Serialize};

/// Largest Contentful Paint Candidate, include image and block-level element containing text
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct LCPCandidate {
    /// The identity of the element.
    pub id: LCPCandidateID,
    /// The size of the visual area
    pub area: usize,
}

impl LCPCandidate {
    pub fn new(id: LCPCandidateID, area: usize) -> Self {
        Self { id, area }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LCPCandidateID(pub usize);

#[derive(Clone, Copy, Debug)]
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
