// META: global=jsshell
// META: script=/wasm/jsapi/assertions.js
// META: script=/wasm/jsapi/table/assertions.js

test(() => {
  const argument = { "element": "anyfunc", "initial": 0, "minimum": 0 };
  assert_throws_js(TypeError, () => WebAssembly.Table(argument));
}, "Supplying both initial and minimum");

test(() => {
  const argument = { "element": "anyfunc", "minimum": 0 };
  const table = new WebAssembly.Table(argument);
  assert_Table(table, { "length": 0 });
}, "Basic (zero, minimum)");

test(() => {
  const argument = { "element": "anyfunc", "minimum": 5 };
  const table = new WebAssembly.Table(argument);
  assert_Table(table, { "length": 5 });
}, "Basic (non-zero, minimum)");