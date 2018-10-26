const methods = [
  "compileStreaming",
  "instantiateStreaming",
];

for (const method of methods) {
  promise_test(async t => {
    const controller = new AbortController();
    const signal = controller.signal;
    controller.abort();
    const request = fetch('../incrementer.wasm', { signal });
    return promise_rejects(t, 'AbortError', WebAssembly[method](request),
                          `${method} should reject`);
  }, `${method}() on an already-aborted request should reject with AbortError`);

  promise_test(async t => {
    const controller = new AbortController();
    const signal = controller.signal;
    const request = fetch('../incrementer.wasm', { signal });
    const promise = WebAssembly[method](request);
    controller.abort();
    return promise_rejects(t, 'AbortError', promise, `${method} should reject`);
  }, `${method}() synchronously followed by abort should reject with AbortError`);
}
