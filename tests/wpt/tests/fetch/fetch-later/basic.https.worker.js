importScripts('/resources/testharness.js');

test(() => {
  assert_false('fetchLater' in self);
}, `fetchLater() is not supported in worker.`);
done();
