// META: global=window,worker

const bytes = new Uint8Array([0, 0x61, 0x73, 0x6d, 0x1, 0, 0, 0]);

promise_test(t => {
  const response = new Response(bytes, { headers: { "Content-Type": "application/wasm" } });
  return promise_rejects_js(
      t, WebAssembly.CompileError,
      WebAssembly.compileStreaming(response));
}, "WebAssembly.compileStreaming() is blocked");

promise_test(t => {
  const response = new Response(bytes, { headers: { "Content-Type": "application/wasm" } });
  return promise_rejects_js(
      t, WebAssembly.CompileError,
      WebAssembly.instantiateStreaming(response));
}, "WebAssembly.instantiateStreaming() is blocked");
