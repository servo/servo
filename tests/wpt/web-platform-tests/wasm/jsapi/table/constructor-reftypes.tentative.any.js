// META: global=window,dedicatedworker,jsshell
// META: script=assertions.js
// META: script=/wasm/jsapi/wasm-module-builder.js

// Test cases for changes to the WebAssembly.Table constructor API that
// come in with the reftypes proposal: the API takes a default argument, which
// is used as an initializing value for the WebAssembly.Table.
//
// See:
//   https://webassembly.github.io/reference-types/js-api/index.html#tables

test(() => {
  const testObject = {};
  const argument = { "element": "externref", "initial": 3 };
  const table = new WebAssembly.Table(argument, testObject);
  assert_equals(table.length, 3);
  assert_equals(table.get(0), testObject);
  assert_equals(table.get(1), testObject);
  assert_equals(table.get(2), testObject);
}, "initialize externref table with default value");

test(() => {
  const argument = { "element": "i32", "initial": 3 };
  assert_throws_js(TypeError, () => new WebAssembly.Table(argument));
}, "initialize table with a wrong element value");

test(() => {
  const builder = new WasmModuleBuilder();
  builder
    .addFunction("fn", kSig_v_v)
    .addBody([])
    .exportFunc();
  const bin = builder.toBuffer();
  const fn = new WebAssembly.Instance(new WebAssembly.Module(bin)).exports.fn;
  const argument = { "element": "anyfunc", "initial": 3 };
  const table = new WebAssembly.Table(argument, fn);
  assert_equals(table.length, 3);
  assert_equals(table.get(0), fn);
  assert_equals(table.get(1), fn);
  assert_equals(table.get(2), fn);
}, "initialize anyfunc table with default value");

test(() => {
  const argument = { "element": "anyfunc", "initial": 3 };
  assert_throws_js(TypeError, () => new WebAssembly.Table(argument, {}));
  assert_throws_js(TypeError, () => new WebAssembly.Table(argument, "cannot be used as a wasm function"));
  assert_throws_js(TypeError, () => new WebAssembly.Table(argument, 37));
}, "initialize anyfunc table with a bad default value");
