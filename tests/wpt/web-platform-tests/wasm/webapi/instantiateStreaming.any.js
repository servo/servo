// META: global=window,worker
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js
// META: script=/wasm/jsapi/instanceTestFactory.js

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

for (const [name, fn] of instanceTestFactory) {
  promise_test(async () => {
    const { buffer, args, exports, verify } = fn();
    const response = new Response(buffer, { "headers": { "Content-Type": "application/wasm" } });
    const result = await WebAssembly.instantiateStreaming(response, ...args);
    assert_WebAssemblyInstantiatedSource(result, exports);
    verify(result.instance);
  }, name);
}
