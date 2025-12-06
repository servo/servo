/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(super) use xml::attribute::OwnedAttribute as Attribute;
use xml::reader::{Error as XmlError, Result as XmlResult, XmlEvent};

pub(super) enum Node {
    Element {
        name: xml::name::OwnedName,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        children: Vec<Node>,
    },
    Text(String),
}

pub(super) fn parse(bytes: &[u8]) -> XmlResult<Vec<Node>> {
    let mut stack = Vec::new();
    let mut nodes = Vec::new();
    let mut reader = xml::EventReader::new(bytes);
    loop {
        match reader.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                stack.push((name, attributes, nodes));
                nodes = Vec::new();
            },
            XmlEvent::EndElement { .. } => {
                let (name, attributes, mut parent_nodes) = stack
                    .pop()
                    .ok_or(XmlError::from((&reader, "Found unexpected closing tag")))?;
                parent_nodes.push(Node::Element {
                    name,
                    attributes,
                    children: nodes,
                });
                nodes = parent_nodes;
            },
            XmlEvent::CData(characters) |
            XmlEvent::Characters(characters) |
            XmlEvent::Whitespace(characters) => {
                if let Some(Node::Text(text)) = nodes.last_mut() {
                    text.push_str(&characters)
                } else {
                    nodes.push(Node::Text(characters))
                }
            },
            XmlEvent::EndDocument => break,
            XmlEvent::StartDocument { .. } |
            XmlEvent::ProcessingInstruction { .. } |
            XmlEvent::Comment(..) => {},
        }
    }

    if !stack.is_empty() {
        return Err(XmlError::from((
            &reader,
            "Found unclosed tags at end of file",
        )));
    }
    Ok(nodes)
}
