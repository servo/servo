// META: global=window,dedicatedworker,jsshell,shadowrealm
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

test(() => {
  assert_equals(typeof AbstractModuleSource, "undefined");
  const AbstractModuleSource = Object.getPrototypeOf(WebAssembly.Module);
  assert_equals(AbstractModuleSource.name, "AbstractModuleSource");
  assert_not_equals(AbstractModuleSource, Function);
}, "AbstractModuleSource intrinsic");

test(() => {
  const AbstractModuleSourceProto = Object.getPrototypeOf(WebAssembly.Module.prototype);
  assert_not_equals(AbstractModuleSourceProto, Object);
  const AbstractModuleSource = Object.getPrototypeOf(WebAssembly.Module);
  assert_equals(AbstractModuleSource.prototype, AbstractModuleSourceProto);
}, "AbstractModuleSourceProto intrinsic");

test(() => {
  const builder = new WasmModuleBuilder();

  builder
    .addFunction("fn", kSig_v_v)
    .addBody([])
    .exportFunc();
  builder.addMemory(0, 256, true);

  const buffer = builder.toBuffer()
  const module = new WebAssembly.Module(buffer);

  const AbstractModuleSource = Object.getPrototypeOf(WebAssembly.Module);
  const toStringTag = Object.getOwnPropertyDescriptor(AbstractModuleSource.prototype, Symbol.toStringTag).get;

  assert_equals(toStringTag.call(module), "WebAssembly.Module");
  assert_throws_js(TypeError, () => toStringTag.call({}));
}, "AbstractModuleSourceProto toStringTag brand check");