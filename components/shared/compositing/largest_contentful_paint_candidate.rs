/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use serde::{Deserialize, Serialize};
use webrender_api::Epoch;

#[derive(Default, Deserialize, Serialize)]
pub struct LCPCandidates {
    pub image_candidate: Option<ImageCandidate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LCPCandidateTag(usize);

impl LCPCandidateTag {
    pub const INVALID: LCPCandidateTag = LCPCandidateTag(0);
}

/// The LCP candidate Image.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageCandidate {
    /// The identity of the element.
    tag: LCPCandidateTag,
    /// The size of the visual area
    size: usize,
    epoch: Epoch,
    pub paint_time: Option<CrossProcessInstant>,
}

impl ImageCandidate {
    pub fn new(tag: usize, size: usize, epoch: Epoch) -> Self {
        Self {
            tag: LCPCandidateTag(tag),
            size,
            epoch,
            paint_time: None,
        }
    }

    pub fn tag(&self) -> LCPCandidateTag {
        self.tag
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn paint_time(&self) -> CrossProcessInstant {
        self.paint_time.unwrap_or(CrossProcessInstant::now())
    }

    pub fn epoch(&self) -> Epoch {
        self.epoch
    }
}

impl Default for ImageCandidate {
    fn default() -> Self {
        Self {
            tag: LCPCandidateTag::INVALID,
            size: 0,
            epoch: Epoch::invalid(),
            paint_time: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LargestContentfulPaint {
    pub tag: LCPCandidateTag,
    pub size: usize,
    pub paint_time: CrossProcessInstant,
}

impl From<&ImageCandidate> for LargestContentfulPaint {
    fn from(candidate: &ImageCandidate) -> Self {
        Self {
            tag: candidate.tag(),
            size: candidate.size(),
            paint_time: candidate.paint_time(),
        }
    }
}
