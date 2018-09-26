// META: global=jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_Memory(memory, expected) {
  assert_equals(Object.getPrototypeOf(memory), WebAssembly.Memory.prototype,
                "prototype");
  assert_true(Object.isExtensible(memory), "extensible");

  // https://github.com/WebAssembly/spec/issues/840
  assert_equals(memory.buffer, memory.buffer, "buffer should be idempotent");
  assert_equals(Object.getPrototypeOf(memory.buffer), ArrayBuffer.prototype,
                "prototype of buffer");
  assert_true(Object.isExtensible(memory.buffer), "buffer extensibility");
  assert_equals(memory.buffer.byteLength, 0x10000 * expected.size, "size of buffer");
  if (expected.size > 0) {
    const array = new Uint8Array(memory.buffer);
    assert_equals(array[0], 0, "first element of buffer");
    assert_equals(array[array.byteLength - 1], 0, "last element of buffer");
  }
}

test(() => {
  assert_function_name(WebAssembly.Memory, "Memory", "WebAssembly.Memory");
}, "name");

test(() => {
  assert_function_length(WebAssembly.Memory, 1, "WebAssembly.Memory");
}, "length");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Memory());
}, "No arguments");

test(() => {
  const argument = { "initial": 0 };
  assert_throws(new TypeError(), () => WebAssembly.Memory(argument));
}, "Calling");

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
                  () => new WebAssembly.Memory(invalidArgument),
                  `new Memory(${format_value(invalidArgument)})`);
  }
}, "Invalid descriptor argument");

test(() => {
  assert_throws(new TypeError(), () => new WebAssembly.Memory({ "initial": undefined }));
}, "Undefined initial value in descriptor");

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
    assert_throws(new TypeError(), () => new WebAssembly.Memory({ "initial": value }));
  }, `Out-of-range initial value in descriptor: ${format_value(value)}`);

  test(() => {
    assert_throws(new TypeError(), () => new WebAssembly.Memory({ "initial": 0, "maximum": value }));
  }, `Out-of-range maximum value in descriptor: ${format_value(value)}`);
}

test(() => {
  assert_throws(new RangeError(), () => new WebAssembly.Memory({ "element": "anyfunc", "initial": 10, "maximum": 9 }));
}, "Initial value exceeds maximum");

test(() => {
  const proxy = new Proxy({}, {
    has(o, x) {
      assert_unreached(`Should not call [[HasProperty]] with ${x}`);
    },
    get(o, x) {
      return 0;
    },
  });
  new WebAssembly.Memory(proxy);
}, "Proxy descriptor");

test(() => {
  const order = [];

  new WebAssembly.Memory({
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
  });

  assert_array_equals(order, [
    "initial",
    "initial valueOf",
    "maximum",
    "maximum valueOf",
  ]);
}, "Order of evaluation for descriptor");

test(() => {
  const argument = { "initial": 0 };
  const memory = new WebAssembly.Memory(argument);
  assert_Memory(memory, { "size": 0 });
}, "Zero initial");

test(() => {
  const argument = { "initial": 4 };
  const memory = new WebAssembly.Memory(argument);
  assert_Memory(memory, { "size": 4 });
}, "Non-zero initial");

test(() => {
  const argument = { "initial": 0 };
  const memory = new WebAssembly.Memory(argument, {});
  assert_Memory(memory, { "size": 0 });
}, "Stray argument");
