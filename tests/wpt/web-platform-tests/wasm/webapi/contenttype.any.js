// META: global=window,worker

const contenttypes = [
  "",
  "application/javascript",
  "application/octet-stream",
  "text/wasm",
  "application/wasm;",
  "application/wasm;x",
  "application/wasm;charset=UTF-8",
];

for (const contenttype of contenttypes) {
  promise_test(t => {
    const response = fetch(`/wasm/incrementer.wasm?pipe=header(Content-Type,${encodeURIComponent(contenttype)})`);
    return promise_rejects(t, new TypeError(), WebAssembly.compileStreaming(response));
  }, `Response with Content-Type ${format_value(contenttype)}: compileStreaming`);

  promise_test(t => {
    const response = fetch(`/wasm/incrementer.wasm?pipe=header(Content-Type,${encodeURIComponent(contenttype)})`);
    return promise_rejects(t, new TypeError(), WebAssembly.instantiateStreaming(response));
  }, `Response with Content-Type ${format_value(contenttype)}: instantiateStreaming`);
}
