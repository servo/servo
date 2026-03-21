/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definitions for Largest Contentful Paint Candidate and Largest Contentful Paint.

use base::cross_process_instant::CrossProcessInstant;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

/// Largest Contentful Paint Candidate, include image and block-level element containing text
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LCPCandidate {
    /// The identity of the element.
    pub id: LCPCandidateID,
    /// The size of the visual area
    pub area: usize,
    /// The candidate's request URL
    pub url: Option<ServoUrl>,
}

impl LCPCandidate {
    pub fn new(id: LCPCandidateID, area: usize, url: Option<ServoUrl>) -> Self {
        Self { id, area, url }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct LCPCandidateID(pub usize);

#[derive(Clone, Debug)]
pub struct LargestContentfulPaint {
    pub id: LCPCandidateID,
    pub area: usize,
    pub paint_time: CrossProcessInstant,
    pub url: Option<ServoUrl>,
}

impl LargestContentfulPaint {
    pub fn from(lcp_candidate: LCPCandidate, paint_time: CrossProcessInstant) -> Self {
        Self {
            id: lcp_candidate.id,
            area: lcp_candidate.area,
            paint_time,
            url: lcp_candidate.url,
        }
    }
}
