// META: global=jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_Global(actual, expected) {
  assert_equals(Object.getPrototypeOf(actual), WebAssembly.Global.prototype,
                "prototype");
  assert_true(Object.isExtensible(actual), "extensible");

  assert_equals(actual.value, expected, "value");
  assert_equals(actual.valueOf(), expected, "valueOf");
}

test(() => {
  assert_function_name(WebAssembly.Global, "Global", "WebAssembly.Global");
}, "name");

test(() => {
  assert_function_length(WebAssembly.Global, 1, "WebAssembly.Global");
}, "length");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Global());
}, "No arguments");

test(() => {
  const argument = { "value": "i32" };
  assert_throws(new TypeError(), () => WebAssembly.Global(argument));
}, "Calling");

test(() => {
  const order = [];

  new WebAssembly.Global({
    get value() {
      order.push("descriptor value");
      return {
        toString() {
          order.push("descriptor value toString");
          return "f64";
        },
      };
    },

    get mutable() {
      order.push("descriptor mutable");
      return false;
    },
  }, {
    valueOf() {
      order.push("value valueOf()");
    }
  });

  assert_array_equals(order, [
    "descriptor mutable",
    "descriptor value",
    "descriptor value toString",
    "value valueOf()",
  ]);
}, "Order of evaluation");

test(() => {
  const invalidArguments = [
    undefined,
    null,
    false,
    true,
    "",
    "test",
    Symbol(),
    1,
    NaN,
    {},
  ];
  for (const invalidArgument of invalidArguments) {
    assert_throws(new TypeError(),
                  () => new WebAssembly.Global(invalidArgument),
                  `new Global(${format_value(invalidArgument)})`);
  }
}, "Invalid descriptor argument");

test(() => {
  const invalidTypes = ["i16", "i128", "f16", "f128", "u32", "u64", "i32\0"];
  for (const value of invalidTypes) {
    const argument = { value };
    assert_throws(new TypeError(), () => new WebAssembly.Global(argument));
  }
}, "Invalid type argument");

test(() => {
  const argument = { "value": "i64" };
  const global = new WebAssembly.Global(argument);
  assert_throws(new TypeError(), () => global.value);
  assert_throws(new TypeError(), () => global.valueOf());
}, "i64 with default");

for (const type of ["i32", "f32", "f64"]) {
  test(() => {
    const argument = { "value": type };
    const global = new WebAssembly.Global(argument);
    assert_Global(global, 0);
  }, `Default value for type ${type}`);

  const valueArguments = [
    [undefined, 0],
    [null, 0],
    [true, 1],
    [false, 0],
    [2, 2],
    ["3", 3],
    [{ toString() { return "5" } }, 5, "object with toString"],
    [{ valueOf() { return "8" } }, 8, "object with valueOf"],
  ];
  for (const [value, expected, name = format_value(value)] of valueArguments) {
    test(() => {
      const argument = { "value": type };
      const global = new WebAssembly.Global(argument, value);
      assert_Global(global, expected);
    }, `Explicit value ${name} for type ${type}`);
  }
}

test(() => {
  const argument = { "value": "i32" };
  const global = new WebAssembly.Global(argument, 0, {});
  assert_Global(global, 0);
}, "Stray argument");
