/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
use style::properties::style_structs;
use style::values::computed::Image;

use crate::context::LayoutContext;
use crate::dom::NodeExt;
use crate::dom_traversal::{NodeAndStyleInfo, PseudoElementContentItem};
use crate::replaced::ReplacedContent;

/// https://drafts.csswg.org/css-lists/#content-property
pub(crate) fn make_marker<'dom, Node>(
    context: &LayoutContext,
    info: &NodeAndStyleInfo<Node>,
) -> Option<Vec<PseudoElementContentItem>>
where
    Node: NodeExt<'dom>,
{
    let style = info.style.get_list();

    // https://drafts.csswg.org/css-lists/#marker-image
    let marker_image = || match &style.list_style_image {
        Image::Url(url) => Some(vec![
            PseudoElementContentItem::Replaced(ReplacedContent::from_image_url(
                info.node, context, url,
            )?),
            PseudoElementContentItem::Text(" ".into()),
        ]),
        // XXX: Non-None image types unimplemented.
        Image::ImageSet(..) |
        Image::Rect(..) |
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

/// https://drafts.csswg.org/css-lists/#marker-string
fn marker_string(style: &style_structs::List) -> Option<&'static str> {
    match style.list_style_type {
        ListStyleType::None => None,
        ListStyleType::Disc => Some("• "),
        ListStyleType::Circle => Some("◦ "),
        ListStyleType::Square => Some("▪ "),
        ListStyleType::DisclosureOpen => Some("▾ "),
        ListStyleType::DisclosureClosed => Some("‣ "),
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
