// META: global=window,dedicatedworker,jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_Memory(memory, expected) {
  assert_equals(Object.getPrototypeOf(memory), WebAssembly.Memory.prototype,
                "prototype");
  assert_true(Object.isExtensible(memory), "extensible");

  // https://github.com/WebAssembly/spec/issues/840
  assert_equals(memory.buffer, memory.buffer, "buffer should be idempotent");
  const isShared = !!expected.shared;
  const bufferType = isShared ? self.SharedArrayBuffer : ArrayBuffer;
  assert_equals(Object.getPrototypeOf(memory.buffer), bufferType.prototype,
                'prototype of buffer');
  assert_equals(memory.buffer.byteLength, 0x10000 * expected.size, "size of buffer");
  if (expected.size > 0) {
    const array = new Uint8Array(memory.buffer);
    assert_equals(array[0], 0, "first element of buffer");
    assert_equals(array[array.byteLength - 1], 0, "last element of buffer");
  }
  assert_equals(isShared, Object.isFrozen(memory.buffer), "buffer frozen");
  assert_not_equals(Object.isExtensible(memory.buffer), isShared, "buffer extensibility");
}

test(() => {
  assert_function_name(WebAssembly.Memory, "Memory", "WebAssembly.Memory");
}, "name");

test(() => {
  assert_function_length(WebAssembly.Memory, 1, "WebAssembly.Memory");
}, "length");

test(() => {
  assert_throws_js(TypeError, () => new WebAssembly.Memory());
}, "No arguments");

test(() => {
  const argument = { "initial": 0 };
  assert_throws_js(TypeError, () => WebAssembly.Memory(argument));
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
    assert_throws_js(TypeError,
                     () => new WebAssembly.Memory(invalidArgument),
                     `new Memory(${format_value(invalidArgument)})`);
  }
}, "Invalid descriptor argument");

test(() => {
  assert_throws_js(TypeError, () => new WebAssembly.Memory({ "initial": undefined }));
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
    assert_throws_js(TypeError, () => new WebAssembly.Memory({ "initial": value }));
  }, `Out-of-range initial value in descriptor: ${format_value(value)}`);

  test(() => {
    assert_throws_js(TypeError, () => new WebAssembly.Memory({ "initial": 0, "maximum": value }));
  }, `Out-of-range maximum value in descriptor: ${format_value(value)}`);
}

test(() => {
  assert_throws_js(RangeError, () => new WebAssembly.Memory({ "initial": 10, "maximum": 9 }));
}, "Initial value exceeds maximum");

test(() => {
  assert_throws_js(TypeError, () => new WebAssembly.Memory({ "initial": 10, "shared": true }));
}, "Shared memory without maximum");

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

test(t => {
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

    get shared() {
      order.push("shared");
      return {
        valueOf: t.unreached_func("should not call shared valueOf"),
      };
    },
  });

  assert_array_equals(order, [
    "initial",
    "initial valueOf",
    "maximum",
    "maximum valueOf",
    "shared",
  ]);
}, "Order of evaluation for descriptor (with shared)");

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

test(() => {
  const argument = { "initial": 4, "maximum": 10, shared: true };
  const memory = new WebAssembly.Memory(argument);
  assert_Memory(memory, { "size": 4, "shared": true });
}, "Shared memory");
