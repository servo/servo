// META: global=jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_Table(actual, expected) {
  assert_equals(Object.getPrototypeOf(actual), WebAssembly.Table.prototype,
                "prototype");
  assert_true(Object.isExtensible(actual), "extensible");

  assert_equals(actual.length, expected.length, "length");
  for (let i = 0; i < expected.length; ++i) {
    assert_equals(actual.get(i), null, `actual.get(${i})`);
  }
}

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
  const argument = { "element": "anyfunc", "initial": 0 };
  assert_throws(new TypeError(), () => WebAssembly.Table(argument));
}, "Calling");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Table({}));
}, "Empty descriptor");

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
                  () => new WebAssembly.Table(invalidArgument),
                  `new Table(${format_value(invalidArgument)})`);
  }
}, "Invalid descriptor argument");

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
  assert_throws(new RangeError(), () => new WebAssembly.Table({ "element": "anyfunc", "initial": 10, "maximum": 9 }));
}, "Initial value exceeds maximum");

test(() => {
  const argument = { "element": "anyfunc", "initial": 0 };
  const table = new WebAssembly.Table(argument);
  assert_Table(table, { "length": 0 });
}, "Basic (zero)");

test(() => {
  const argument = { "element": "anyfunc", "initial": 5 };
  const table = new WebAssembly.Table(argument);
  assert_Table(table, { "length": 5 });
}, "Basic (non-zero)");

test(() => {
  const argument = { "element": "anyfunc", "initial": 0 };
  const table = new WebAssembly.Table(argument, {});
  assert_Table(table, { "length": 0 });
}, "Stray argument");

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
  const table = new WebAssembly.Table(proxy);
  assert_Table(table, { "length": 0 });
}, "Proxy descriptor");

test(() => {
  const table = new WebAssembly.Table({
    "element": {
      toString() { return "anyfunc"; },
    },
    "initial": 1,
  });
  assert_Table(table, { "length": 1 });
}, "Type conversion for descriptor.element");

test(() => {
  const order = [];

  new WebAssembly.Table({
    get maximum() {
      order.push("maximum");
      return {
        valueOf() {
          order.push("maximum valueOf");
          return 1;
        },
      };
    },

    get initial() {
      order.push("initial");
      return {
        valueOf() {
          order.push("initial valueOf");
          return 1;
        },
      };
    },

    get element() {
      order.push("element");
      return {
        toString() {
          order.push("element toString");
          return "anyfunc";
        },
      };
    },
  });

  assert_array_equals(order, [
    "element",
    "element toString",
    "initial",
    "initial valueOf",
    "maximum",
    "maximum valueOf",
  ]);
}, "Order of evaluation for descriptor");
