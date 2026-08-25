importScripts("/resources/testharness.js");

test(t => {
  // The Window test html conditionally fetches and runs these tests only if the
  // implementation exposes a true-valued static canConstructInDedicatedWorker
  // attribute on MediaSource in the Window context. So, the implementation must
  // agree on support here in the dedicated worker context.

  // Ensure we're executing in a dedicated worker context.
  assert_true(self instanceof DedicatedWorkerGlobalScope, "self instanceof DedicatedWorkerGlobalScope");
  assert_true(MediaSource.hasOwnProperty("canConstructInDedicatedWorker", "DedicatedWorker MediaSource hasOwnProperty 'canConstructInDedicatedWorker'"));
  assert_true(MediaSource.canConstructInDedicatedWorker, "DedicatedWorker MediaSource.canConstructInDedicatedWorker");
}, "MediaSource in DedicatedWorker context must have true-valued canConstructInDedicatedWorker if Window context had it");

test(t => {
  const ms = new MediaSource();
  assert_equals(ms.readyState, "closed");
}, "MediaSource construction succeeds with initial closed readyState in DedicatedWorker");

test(t => {
  const ms = new MediaSource();
  const url = URL.createObjectURL(ms);
}, "URL.createObjectURL(mediaSource) in DedicatedWorker does not throw exception");

test(t => {
  const ms = new MediaSource();
  const url1 = URL.createObjectURL(ms);
  const url2 = URL.createObjectURL(ms);
  URL.revokeObjectURL(url1);
  URL.revokeObjectURL(url2);
}, "URL.revokeObjectURL(mediaSource) in DedicatedWorker with two url for same MediaSource");

done();
