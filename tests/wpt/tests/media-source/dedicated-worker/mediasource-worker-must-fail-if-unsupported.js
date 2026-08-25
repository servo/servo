importScripts("/resources/testharness.js");

test(t => {
  // The Window test html conditionally fetches and runs these tests only if the
  // implementation does not have a true-valued static
  // canConstructInDedicatedWorker property on MediaSource in the Window
  // context. So, the implementation must agree on lack of support here in the
  // dedicated worker context.

  // Ensure we're executing in a dedicated worker context.
  assert_true(self instanceof DedicatedWorkerGlobalScope, "self instanceof DedicatedWorkerGlobalScope");
  assert_true(self.MediaSource === undefined, "MediaSource is undefined in DedicatedWorker");
  assert_throws_js(ReferenceError,
                   function() { var ms = new MediaSource(); },
                   "MediaSource construction in DedicatedWorker throws exception");
}, "MediaSource construction in DedicatedWorker context must fail if Window context did not claim MSE supported in DedicatedWorker");

done();
