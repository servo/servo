// META: global=!default,dedicatedworker,sharedworker

test(() => {
  const sab = new SharedArrayBuffer(16);
  const ta = new Int32Array(sab);

  assert_equals(Atomics.wait(ta, 0, 0, 10), "timed-out");
}, `[[CanBlock]] in a ${self.constructor.name}`);
