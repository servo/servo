/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericSender;
use servo_base::id::PainterId;
use style::values::computed::font::SingleFontFamily;
use webrender_api::units::Au;
use webrender_api::{
    FontInstanceFlags, FontInstanceKey, FontInstancePlatformOptions, FontKey, FontVariation,
};

use crate::{FontDescriptor, FontIdentifier, FontTemplate};

/// Commands that the `FontContext` sends to the `SystemFontService`.
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum SystemFontServiceMessage {
    GetFontTemplates(
        Option<FontDescriptor>,
        SingleFontFamily,
        GenericSender<Vec<FontTemplate>>,
    ),
    GetFontInstance(
        PainterId,
        FontIdentifier,
        Au,
        FontInstanceFlags,
        FontInstancePlatformOptions,
        Vec<FontVariation>,
        GenericSender<FontInstanceKey>,
    ),
    PrefetchFontKeys(PainterId),
    GetFontKey(PainterId, GenericSender<FontKey>),
    GetFontInstanceKey(PainterId, GenericSender<FontInstanceKey>),
    CollectMemoryReport(ReportsChan),
    Exit(GenericSender<()>),
    Ping,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SystemFontServiceProxySender(pub GenericSender<SystemFontServiceMessage>);
