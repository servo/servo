// META: global=jsshell
// META: script=/wasm/jsapi/wasm-constants.js
// META: script=/wasm/jsapi/wasm-module-builder.js
// META: script=/wasm/jsapi/assertions.js

let emptyModuleBinary;
setup(() => {
  emptyModuleBinary = new WasmModuleBuilder().toBuffer();
});

test(() => {
  assert_function_name(WebAssembly.Table, "Table", "WebAssembly.Table");
}, "name");

test(() => {
  assert_function_length(WebAssembly.Table, 1, "WebAssembly.Table");
}, "length");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Table());
}, "No arguments");

test(() => {
  const argument = { "initial": 0 };
  assert_throws(new TypeError(), () => WebAssembly.Table(argument));
}, "Calling");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Table({}));
}, "Empty descriptor");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Table({ "element": "anyfunc", "initial": undefined }));
}, "Undefined initial value in descriptor");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Table({ "element": undefined, "initial": 0 }));
}, "Undefined element value in descriptor");

const outOfRangeValues = [
  NaN,
  Infinity,
  -Infinity,
  -1,
  0x100000000,
  0x1000000000,
];

for (const value of outOfRangeValues) {
  test(() => {
    assert_throws(new TypeError(), () => new WebAssembly.Table({ "element": "anyfunc", "initial": value }));
  }, `Out-of-range initial value in descriptor: ${format_value(value)}`);

  test(() => {
    assert_throws(new TypeError(), () => new WebAssembly.Table({ "element": "anyfunc", "initial": 0, "maximum": value }));
  }, `Out-of-range maximum value in descriptor: ${format_value(value)}`);
}

test(() => {
  const proxy = new Proxy({}, {
    has(o, x) {
      assert_unreached(`Should not call [[HasProperty]] with ${x}`);
    },
    get(o, x) {
      switch (x) {
      case "element":
        return "anyfunc";
      case "initial":
      case "maximum":
        return 0;
      default:
        return undefined;
      }
    },
  });
  new WebAssembly.Table(proxy);
}, "Proxy descriptor");

test(() => {
  const argument = { "element": "anyfunc", "initial": 0 };
  const table = new WebAssembly.Table(argument);
  assert_equals(Object.getPrototypeOf(table), WebAssembly.Table.prototype);
}, "Prototype");
