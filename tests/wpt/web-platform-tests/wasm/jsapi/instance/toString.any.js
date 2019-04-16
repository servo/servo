// META: global=jsshell
// META: script=/wasm/jsapi/wasm-module-builder.js

test(() => {
  const emptyModuleBinary = new WasmModuleBuilder().toBuffer();
  const module = new WebAssembly.Module(emptyModuleBinary);
  const instance = new WebAssembly.Instance(module);
  assert_class_string(instance, "WebAssembly.Instance");
}, "Object.prototype.toString on an Instance");
