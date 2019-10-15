//META: title=Screen wake lock should not be allowed in dedicated worker
importScripts("/resources/testharness.js");

promise_test(t => {
  return promise_rejects(t, "NotAllowedError", navigator.wakeLock.request('screen'));
}, "Screen wake lock should not be allowed in dedicated worker");

done();
