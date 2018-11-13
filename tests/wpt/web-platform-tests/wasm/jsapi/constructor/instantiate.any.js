// META: global=jsshell
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js
// META: script=/wasm/jsapi/instanceTestFactory.js

function assert_WebAssemblyInstantiatedSource(actual, expected_exports={}) {
  assert_equals(Object.getPrototypeOf(actual), Object.prototype,
                "Prototype");
  assert_true(Object.isExtensible(actual), "Extensibility");

  const module = Object.getOwnPropertyDescriptor(actual, "module");
  assert_equals(typeof module, "object", "module: type of descriptor");
  assert_true(module.writable, "module: writable");
  assert_true(module.enumerable, "module: enumerable");
  assert_true(module.configurable, "module: configurable");
  assert_equals(Object.getPrototypeOf(module.value), WebAssembly.Module.prototype,
                "module: prototype");

  const instance = Object.getOwnPropertyDescriptor(actual, "instance");
  assert_equals(typeof instance, "object", "instance: type of descriptor");
  assert_true(instance.writable, "instance: writable");
  assert_true(instance.enumerable, "instance: enumerable");
  assert_true(instance.configurable, "instance: configurable");
  assert_Instance(instance.value, expected_exports);
}

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

promise_test(t => {
  return promise_rejects(t, new TypeError(), WebAssembly.instantiate());
}, "Missing arguments");

promise_test(() => {
  const fn = WebAssembly.instantiate;
  const thisValues = [
    undefined,
    null,
    true,
    "",
    Symbol(),
    1,
    {},
    WebAssembly,
  ];
  return Promise.all(thisValues.map(thisValue => {
    return fn.call(thisValue, emptyModuleBinary).then(assert_WebAssemblyInstantiatedSource);
  }));
}, "Branding");

promise_test(t => {
  const invalidArguments = [
    undefined,
    null,
    true,
    "",
    Symbol(),
    1,
    {},
    WebAssembly.Module,
    WebAssembly.Module.prototype,
    ArrayBuffer,
    ArrayBuffer.prototype,
    Array.from(emptyModuleBinary),
  ];
  return Promise.all(invalidArguments.map(argument => {
    return promise_rejects(t, new TypeError(), WebAssembly.instantiate(argument),
                           `instantiate(${format_value(argument)})`);
  }));
}, "Invalid arguments");

test(() => {
  const promise = WebAssembly.instantiate(emptyModuleBinary);
  assert_equals(Object.getPrototypeOf(promise), Promise.prototype, "prototype");
  assert_true(Object.isExtensible(promise), "extensibility");
}, "Promise type");

for (const [name, fn] of instanceTestFactory) {
  promise_test(() => {
    const { buffer, args, exports, verify } = fn();
    return WebAssembly.instantiate(buffer, ...args).then(result => {
      assert_WebAssemblyInstantiatedSource(result, exports);
      verify(result.instance);
    });
  }, `${name}: BufferSource argument`);

  promise_test(() => {
    const { buffer, args, exports, verify } = fn();
    const module = new WebAssembly.Module(buffer);
    return WebAssembly.instantiate(module, ...args).then(instance => {
      assert_Instance(instance, exports);
      verify(instance);
    });
  }, `${name}: Module argument`);
}

promise_test(t => {
  const buffer = new Uint8Array();
  return promise_rejects(t, new WebAssembly.CompileError(), WebAssembly.instantiate(buffer));
}, "Invalid code");

promise_test(() => {
  const buffer = new WasmModuleBuilder().toBuffer();
  assert_equals(buffer[0], 0);
  const promise = WebAssembly.instantiate(buffer);
  buffer[0] = 1;
  return promise.then(assert_WebAssemblyInstantiatedSource);
}, "Changing the buffer");
