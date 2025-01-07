/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use js::jsval::UndefinedValue;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlscriptelement::SourceCode;
use crate::dom::node::NodeTraits;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::CanGc;

pub fn load_script(head: &HTMLHeadElement) {
    let doc = head.owner_document();
    let path_str = match doc.window().get_userscripts_path() {
        Some(p) => p,
        None => return,
    };
    let window = Trusted::new(doc.window());
    doc.add_delayed_task(task!(UserScriptExecute: move || {
        let win = window.root();
        let cx = win.get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());

        let path = PathBuf::from(&path_str);
        let mut files = read_dir(path)
            .expect("Bad path passed to --userscripts")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect::<Vec<_>>();

        files.sort();

        for file in files {
            let mut f = File::open(&file).unwrap();
            let mut contents = vec![];
            f.read_to_end(&mut contents).unwrap();
            let script_text = SourceCode::Text(
                Rc::new(DOMString::from_string(String::from_utf8_lossy(&contents).to_string()))
            );

            let global_scope = win.as_global_scope();
            global_scope.evaluate_script_on_global_with_result(
                &script_text,
                &file.to_string_lossy(),
                rval.handle_mut(),
                1,
                ScriptFetchOptions::default_classic_script(global_scope),
                global_scope.api_base_url(),
                CanGc::note(),
            );
        }
    }));
}
