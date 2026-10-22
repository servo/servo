/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::debug;

use crate::model::document::SvgTree;
use crate::model::element::SvgNode;

pub fn render_svg_tree(tree: &SvgTree) {
    render_node(&tree.root);
}

fn render_node(node: &SvgNode) {
    debug!(target: "svg", "render_node: {:?}", node.tag);
    for child in &node.children {
        render_node(child);
    }
}
