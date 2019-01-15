/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::node::document_from_node;
use js::jsval::UndefinedValue;
use servo_config::opts;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;

pub fn load_script(head: &HTMLHeadElement) {
    let path_str = match opts::get().userscripts.clone() {
        Some(p) => p,
        None => return,
    };
    let doc = document_from_node(head);
    let win = Trusted::new(doc.window());
    doc.add_delayed_task(task!(UserScriptExecute: move || {
        let win = win.root();
        let cx = win.get_cx();
        rooted!(in(cx) let mut rval = UndefinedValue());

        let path = PathBuf::from(&path_str);
        let mut files = read_dir(&path)
            .expect("Bad path passed to --userscripts")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect::<Vec<_>>();

        files.sort();

        for file in files {
            let mut f = File::open(&file).unwrap();
            let mut contents = vec![];
            f.read_to_end(&mut contents).unwrap();
            let script_text = String::from_utf8_lossy(&contents);
            win.upcast::<GlobalScope>()
                .evaluate_js_on_global_with_result(&script_text, rval.handle_mut());
        }
    }));
}
