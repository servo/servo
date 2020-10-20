// META: script=resources/support.js
//
// Spec: https://wicg.github.io/cors-rfc1918/#integration-html
//
// See also: address-space.https.window.js
'use strict';

const kMinimalDoc = [
    "<!doctype html>",
    "<meta charset=utf-8>",
    "<title>Loaded</title>",
].join("");

setup(() => {
  // No sense running tests if `document.addressSpace` is not implemented.
  assert_implements(document.addressSpace);

  // The tests below assume that the root document is loaded from the `local`
  // address space. This might fail depending on how the tests are run/served.
  // See https://github.com/web-platform-tests/wpt/issues/26166.
  assert_implements_optional(document.addressSpace == "local");
});

promise_test(t => {
  return append_child_frame_with(t, document, () => {
        // Do nothing with the child frame's `src` attribute.
      })
      .then(child => {
        assert_equals(child.contentDocument.addressSpace, "local");
      });
}, "About:blank iframe's addressSpace is inherited from the root.");

promise_test(t => {
  return append_child_frame_with(t, document, child => {
        child.srcdoc = kMinimalDoc;
      })
      .then(child => {
        assert_equals(child.contentDocument.title, "Loaded");
        assert_equals(child.contentDocument.addressSpace, "local");
      });
}, "About:srcdoc iframe's addressSpace is inherited from the root.");

promise_test(t => {
  // Register an event listener that will resolve this promise when this window
  // receives a message posted to it.
  const event_received = new Promise(resolve => {
    window.addEventListener("message", resolve);
  });

  const script = "window.parent.postMessage(document.addressSpace, '*');";
  const url = "data:text/html,<script>" + script + "</script>";
  return append_child_frame(t, document, url)
      // Wait for the iframe to be loaded, then wait for an event.
      .then(() => event_received)
      .then(evt => {
        assert_equals(evt.data, "local");
      });
}, "Data: iframe's addressSpace is inherited from the root.");

promise_test(t => {
  const blob = new Blob([kMinimalDoc], {type: "text/html"});
  return append_child_frame(t, document, URL.createObjectURL(blob))
      .then(child => {
        assert_equals(child.contentDocument.title, "Loaded");
        assert_equals(child.contentDocument.addressSpace, "local");
      });
}, "Blob: iframe's addressSpace is inherited from the root.");

promise_test(t => {
  return append_child_frame(t, document, "resources/title.html")
      .then(child => {
        assert_equals(child.contentDocument.title, "Loaded");
        assert_equals(child.contentDocument.addressSpace, "local");
      });
}, "Local iframe's addressSpace is local.");

promise_test(t => {
  return append_child_frame(t, document, "resources/treat-as-public-address.html")
      .then(child => {
        assert_equals(child.contentDocument.title, "Loaded");
        assert_equals(child.contentDocument.addressSpace, "public");
      });
}, "Treat-as-public-address iframe's addressSpace is public.");

promise_test(t => {
  return append_child_frame(t, document, "resources/title.html")
      .then(child => {
        const doc = child.contentDocument;
        assert_equals(doc.title, "Loaded", "child");
        assert_equals(doc.addressSpace, "local", "child");
        return append_child_frame(t, doc, "resources/treat-as-public-address.html");
      })
      .then(grandchild => {
        const doc = grandchild.contentDocument;
        assert_equals(doc.title, "Loaded", "grandchild");
        assert_equals(doc.addressSpace, "local", "grandchild");
      });
}, "Local-local grandchild iframe's addressSpace is local.");
