// META: global=window,worker

const bytes = new Uint8Array([0, 0x61, 0x73, 0x6d, 0x1, 0, 0, 0]);

promise_test(t => {
  return promise_rejects_js(
      t, WebAssembly.CompileError,
      WebAssembly.instantiate(bytes));
}, "WebAssembly.instantiate() is blocked");

promise_test(t => {
  return promise_rejects_js(
      t, WebAssembly.CompileError,
      WebAssembly.compile(bytes));
}, "WebAssembly.compile() is blocked");

test(() => {
  assert_throws_js(
      WebAssembly.CompileError,
      () => new WebAssembly.Module(bytes));
}, "new WebAssembly.Module() is blocked");
