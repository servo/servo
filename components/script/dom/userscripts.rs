/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use js::jsval::UndefinedValue;
use script_bindings::root::DomRoot;

use crate::dom::bindings::str::DOMString;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlscriptelement::SourceCode;
use crate::dom::node::NodeTraits;
use crate::dom::window::Window;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::CanGc;

pub(crate) fn load_script(head: &HTMLHeadElement) {
    let doc = head.owner_document();
    let userscripts = doc.window().userscripts().to_owned();
    if userscripts.is_empty() {
        return;
    }
    let win = DomRoot::from_ref(doc.window());
    doc.add_delayed_task(task!(UserScriptExecute: |win: DomRoot<Window>| {
        let cx = win.get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());

        for user_script in userscripts {
            let script_text = SourceCode::Text(
                Rc::new(DOMString::from_string(user_script.script))
            );
            let global_scope = win.as_global_scope();
            global_scope.evaluate_script_on_global_with_result(
                &script_text,
                &user_script.source_file.map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
                rval.handle_mut(),
                1,
                ScriptFetchOptions::default_classic_script(global_scope),
                global_scope.api_base_url(),
                CanGc::note(),
            );
        }
    }));
}
