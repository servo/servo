/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use base::cross_process_instant::CrossProcessInstant;
use serde::{Deserialize, Serialize};
use webrender_api::units::LayoutRect;
use webrender_api::{Epoch, ImageKey};

pub const IMAGE_ENTROPY_THEAHOLD: f64 = 0.05;

/// For images, if they have large visual area and very small number of bytes,
/// they are typically low-content background and the like, and are not considered
/// as largest-contentful-paint candidate. So we need record the file size of the
/// image.
static IMAGE_RAW_SIZE: LazyLock<Mutex<HashMap<ImageKey, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn record_image_raw_size(key: ImageKey, raw_size: usize) {
    IMAGE_RAW_SIZE
        .lock()
        .unwrap()
        .entry(key)
        .or_insert(raw_size);
}

pub fn get_image_raw_size(key: &ImageKey) -> usize {
    IMAGE_RAW_SIZE
        .lock()
        .unwrap()
        .get(key)
        .copied()
        .unwrap_or(0)
}

pub fn reset_all_image_raw_size() {
    IMAGE_RAW_SIZE.lock().unwrap().clear();
}

#[derive(Default, Deserialize, Serialize)]
pub struct LCPCandidateRecord {
    pub image_records: Vec<ImageRecord>,
    pub text_records: HashMap<LCPRecordTag, TextRecord>,
}

impl LCPCandidateRecord {
    pub fn new() -> Self {
        Self {
            image_records: Vec::new(),
            text_records: HashMap::new(),
        }
    }

    pub fn record_image(
        &mut self,
        tag: usize,
        rect: &LayoutRect,
        image_key: &ImageKey,
        epoch: Epoch,
    ) {
        let size = rect.area() as usize;
        if size == 0 {
            return;
        }

        let raw_size = get_image_raw_size(image_key);
        self.image_records.push(ImageRecord {
            tag: LCPRecordTag(tag),
            size,
            raw_size,
            epoch,
            paint_time: None,
        });
    }

    pub fn record_text(
        &mut self,
        tag: usize,
        block_parent: usize,
        rect: &LayoutRect,
        epoch: Epoch,
    ) {
        if rect.area() == 0.0 {
            return;
        }

        let text_block_parent = LCPRecordTag(block_parent);
        // Because a block node may contains many text node, we need union their visual area.
        self.text_records
            .entry(text_block_parent)
            .and_modify(|record| {
                if epoch == record.epoch {
                    let union_rect = record.rect.union(rect);
                    record.rect = union_rect;
                }
            })
            .or_insert(TextRecord {
                tag: LCPRecordTag(tag),
                text_block_parent,
                rect: *rect,
                epoch,
                paint_time: None,
            });
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LCPRecordTag(usize);

impl LCPRecordTag {
    pub const INVALID: LCPRecordTag = LCPRecordTag(0);
}

/// The LCP candidate Image.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageRecord {
    /// The identity of the element.
    tag: LCPRecordTag,
    /// The size of the visual area
    size: usize,
    /// The file size of the image in bytes.
    raw_size: usize,
    epoch: Epoch,
    pub paint_time: Option<CrossProcessInstant>,
}

impl ImageRecord {
    pub fn tag(&self) -> LCPRecordTag {
        self.tag
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Used to judge if the image is low-content.
    pub fn image_entropy(&self) -> f64 {
        (self.raw_size as f64 * 8.0) / (self.size() as f64)
    }

    pub fn paint_time(&self) -> CrossProcessInstant {
        self.paint_time.unwrap_or(CrossProcessInstant::now())
    }

    pub fn epoch(&self) -> Epoch {
        self.epoch
    }
}

impl Default for ImageRecord {
    fn default() -> Self {
        Self {
            tag: LCPRecordTag::INVALID,
            size: 0,
            raw_size: 0,
            epoch: Epoch::invalid(),
            paint_time: None,
        }
    }
}

/// The LCP candidate text. For text LCP, we record the block-level elements containing
/// text nodes or other inline-level text element children.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextRecord {
    /// The tag of text element
    tag: LCPRecordTag,
    /// The block parent contains current text.
    text_block_parent: LCPRecordTag,
    /// The LayoutRect of the text node. It will be union.
    rect: LayoutRect,
    epoch: Epoch,
    pub paint_time: Option<CrossProcessInstant>,
}

impl TextRecord {
    pub fn parent_tag(&self) -> LCPRecordTag {
        self.text_block_parent
    }

    pub fn size(&self) -> usize {
        self.rect.area() as usize
    }

    pub fn paint_time(&self) -> CrossProcessInstant {
        self.paint_time.unwrap_or(CrossProcessInstant::now())
    }

    pub fn epoch(&self) -> Epoch {
        self.epoch
    }
}

impl Default for TextRecord {
    fn default() -> Self {
        Self {
            tag: LCPRecordTag::INVALID,
            text_block_parent: LCPRecordTag::INVALID,
            rect: LayoutRect::zero(),
            epoch: Epoch::invalid(),
            paint_time: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LargestContentfulPaint {
    pub tag: LCPRecordTag,
    pub size: usize,
    pub paint_time: CrossProcessInstant,
}

impl From<&ImageRecord> for LargestContentfulPaint {
    fn from(record: &ImageRecord) -> Self {
        Self {
            tag: record.tag(),
            size: record.size(),
            paint_time: record.paint_time(),
        }
    }
}

impl From<&TextRecord> for LargestContentfulPaint {
    fn from(record: &TextRecord) -> Self {
        Self {
            tag: record.parent_tag(),
            size: record.size(),
            paint_time: record.paint_time(),
        }
    }
}
