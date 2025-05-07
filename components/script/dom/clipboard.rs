/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::str::FromStr;

use constellation_traits::BlobImpl;
use data_url::mime::Mime;
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use js::rust::HandleValue as SafeHandleValue;

use crate::dom::bindings::codegen::Bindings::ClipboardBinding::{
    ClipboardMethods, PresentationStyle,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::clipboarditem::Representation;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::window::Window;
use crate::realms::{InRealm, enter_realm};
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The fulfillment handler for the reacting to representationDataPromise part of
/// <https://w3c.github.io/clipboard-apis/#dom-clipboard-readtext>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct RepresentationDataPromiseFulfillmentHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    promise: Rc<Promise>,
}

impl Callback for RepresentationDataPromiseFulfillmentHandler {
    /// The fulfillment case of Step 3.4.1.1.4.3 of
    /// <https://w3c.github.io/clipboard-apis/#dom-clipboard-readtext>.
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If v is a DOMString, then follow the below steps:
        // Resolve p with v.
        // Return p.
        self.promise.resolve(cx, v, can_gc);

        // NOTE: Since we ask text from arboard, v can't be a Blob
        // If v is a Blob, then follow the below steps:
        // Let string be the result of UTF-8 decoding v’s underlying byte sequence.
        // Resolve p with string.
        // Return p.
    }
}

/// The rejection handler for the reacting to representationDataPromise part of
/// <https://w3c.github.io/clipboard-apis/#dom-clipboard-readtext>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct RepresentationDataPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    promise: Rc<Promise>,
}

impl Callback for RepresentationDataPromiseRejectionHandler {
    /// The rejection case of Step 3.4.1.1.4.3 of
    /// <https://w3c.github.io/clipboard-apis/#dom-clipboard-readtext>.
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Reject p with "NotFoundError" DOMException in realm.
        // Return p.
        self.promise.reject_error(Error::NotFound, can_gc);
    }
}

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
    /// <https://w3c.github.io/clipboard-apis/#dom-clipboard-readtext>
    fn ReadText(&self, can_gc: CanGc) -> Rc<Promise> {
        // Step 1 Let realm be this's relevant realm.
        let global = self.global();

        // Step 2 Let p be a new promise in realm.
        let p = Promise::new(&global, can_gc);

        // Step 3 Run the following steps in parallel:

        // TODO Step 3.1 Let r be the result of running check clipboard read permission.
        // Step 3.2 If r is false, then:
        // Step 3.2.1 Queue a global task on the permission task source, given realm’s global object,
        // to reject p with "NotAllowedError" DOMException in realm.
        // Step 3.2.2 Abort these steps.

        // Step 3.3 Let data be a copy of the system clipboard data.
        let window = global.as_window();
        let sender = route_promise(&p, self, global.task_manager().clipboard_task_source());
        window.send_to_embedder(EmbedderMsg::GetClipboardText(window.webview_id(), sender));

        // Step 3.4 Queue a global task on the clipboard task source,
        // given realm’s global object, to perform the below steps:
        // NOTE: We queue the task inside route_promise and perform the steps inside handle_response

        p
    }

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

impl RoutedPromiseListener<Result<String, String>> for Clipboard {
    fn handle_response(
        &self,
        response: Result<String, String>,
        promise: &Rc<Promise>,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let text = response.unwrap_or_default();

        // Step 3.4.1 For each systemClipboardItem in data:
        // Step 3.4.1.1 For each systemClipboardRepresentation in systemClipboardItem:
        // TODO: Arboard provide the first item that has a String representation

        // Step 3.4.1.1.1 Let mimeType be the result of running the
        // well-known mime type from os specific format algorithm given systemClipboardRepresentation’s name.
        // Note: This is done by arboard, so we just convert the format to a MIME
        let mime_type = Mime::from_str("text/plain").unwrap();

        // Step 3.4.1.1.2 If mimeType is null, continue this loop.
        // Note: Since the previous step is infallible, we don't need to handle this case

        // Step 3.4.1.1.3 Let representation be a new representation.
        let representation = Representation {
            mime_type,
            is_custom: false,
            data: Promise::new_resolved(
                &global,
                GlobalScope::get_cx(),
                DOMString::from(text),
                can_gc,
            ),
        };

        // Step 3.4.1.1.4 If representation’s MIME type essence is "text/plain", then:

        // Step 3.4.1.1.4.1 Set representation’s MIME type to mimeType.
        // Note: Done when creating a new representation

        // Step 3.4.1.1.4.2 Let representationDataPromise be the representation’s data.
        // Step 3.4.1.1.4.3 React to representationDataPromise:
        let fulfillment_handler = Box::new(RepresentationDataPromiseFulfillmentHandler {
            promise: promise.clone(),
        });
        let rejection_handler = Box::new(RepresentationDataPromiseRejectionHandler {
            promise: promise.clone(),
        });
        let handler = PromiseNativeHandler::new(
            &global,
            Some(fulfillment_handler),
            Some(rejection_handler),
            can_gc,
        );
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        representation
            .data
            .append_native_handler(&handler, comp, can_gc);

        // Step 3.4.2 Reject p with "NotFoundError" DOMException in realm.
        // Step 3.4.3 Return p.
        // NOTE: We follow the same behaviour of Gecko by doing nothing if no text is available instead of rejecting p
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
