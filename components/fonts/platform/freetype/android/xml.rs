/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(super) use xml::attribute::OwnedAttribute as Attribute;
use xml::reader::XmlEvent::*;

pub(super) enum Node {
    Element {
        name: xml::name::OwnedName,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        children: Vec<Node>,
    },
    Text(String),
}

pub(super) fn parse(bytes: &[u8]) -> xml::reader::Result<Vec<Node>> {
    let mut stack = Vec::new();
    let mut nodes = Vec::new();
    for result in xml::EventReader::new(&*bytes) {
        match result? {
            StartElement {
                name, attributes, ..
            } => {
                stack.push((name, attributes, nodes));
                nodes = Vec::new();
            },
            EndElement { .. } => {
                let (name, attributes, mut parent_nodes) = stack.pop().unwrap();
                parent_nodes.push(Node::Element {
                    name,
                    attributes,
                    children: nodes,
                });
                nodes = parent_nodes;
            },
            CData(s) | Characters(s) | Whitespace(s) => {
                if let Some(Node::Text(text)) = nodes.last_mut() {
                    text.push_str(&s)
                } else {
                    nodes.push(Node::Text(s))
                }
            },
            StartDocument { .. } | EndDocument | ProcessingInstruction { .. } | Comment(..) => {},
        }
    }
    assert!(stack.is_empty());
    Ok(nodes)
}
