/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::globalscope::GlobalScope;
use dom::htmlheadelement::HTMLHeadElement;
use dom::node::Node;
use js::jsval::UndefinedValue;
use servo_config::opts;
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::PathBuf;
use typeholder::TypeHolderTrait;

pub fn load_script<TH: TypeHolderTrait>(head: &HTMLHeadElement<TH>) {
    if let Some(ref path_str) = opts::get().userscripts {
        let node = head.upcast::<Node<TH>>();
        let doc = node.owner_doc();
        let win = doc.window();
        let cx = win.get_cx();
        rooted!(in(cx) let mut rval = UndefinedValue());

        let path = PathBuf::from(path_str);
        let mut files = read_dir(&path).expect("Bad path passed to --userscripts")
                                       .filter_map(|e| e.ok())
                                       .map(|e| e.path()).collect::<Vec<_>>();

        files.sort();

        for file in files {
            let mut f = File::open(&file).unwrap();
            let mut contents = vec![];
            f.read_to_end(&mut contents).unwrap();
            let script_text = String::from_utf8_lossy(&contents);
            win.upcast::<GlobalScope<TH>>().evaluate_js_on_global_with_result(&script_text, rval.handle_mut());
        }
    }
}
