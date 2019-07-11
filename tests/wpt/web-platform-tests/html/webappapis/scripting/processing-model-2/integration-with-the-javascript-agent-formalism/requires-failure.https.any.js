// META: global=!default,window,serviceworker

test(() => {
  const sab = new SharedArrayBuffer(16);
  const ta = new Int32Array(sab);

  assert_throws(new TypeError(), () => {
    Atomics.wait(ta, 0, 0, 10);
  });
}, `[[CanBlock]] in a ${self.constructor.name}`);
