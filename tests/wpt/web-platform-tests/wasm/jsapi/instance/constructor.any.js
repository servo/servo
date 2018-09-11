// META: global=jsshell
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js

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
  ];
  for (const argument of invalidArguments) {
    assert_throws(new TypeError(), () => new WebAssembly.Instance(argument),
                  `new Instance(${format_value(argument)})`);
  }
}, "Non-Module arguments");

test(() => {
  const module = new WebAssembly.Module(emptyModuleBinary);
  const invalidArguments = [
    null,
    true,
    "",
    Symbol(),
    1,
  ];
  for (const argument of invalidArguments) {
    assert_throws(new TypeError(), () => new WebAssembly.Instance(module, argument),
                  `new Instance(module, ${format_value(argument)})`);
  }
}, "Non-object imports");

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
  builder.addImportedGlobal("module2", "global3", kWasmI32);
  builder.addImportedMemory("module", "memory", 0, 128);
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
        get memory() {
          order.push("memory getter");
          return new WebAssembly.Memory({ "initial": 64, maximum: 128 });
        },
      }
    },
    get module2() {
      order.push("module2 getter");
      return {
        get global3() {
          order.push("global3 getter");
          return 0;
        },
      }
    },
  };
  new WebAssembly.Instance(module, imports);
  const expected = [
    "module getter",
    "global1 getter",
    "module2 getter",
    "global3 getter",
    "module getter",
    "memory getter",
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

test(() => {
  const value = 102;

  const builder = new WasmModuleBuilder();

  builder.addImportedGlobal("module", "global", kWasmI32);
  builder
    .addFunction("fn", kSig_i_v)
    .addBody([
        kExprGetGlobal,
        0,
        kExprReturn,
        kExprEnd,
    ])
    .exportFunc();

  const buffer = builder.toBuffer();
  const module = new WebAssembly.Module(buffer);
  const instance = new WebAssembly.Instance(module, {
    "module": {
      "global": value,
    },
  });
  const expected = {
    "fn": { "kind": "function", "name": "0", "length": 0 },
  };
  assert_Instance(instance, expected);

  assert_equals(instance.exports.fn(), value);
}, "exports and imports");
