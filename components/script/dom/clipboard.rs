/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use constellation_traits::BlobImpl;
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;

use crate::dom::bindings::codegen::Bindings::ClipboardBinding::{
    ClipboardMethods, PresentationStyle,
};
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Clipboard {
    event_target: EventTarget,
}

impl Clipboard {
    fn new_inherited() -> Clipboard {
        Clipboard {
            event_target: EventTarget::new_inherited(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Clipboard> {
        reflect_dom_object(Box::new(Clipboard::new_inherited()), global, can_gc)
    }
}

impl ClipboardMethods<crate::DomTypeHolder> for Clipboard {
    /// <https://w3c.github.io/clipboard-apis/#dom-clipboard-writetext>
    fn WriteText(&self, data: DOMString, can_gc: CanGc) -> Rc<Promise> {
        // Step 1 Let realm be this's relevant realm.
        // Step 2 Let p be a new promise in realm.
        let p = Promise::new(&self.global(), can_gc);

        // Step 3 Run the following steps in parallel:

        // TODO write permission could be removed from spec
        // Step 3.1 Let r be the result of running check clipboard write permission.
        // Step 3.2 If r is false, then:
        // Step 3.2.1 Queue a global task on the permission task source, given realm’s global object,
        // to reject p with "NotAllowedError" DOMException in realm.
        // Step 3.2.2 Abort these steps.

        let trusted_promise = TrustedPromise::new(p.clone());
        let bytes = Vec::from(data);

        // Step 3.3 Queue a global task on the clipboard task source,
        // given realm’s global object, to perform the below steps:
        self.global().task_manager().clipboard_task_source().queue(
            task!(write_to_system_clipboard: move || {
                let promise = trusted_promise.root();
                let global = promise.global();

                // Step 3.3.1 Let itemList be an empty sequence<Blob>.
                let mut item_list = Vec::new();

                // Step 3.3.2 Let textBlob be a new Blob created with: type attribute set to "text/plain;charset=utf-8",
                // and its underlying byte sequence set to the UTF-8 encoding of data.
                let text_blob = Blob::new(
                    &global,
                    BlobImpl::new_from_bytes(bytes, "text/plain;charset=utf-8".into()),
                    CanGc::note(),
                );

                // Step 3.3.3 Add textBlob to itemList.
                item_list.push(text_blob);

                // Step 3.3.4 Let option be set to "unspecified".
                let option = PresentationStyle::Unspecified;

                // Step 3.3.5 Write blobs and option to the clipboard with itemList and option.
                write_blobs_and_option_to_the_clipboard(global.as_window(), item_list, option);

                // Step 3.3.6 Resolve p.
                promise.resolve_native(&(), CanGc::note());
            }),
        );

        // Step 3.4 Return p.
        p
    }
}

/// <https://w3c.github.io/clipboard-apis/#write-blobs-and-option-to-the-clipboard>
fn write_blobs_and_option_to_the_clipboard(
    window: &Window,
    items: Vec<DomRoot<Blob>>,
    _presentation_style: PresentationStyle,
) {
    // TODO Step 1 Let webCustomFormats be a sequence<Blob>.

    // Step 2 For each item in items:
    for item in items {
        // TODO support more formats than just text/plain
        // Step 2.1 Let formatString be the result of running os specific well-known format given item’s type.

        // Step 2.2 If formatString is empty then follow the below steps:

        // Step 2.2.1 Let webCustomFormatString be the item’s type.
        // Step 2.2.2 Let webCustomFormat be an empty type.
        // Step 2.2.3 If webCustomFormatString starts with "web " prefix,
        // then remove the "web " prefix and store the remaining string in webMimeTypeString.
        // Step 2.2.4 Let webMimeType be the result of parsing a MIME type given webMimeTypeString.
        // Step 2.2.5 If webMimeType is failure, then abort all steps.
        // Step 2.2.6 Let webCustomFormat’s type's essence equal to webMimeType.
        // Step 2.2.7 Set item’s type to webCustomFormat.
        // Step 2.2.8 Append webCustomFormat to webCustomFormats.

        // Step 2.3 Let payload be the result of UTF-8 decoding item’s underlying byte sequence.
        // Step 2.4 Insert payload and presentationStyle into the system clipboard
        // using formatString as the native clipboard format.
        window.send_to_embedder(EmbedderMsg::SetClipboardText(
            window.webview_id(),
            String::from_utf8(
                item.get_bytes()
                    .expect("No bytes found for Blob created by caller"),
            )
            .expect("DOMString contained invalid bytes"),
        ));
    }

    // TODO Step 3 Write web custom formats given webCustomFormats.
    // Needs support to arbitrary formats inside arboard
}
