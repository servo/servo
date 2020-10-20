// META: script=resources/support.js
//
// Spec: https://wicg.github.io/cors-rfc1918/#integration-html
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: address-space.window.js
'use strict';

setup(() => {
  // No sense running tests if `document.addressSpace` is not implemented.
  assert_implements(document.addressSpace);

  // The tests below assume that the root document is loaded from the `local`
  // address space. This might fail depending on how the tests are run/served.
  // See https://github.com/web-platform-tests/wpt/issues/26166.
  assert_implements_optional(document.addressSpace == "local");
});

promise_test(t => {
  return append_child_frame(t, document, "resources/treat-as-public-address.https.html")
      .then(child => {
        return append_child_frame(t, child.contentDocument, "/common/blank.html");
      })
      .then(grandchild => {
        assert_equals(grandchild.contentDocument.addressSpace, "local");
      });
}, "Public-local grandchild iframe's addressSpace is local");
