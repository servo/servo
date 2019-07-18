// META: title=WakeLock.request() AbortSignal Test

'use strict';

promise_test(async t => {
  const invalidSignals = [
    "string",
    123,
    {},
    true,
    Symbol(),
    () => {},
    self
  ];

  for (let signal of invalidSignals) {
    await promise_rejects(t, new TypeError(), WakeLock.request('system', { signal: signal }));
  }
}, "'TypeError' is thrown when the signal option is not an AbortSignal");

promise_test(t => {
  const abortController = new AbortController();
  const abortSignal = abortController.signal;
  abortController.abort();
  assert_true(abortSignal.aborted);

  return promise_rejects(t, "AbortError", WakeLock.request('system', { signal: abortSignal }));
}, "A WakeLock request with an AbortSignal whose abort flag is set always aborts");

promise_test(async t => {
  const abortController = new AbortController();
  const abortSignal = abortController.signal;
  abortController.abort();
  assert_true(abortSignal.aborted);

  const lock1 = WakeLock.request('system', { signal: abortSignal });
  const lock2 = WakeLock.request('system', { signal: abortSignal });
  const lock3 = WakeLock.request('system', { signal: abortSignal });

  await promise_rejects(t, "AbortError", lock1);
  await promise_rejects(t, "AbortError", lock2);
  await promise_rejects(t, "AbortError", lock3);
}, "The same AbortSignal can be used to cause multiple wake locks to abort");
