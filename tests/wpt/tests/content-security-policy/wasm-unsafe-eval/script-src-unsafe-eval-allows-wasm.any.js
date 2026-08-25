// META: global=window,worker

const bytes = new Uint8Array([0, 0x61, 0x73, 0x6d, 0x1, 0, 0, 0]);

promise_test(t => {
  return WebAssembly.instantiate(bytes);
}, "WebAssembly.instantiate() is allowed");

promise_test(t => {
  return WebAssembly.compile(bytes);
}, "WebAssembly.compile() is allowed");

test(() => {
  new WebAssembly.Module(bytes);
}, "new WebAssembly.Module() is allowed");

promise_test(t => {
  const response = new Response(bytes, { headers: { "Content-Type": "application/wasm" } });
  return WebAssembly.compileStreaming(response);
}, "WebAssembly.compileStreaming() is allowed");

promise_test(t => {
  const response = new Response(bytes, { headers: { "Content-Type": "application/wasm" } });
  return WebAssembly.instantiateStreaming(response);
}, "WebAssembly.instantiateStreaming() is allowed");
