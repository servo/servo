/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JSRef, OptionalRootable, Rootable, RootedReference};
use dom::element::AttributeHandlers;
use dom::htmlheadelement::HTMLHeadElement;
use dom::node::{Node, NodeHelpers};
use util::opts;
use util::resource_files::resources_dir_path;
use std::borrow::ToOwned;
use std::fs::read_dir;
use std::path::PathBuf;


pub fn load_script(head: JSRef<HTMLHeadElement>) {
    if let Some(ref path_str) = opts::get().userscripts {
        let node: &JSRef<Node> = NodeCast::from_borrowed_ref(&head);
        let first_child = node.GetFirstChild().root();
        let doc = node.owner_doc().root();
        let doc = doc.r();

        let path = if &**path_str == "" {
            let mut p = resources_dir_path();
            p.push("user-agent-js");
            p
        } else {
            PathBuf::from(path_str)
        };

        let mut files = read_dir(&path).ok().expect("Bad path passed to --userscripts")
                                       .filter_map(|e| e.ok())
                                       .map(|e| e.path()).collect::<Vec<_>>();

        files.sort();

        for file in files {
            let name = match file.into_os_string().into_string() {
                Ok(ref s) if s.ends_with(".js") => "file://".to_owned() + &s[..],
                _ => continue
            };
            let new_script = doc.CreateElement("script".to_owned()).unwrap().root();
            let new_script = new_script.r();
            new_script.set_string_attribute(&atom!("src"), name);
            let new_script_node: &JSRef<Node> = NodeCast::from_borrowed_ref(&new_script);
            node.InsertBefore(*new_script_node, first_child.r()).unwrap();
        }
    }
}
