// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(t => {
  const promise = CreateDetachedFrame().requestStorageAccess();
  const description = "document.requestStorageAccess() call in a detached frame";
  // Can't use `promise_rejects_dom` here, since the error comes from the wrong global.
  return promise.then(t.unreached_func("Should have rejected: " + description), (e) => {
    assert_equals(e.name, 'InvalidStateError', description);
    t.done();
  });
}, "[non-fully-active] document.requestStorageAccess() should not resolve when run in a detached frame");

promise_test(t => {
  return promise_rejects_dom(t, 'InvalidStateError', CreateDocumentViaDOMParser().requestStorageAccess(),
   "document.requestStorageAccess() in a detached DOMParser result");
}, "[non-fully-active] document.requestStorageAccess() should not resolve when run in a detached DOMParser document");
