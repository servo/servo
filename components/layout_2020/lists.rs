/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{NodeAndStyleInfo, NodeExt, PseudoElementContentItem};
use crate::replaced::ReplacedContent;
use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
use style::properties::style_structs;
use style::values::computed::Image;

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
    // FIXME: add support for counters and other style types
    match style.list_style_type {
        ListStyleType::None => None,
        ListStyleType::Disc => Some("• "),
        ListStyleType::Circle => Some("◦ "),
        ListStyleType::Square => Some("▪ "),
        ListStyleType::DisclosureOpen => Some("▾ "),
        ListStyleType::DisclosureClosed => Some("‣ "),
    }
}
