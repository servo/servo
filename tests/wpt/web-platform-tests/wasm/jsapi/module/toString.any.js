// META: global=jsshell
// META: script=/wasm/jsapi/wasm-module-builder.js

test(() => {
  const emptyModuleBinary = new WasmModuleBuilder().toBuffer();
  const module = new WebAssembly.Module(emptyModuleBinary);
  assert_class_string(module, "WebAssembly.Module");
}, "Object.prototype.toString on an Module");
