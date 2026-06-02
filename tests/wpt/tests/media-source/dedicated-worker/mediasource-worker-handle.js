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
  assert_true(
      'handle' in MediaSource.prototype,
      'dedicated worker MediaSource must have handle in prototype');
  assert_true(self.hasOwnProperty("MediaSourceHandle"), "dedicated worker must have MediaSourceHandle visibility");
}, 'MediaSource prototype in DedicatedWorker context must have \'handle\', and worker must have MediaSourceHandle');

test(t => {
  const ms = new MediaSource();
  assert_equals(ms.readyState, "closed");
}, "MediaSource construction succeeds with initial closed readyState in DedicatedWorker");

test(t => {
  const ms = new MediaSource();
  const handle = ms.handle;
  assert_not_equals(handle, null, 'must have a non-null \'handle\' attribute');
  assert_true(handle instanceof MediaSourceHandle, "must be a MediaSourceHandle");
}, 'mediaSource.handle in DedicatedWorker returns a MediaSourceHandle');

test(t => {
  const msA = new MediaSource();
  const msB = new MediaSource();
  const handleA1 = msA.handle;
  const handleA2 = msA.handle;
  const handleA3 = msA['handle'];
  const handleB1 = msB.handle;
  const handleB2 = msB.handle;
  assert_true(
      handleA1 === handleA2 && handleB1 === handleB2 && handleA1 != handleB1,
      'SameObject is observed for mediaSource.handle, and different MediaSource instances have different handles');
  assert_true(
      handleA1 === handleA3,
      'SameObject is observed even when accessing handle differently');
  assert_true(
      handleA1 instanceof MediaSourceHandle &&
          handleB1 instanceof MediaSourceHandle,
      'handle property returns MediaSourceHandles');
}, 'mediaSource.handle observes SameObject property correctly');

test(t => {
  const ms1 = new MediaSource();
  const handle1 = ms1.handle;
  const ms2 = new MediaSource();
  const handle2 = ms2.handle;
  assert_true(
      handle1 !== handle2,
      'distinct MediaSource instances must have distinct handles');

  // Verify attempt to change value of the handle property does not succeed.
  ms1.handle = handle2;
  assert_true(
      ms1.handle === handle1 && ms2.handle === handle2,
      'MediaSource handle is readonly, so should not have changed');
}, 'Attempt to set MediaSource handle property should fail to change it, since it is readonly');

done();
