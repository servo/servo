/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::globalscope::GlobalScope;
use dom::htmlheadelement::HTMLHeadElement;
use dom::node::Node;
use js::jsval::UndefinedValue;
use servo_config::opts;
use servo_config::resource_files::resources_dir_path;
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::PathBuf;


pub fn load_script(head: &HTMLHeadElement) {
    if let Some(ref path_str) = opts::get().userscripts {
        let node = head.upcast::<Node>();
        let doc = node.owner_doc();
        let win = doc.window();
        let cx = win.get_cx();
        rooted!(in(cx) let mut rval = UndefinedValue());

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
            let mut f = File::open(&file).unwrap();
            let mut contents = vec![];
            f.read_to_end(&mut contents).unwrap();
            let script_text = String::from_utf8_lossy(&contents);
            win.upcast::<GlobalScope>().evaluate_js_on_global_with_result(&script_text, rval.handle_mut());
        }
    }
}
