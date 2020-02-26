// META: global=window,worker
// META: script=/wasm/jsapi/wasm-module-builder.js

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

for (const method of ["compileStreaming", "instantiateStreaming"]) {
  promise_test(t => {
    const buffer = new Uint8Array(Array.from(emptyModuleBinary).concat([0xCA, 0xFE]));
    const response = new Response(buffer, { headers: { "Content-Type": "application/wasm" } });
    return promise_rejects_js(t, WebAssembly.CompileError, WebAssembly[method](response));
  }, `Invalid code: ${method}`);
}
