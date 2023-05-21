// META: global=window,dedicatedworker

promise_test(async () => {
  const sab = new SharedArrayBuffer(64);
  const ta = new Int32Array(sab);

  const waitAsyncObj = Atomics.waitAsync(ta, 0, 0, 10);
  assert_equals(waitAsyncObj.async, true);
  const v = await waitAsyncObj.value;
  assert_equals(v, "timed-out");
}, `Atomics.waitAsync timeout in a ${self.constructor.name}`);

promise_test(async () => {
  const sab = new SharedArrayBuffer(64);
  const ta = new Int32Array(sab);

  const waitAsyncObj = Atomics.waitAsync(ta, 0, 0);
  assert_equals(waitAsyncObj.async, true);

  const worker = new Worker("resources/notify-worker.js");
  worker.postMessage(sab);

  const v = await waitAsyncObj.value;
  assert_equals(v, "ok");
}, `Atomics.waitAsync notification in a ${self.constructor.name}`);
