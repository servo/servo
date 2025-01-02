// META: global=window,worker

promise_test(t => {
  return WebAssembly.instantiate(
      new Uint8Array([0, 0x61, 0x73, 0x6d, 0x1, 0, 0, 0]));
});
