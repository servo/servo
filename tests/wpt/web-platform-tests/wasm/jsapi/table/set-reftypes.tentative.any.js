// META: global=window,dedicatedworker,jsshell
// META: script=assertions.js
// META: script=/wasm/jsapi/wasm-module-builder.js

// Test cases for changes to the WebAssembly.Table.prototype.set() API that
// come in with the reftypes proposal: the API makes the second argument optional and
// if it is missing we should use DefaultValue of the table's element type.
//
// See:
//   https://webassembly.github.io/reference-types/js-api/index.html#tables

test(() => {
  const builder = new WasmModuleBuilder();
  builder
    .addFunction("fn", kSig_v_v)
    .addBody([])
    .exportFunc();
  const bin = builder.toBuffer();
  const fn = new WebAssembly.Instance(new WebAssembly.Module(bin)).exports.fn;

  const argument = { "element": "anyfunc", "initial": 1 };
  const table = new WebAssembly.Table(argument, fn);

  assert_equals(table.get(0), fn);
  table.set(0);
  assert_equals(table.get(0), null);

  table.set(0, fn);
  assert_equals(table.get(0), fn);

  assert_throws_js(TypeError, () => table.set(0, {}));
  assert_throws_js(TypeError, () => table.set(0, 37));
}, "Arguments for anyfunc table set");

test(() => {
  const testObject = {};
  const argument = { "element": "externref", "initial": 1 };
  const table = new WebAssembly.Table(argument, testObject);

  assert_equals(table.get(0), testObject);
  table.set(0);
  assert_equals(table.get(0), undefined);

  table.set(0, testObject);
  assert_equals(table.get(0), testObject);

  table.set(0, 37);
  assert_equals(table.get(0), 37);
}, "Arguments for externref table set");
