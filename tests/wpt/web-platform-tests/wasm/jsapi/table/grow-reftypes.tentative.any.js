// META: global=jsshell
// META: script=assertions.js
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js

// Test cases for changes to the WebAssembly.Table.prototype.grow() API that
// come in with the reftypes proposal: the API takes a default argument, which
// for tables of anyfunc must be either an exported wasm function or null.
//
// See:
//   https://github.com/WebAssembly/reference-types
//   https://bugzilla.mozilla.org/show_bug.cgi?id=1507491
//   https://github.com/WebAssembly/reference-types/issues/22

test(() => {
  const builder = new WasmModuleBuilder();
  builder
    .addFunction("fn", kSig_v_v)
    .addBody([])
    .exportFunc();
  const bin = builder.toBuffer()
  const argument = { "element": "anyfunc", "initial": 1 };
  const table = new WebAssembly.Table(argument);
  const fn = new WebAssembly.Instance(new WebAssembly.Module(bin)).exports.fn;
  const result = table.grow(2, fn);
  assert_equals(result, 1);
  assert_equals(table.get(0), null);
  assert_equals(table.get(1), fn);
  assert_equals(table.get(2), fn);
}, "Grow with exported-function argument");

test(() => {
  const argument = { "element": "anyfunc", "initial": 1 };
  const table = new WebAssembly.Table(argument);
  assert_throws_js(TypeError, () => table.grow(2, {}));
}, "Grow with non-function argument");

test(() => {
  const argument = { "element": "anyfunc", "initial": 1 };
  const table = new WebAssembly.Table(argument);
  assert_throws_js(TypeError, () => table.grow(2, () => true));
}, "Grow with JS-function argument");
