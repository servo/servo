/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::warn;
use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
use style::properties::style_structs;
use style::values::computed::Image;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{NodeAndStyleInfo, PseudoElementContentItem};
use crate::replaced::ReplacedContent;

/// <https://drafts.csswg.org/css-lists/#content-property>
pub(crate) fn make_marker<'dom, Node>(
    context: &LayoutContext,
    info: &NodeAndStyleInfo<Node>,
) -> Option<Vec<PseudoElementContentItem>>
where
    Node: NodeExt<'dom>,
{
    let style = info.style.get_list();
    let node = match info.node {
        Some(node) => node,
        None => {
            warn!("Tried to make a marker for an anonymous node!");
            return None;
        },
    };

    // https://drafts.csswg.org/css-lists/#marker-image
    let marker_image = || match &style.list_style_image {
        Image::Url(url) => Some(vec![
            PseudoElementContentItem::Replaced(ReplacedContent::from_image_url(
                node, context, url,
            )?),
            PseudoElementContentItem::Text(" ".into()),
        ]),
        // XXX: Non-None image types unimplemented.
        Image::ImageSet(..) |
        Image::Gradient(..) |
        Image::CrossFade(..) |
        Image::PaintWorklet(..) |
        Image::None => None,
    };
    marker_image().or_else(|| {
        Some(vec![PseudoElementContentItem::Text(
            marker_string(style)?.into(),
        )])
    })
}

/// <https://drafts.csswg.org/css-lists/#marker-string>
fn marker_string(style: &style_structs::List) -> Option<&'static str> {
    match style.list_style_type {
        ListStyleType::None => None,
        // TODO: Using non-breaking space here is a bit of a hack to give a bit of margin to outside
        // markers, but really we should be setting `white-space: pre` on them instead.
        // See https://github.com/w3c/csswg-drafts/issues/4891.
        ListStyleType::Disc => Some("•\u{00a0}"),
        ListStyleType::Circle => Some("◦\u{00a0}"),
        ListStyleType::Square => Some("▪\u{00a0}"),
        ListStyleType::DisclosureOpen => Some("▾\u{00a0}"),
        ListStyleType::DisclosureClosed => Some("‣\u{00a0}"),
        ListStyleType::Decimal |
        ListStyleType::LowerAlpha |
        ListStyleType::UpperAlpha |
        ListStyleType::ArabicIndic |
        ListStyleType::Bengali |
        ListStyleType::Cambodian |
        ListStyleType::CjkDecimal |
        ListStyleType::Devanagari |
        ListStyleType::Gujarati |
        ListStyleType::Gurmukhi |
        ListStyleType::Kannada |
        ListStyleType::Khmer |
        ListStyleType::Lao |
        ListStyleType::Malayalam |
        ListStyleType::Mongolian |
        ListStyleType::Myanmar |
        ListStyleType::Oriya |
        ListStyleType::Persian |
        ListStyleType::Telugu |
        ListStyleType::Thai |
        ListStyleType::Tibetan |
        ListStyleType::CjkEarthlyBranch |
        ListStyleType::CjkHeavenlyStem |
        ListStyleType::LowerGreek |
        ListStyleType::Hiragana |
        ListStyleType::HiraganaIroha |
        ListStyleType::Katakana |
        ListStyleType::KatakanaIroha => {
            // TODO: Implement support for counters.
            None
        },
    }
}
