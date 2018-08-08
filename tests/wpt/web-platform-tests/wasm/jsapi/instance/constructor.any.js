// META: global=jsshell
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js

function assert_exported_function(fn, { name, length }, description) {
  assert_equals(Object.getPrototypeOf(fn), Function.prototype,
                `${description}: prototype`);

  assert_function_name(fn, name, description);
  assert_function_length(fn, length, description);
}

function assert_Instance(instance, expected_exports) {
  assert_equals(Object.getPrototypeOf(instance), WebAssembly.Instance.prototype,
                "prototype");
  assert_true(Object.isExtensible(instance), "extensible");

  assert_equals(instance.exports, instance.exports, "exports should be idempotent");
  const exports = instance.exports;

  assert_equals(Object.getPrototypeOf(exports), null, "exports prototype");
  assert_false(Object.isExtensible(exports), "extensible exports");
  for (const [key, expected] of Object.entries(expected_exports)) {
    const property = Object.getOwnPropertyDescriptor(exports, key);
    assert_equals(typeof property, "object", `${key} should be present`);
    assert_false(property.writable, `${key}: writable`);
    assert_true(property.enumerable, `${key}: enumerable`);
    assert_false(property.configurable, `${key}: configurable`);
    const actual = property.value;

    switch (expected.kind) {
    case "function":
      assert_exported_function(actual, expected, `value of ${key}`);
      break;
    case "global":
      assert_equals(Object.getPrototypeOf(actual), WebAssembly.Global.prototype,
                    `value of ${key}: prototype`);
      assert_equals(actual.value, expected.value, `value of ${key}: value`);
      assert_equals(actual.valueOf(), expected.value, `value of ${key}: valueOf()`);
      break;
    case "memory":
      assert_equals(Object.getPrototypeOf(actual), WebAssembly.Memory.prototype,
                    `value of ${key}: prototype`);
      assert_equals(Object.getPrototypeOf(actual.buffer), ArrayBuffer.prototype,
                    `value of ${key}: prototype of buffer`);
      assert_equals(actual.buffer.byteLength, 0x10000 * expected.size, `value of ${key}: size of buffer`);
      const array = new Uint8Array(actual.buffer);
      assert_equals(array[0], 0, `value of ${key}: first element of buffer`);
      assert_equals(array[array.byteLength - 1], 0, `value of ${key}: last element of buffer`);
      break;
    case "table":
      assert_equals(Object.getPrototypeOf(actual), WebAssembly.Table.prototype,
                    `value of ${key}: prototype`);
      assert_equals(actual.length, expected.length, `value of ${key}: length of table`);
      break;
    }
  }
}

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

test(() => {
  assert_function_name(WebAssembly.Instance, "Instance", "WebAssembly.Instance");
}, "name");

test(() => {
  assert_function_length(WebAssembly.Instance, 1, "WebAssembly.Instance");
}, "length");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Instance());
}, "No arguments");

test(() => {
  const module = new WebAssembly.Module(emptyModuleBinary);
  assert_throws(new TypeError(), () => WebAssembly.Instance(module));
}, "Calling");

test(() => {
  const module = new WebAssembly.Module(emptyModuleBinary);
  const arguments = [
    [],
    [undefined],
    [{}],
  ];
  for (const value of arguments) {
    const instance = new WebAssembly.Instance(module, ...arguments);
    assert_Instance(instance, {});
  }
}, "Empty module");

test(() => {
  const builder = new WasmModuleBuilder();
  builder.addImportedGlobal("module", "global1", kWasmI32);
  builder.addImportedGlobal("module", "global2", kWasmI32);
  const buffer = builder.toBuffer();
  const module = new WebAssembly.Module(buffer);
  const order = [];
  const imports = {
    get module() {
      order.push("module getter");
      return {
        get global1() {
          order.push("global1 getter");
          return 0;
        },
        get global2() {
          order.push("global2 getter");
          return 0;
        },
      }
    },
  };
  new WebAssembly.Instance(module, imports);
  const expected = [
    "module getter",
    "global1 getter",
    "module getter",
    "global2 getter",
  ];
  assert_array_equals(order, expected);
}, "getter order for imports object");

test(() => {
  const builder = new WasmModuleBuilder();

  builder.addImport("module", "fn", kSig_v_v);
  builder.addImportedGlobal("module", "global", kWasmI32);
  builder.addImportedMemory("module", "memory", 0, 128);
  builder.addImportedTable("module", "table", 0, 128);

  const buffer = builder.toBuffer();
  const module = new WebAssembly.Module(buffer);
  const instance = new WebAssembly.Instance(module, {
    "module": {
      "fn": function() {},
      "global": 0,
      "memory": new WebAssembly.Memory({ "initial": 64, maximum: 128 }),
      "table": new WebAssembly.Table({ "element": "anyfunc", "initial": 64, maximum: 128 }),
    },
    get "module2"() {
      assert_unreached("Should not get modules that are not imported");
    },
  });
  assert_Instance(instance, {});
}, "imports");

test(() => {
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

  const buffer = builder.toBuffer()
  const module = new WebAssembly.Module(buffer);

  const instance = new WebAssembly.Instance(module, {});
  const expected = {
    "fn": { "kind": "function", "name": "0", "length": 1 },
    "fn2": { "kind": "function", "name": "1", "length": 0 },
    "table": { "kind": "table", "length": 1 },
    "global": { "kind": "global", "value": 7 },
    "global2": { "kind": "global", "value": 1.2 },
    "memory": { "kind": "memory", "size": 4 },
  };
  assert_Instance(instance, expected);
}, "exports");
