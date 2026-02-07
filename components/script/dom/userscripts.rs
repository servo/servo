/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsval::UndefinedValue;
use script_bindings::root::DomRoot;

use crate::dom::html::htmlheadelement::HTMLHeadElement;
use crate::dom::node::NodeTraits;
use crate::dom::window::Window;
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

        let global_scope = win.as_global_scope();
        for user_script in userscripts {
            _ = global_scope.evaluate_js_on_global(
                user_script.script().into(),
                &user_script.source_file().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
                None,
                rval.handle_mut(),
                CanGc::note(),
            );
        }
    }));
}
