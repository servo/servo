/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::RootedReference;
use dom::bindings::str::DOMString;
use dom::htmlheadelement::HTMLHeadElement;
use dom::node::Node;
use std::borrow::ToOwned;
use std::fs::read_dir;
use std::path::PathBuf;
use util::opts;
use util::resource_files::resources_dir_path;


pub fn load_script(head: &HTMLHeadElement) {
    if let Some(ref path_str) = opts::get().userscripts {
        let node = head.upcast::<Node>();
        let first_child = node.GetFirstChild();
        let doc = node.owner_doc();

        let path = if &**path_str == "" {
            if let Ok(mut p) = resources_dir_path() {
                p.push("user-agent-js");
                p
            } else {
                return
            }
        } else {
            PathBuf::from(path_str)
        };

        let mut files = read_dir(&path).expect("Bad path passed to --userscripts")
                                       .filter_map(|e| e.ok())
                                       .map(|e| e.path()).collect::<Vec<_>>();

        files.sort();

        for file in files {
            let name = match file.into_os_string().into_string() {
                Ok(ref s) if s.ends_with(".js") => "file://".to_owned() + &s[..],
                _ => continue
            };
            let new_script = doc.CreateElement(DOMString::from("script")).unwrap();
            new_script.set_string_attribute(&atom!("src"), DOMString::from(name));
            node.InsertBefore(new_script.upcast(), first_child.r()).unwrap();
        }
    }
}
