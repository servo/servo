// META: global=jsshell
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js

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

const createModule = () => {
  const builder = new WasmModuleBuilder();

  builder
    .addFunction("fn", kSig_v_d)
    .addBody([
        kExprEnd
    ])
    .exportFunc();
  builder
    .addFunction("fn2", kSig_v_v)
    .addBody([
        kExprEnd
    ])
    .exportFunc();

  builder.setFunctionTableLength(1);
  builder.addExportOfKind("table", kExternalTable, 0);

  builder.addGlobal(kWasmI32, true)
    .exportAs("global")
    .init = 7;
  builder.addGlobal(kWasmF64, true)
    .exportAs("global2")
    .init = 1.2;

  builder.addMemory(4, 8, true);

  const buffer = builder.toBuffer();

  const exports = {
    "fn": { "kind": "function", "name": "0", "length": 1 },
    "fn2": { "kind": "function", "name": "1", "length": 0 },
    "table": { "kind": "table", "length": 1 },
    "global": { "kind": "global", "value": 7 },
    "global2": { "kind": "global", "value": 1.2 },
    "memory": { "kind": "memory", "size": 4 },
  };

  return [buffer, exports];
}

promise_test(() => {
  const [buffer, expected] = createModule();
  return WebAssembly.instantiate(buffer).then(result => assert_WebAssemblyInstantiatedSource(result, expected));
}, "BufferSource argument");

promise_test(() => {
  const [buffer, expected] = createModule();
  const module = new WebAssembly.Module(buffer);
  return WebAssembly.instantiate(module).then(instance => assert_Instance(instance, expected));
}, "Module argument");

const createModuleWithImports = () => {
  const builder = new WasmModuleBuilder();

  const index = builder.addImportedGlobal("module", "global", kWasmI32);
  builder
    .addFunction("fn", kSig_i_v)
    .addBody([
        kExprGetGlobal,
        index,
        kExprReturn,
        kExprEnd,
    ])
    .exportFunc();

  const buffer = builder.toBuffer();

  const expected = {
    "fn": { "kind": "function", "name": "0", "length": 0 },
  };

  return [buffer, expected];
};

promise_test(() => {
  const [buffer, expected] = createModuleWithImports();

  const value = 102;
  return WebAssembly.instantiate(buffer, {
    "module": {
      "global": value,
    },
  }).then(result => {
    assert_WebAssemblyInstantiatedSource(result, expected)
    assert_equals(result.instance.exports.fn(), value);
  });
}, "exports and imports: buffer argument");

promise_test(() => {
  const [buffer, expected] = createModuleWithImports();
  const module = new WebAssembly.Module(buffer);

  const value = 102;
  return WebAssembly.instantiate(module, {
    "module": {
      "global": value,
    },
  }).then(instance => {
    assert_Instance(instance, expected)
    assert_equals(instance.exports.fn(), value);
  });
}, "exports and imports: Module argument");

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
