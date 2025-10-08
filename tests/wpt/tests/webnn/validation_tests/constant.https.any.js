// META: title=validation tests for WebNN API constant interface
// META: global=window,worker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  // Tests for constant(descriptor, buffer)
  {
    name:
        '[constant] Test building a 0-D scalar constant with empty dimensions',
    descriptor: {dataType: 'float32', shape: []},
    buffer: {type: Float32Array, byteLength: 1 * 4},
    output: {dataType: 'float32', shape: []}
  },
  {
    name: '[constant] Test building a constant with float32 data type',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    buffer: {type: Float32Array, byteLength: 6 * 4},
    output: {dataType: 'float32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of float32 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    buffer: {
      type: Float32Array,
      byteLength: 6 * 4 - 4  // The buffer's byte length is less than the
                             // one by given dimensions
    }
  },
  {
    name: '[constant] Test building a constant with float16 data type',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {type: Float16Array, byteLength: 6 * 2},
    output: {dataType: 'float16', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of float16 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {
      type: Float16Array,
      byteLength: 6 * 2 - 2  // The buffer's byte length is less than the
                             // one by given dimensions
    }
  },
  {
    name:
        '[constant] Throw if using Float32Array buffer for float16 operand data type',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {
      type: Float32Array,
      byteLength: 6 * 4,
    },
    viewTestOnly: true
  },
  {
    name:
        '[constant] Throw if using Int16Array buffer for float16 operand data type',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {
      type: Int16Array,
      byteLength: 6 * 2,
    },
    viewTestOnly: true
  },

  // TODO(crbug.com/399459942): remove below two Uint16Array buffer tests for
  // float16 data type when implementation removes it.
  {
    name:
        '[constant] Test building a constant with float16 data type using Uint16Array buffer',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {type: Uint16Array, byteLength: 6 * 2},
    output: {dataType: 'float16', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of float16 buffer (using Uint16Array buffer) doesn\'t match the given dimensions',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {
      type: Uint16Array,
      byteLength: 6 * 2 - 2  // The buffer's byte length is less than the
                             // one by given dimensions
    }
  },
  {
    name: '[constant] Test building a constant with int32 data type',
    descriptor: {dataType: 'int32', shape: [2, 3]},
    buffer: {type: Int32Array, byteLength: 6 * 4},
    output: {dataType: 'int32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of int32 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'int32', shape: [2, 3]},
    buffer: {
      type: Int32Array,
      byteLength: 6 * 4 + 4  // The buffer's byte length is greater than the
                             // one by given dimensions
    }
  },
  {
    name: '[constant] Test building a constant with uint32 data type',
    descriptor: {dataType: 'uint32', shape: [2, 3]},
    buffer: {type: Uint32Array, byteLength: 6 * 4},
    output: {dataType: 'uint32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of uint32 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint32', shape: [2, 3]},
    buffer: {type: Uint32Array, byteLength: 6 * 4 + 4}
  },
  {
    name: '[constant] Test building a constant with int64 data type',
    descriptor: {dataType: 'int64', shape: [2, 3]},
    buffer: {type: BigInt64Array, byteLength: 6 * 8},
    output: {dataType: 'int64', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of int64 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'int64', shape: [2, 3]},
    buffer: {type: BigInt64Array, byteLength: 6 * 8 + 8}
  },
  {
    name: '[constant] Test building a constant with uint64 data type',
    descriptor: {dataType: 'uint64', shape: [2, 3]},
    buffer: {type: BigUint64Array, byteLength: 6 * 8},
    output: {dataType: 'uint64', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of uint64 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint64', shape: [2, 3]},
    buffer: {type: BigUint64Array, byteLength: 6 * 8 + 8}
  },
  {
    name: '[constant] Test building a constant with int8 data type',
    descriptor: {dataType: 'int8', shape: [2, 3]},
    buffer: {type: Int8Array, byteLength: 6 * 1},
    output: {dataType: 'int8', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of int8 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'int8', shape: [2, 3]},
    buffer: {type: Int8Array, byteLength: 6 * 4 - 4}
  },
  {
    name: '[constant] Test building a constant with uint8 data type',
    descriptor: {dataType: 'uint8', shape: [2, 3]},
    buffer: {type: Uint8Array, byteLength: 6 * 1},
    output: {dataType: 'uint8', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of uint8 buffer doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint8', shape: [2, 3]},
    buffer: {type: Uint8Array, byteLength: 6 * 4 - 4}
  },
  {
    name: '[constant] Throw if a dimension is 0',
    descriptor: {dataType: 'float32', shape: [2, 0]},
    buffer: {type: Float32Array, byteLength: 2 * 4}
  },
  {
    name:
        '[constant] Throw if using Int32Array buffer for float32 operand data type',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    buffer: {type: Int32Array, byteLength: 6 * 4},
    viewTestOnly: true
  },
  {
    name:
        '[constant] Throw if the operand data type isn\'t of type MLOperandDataType',
    descriptor: {dataType: 'int16', shape: [2, 3]},
    buffer: {type: Int16Array, byteLength: 6 * 2}
  },
  {
    name:
        '[constant] Uint8Array should be allowed for float32 operand data type',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    buffer: {type: Uint8Array, byteLength: 6 * 4},
    output: {dataType: 'float32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Uint8Array should be allowed for float16 operand data type',
    descriptor: {dataType: 'float16', shape: [2, 3]},
    buffer: {type: Uint8Array, byteLength: 6 * 2},
    output: {dataType: 'float16', shape: [2, 3]}
  },
  {
    name: '[constant] Uint8Array should be allowed for int32 operand data type',
    descriptor: {dataType: 'int32', shape: [2, 3]},
    buffer: {type: Uint8Array, byteLength: 6 * 4},
    output: {dataType: 'int32', shape: [2, 3]}
  }
];

// Tests for constant(type, value)
const scalarTests = [
  {
    name: '[constant] Test building a scalar constant with float32 data type',
    dataType: 'float32',
    value: 3.14,
    output: {dataType: 'float32'}
  },
  {
    name: '[constant] Test building a scalar constant with float16 data type',
    dataType: 'float16',
    value: 2.5,
    output: {dataType: 'float16'}
  },
  {
    name: '[constant] Test building a scalar constant with int32 data type',
    dataType: 'int32',
    value: 42,
    output: {dataType: 'int32'}
  },
  {
    name: '[constant] Test building a scalar constant with uint32 data type',
    dataType: 'uint32',
    value: 123,
    output: {dataType: 'uint32'}
  },
  {
    name: '[constant] Test building a scalar constant with max safe integer as BigInt',
    dataType: 'int64',
    value: 9007199254740991n,
    output: {dataType: 'int64'}
  },
  {
    name: '[constant] Test building a scalar constant with max uint64 as BigInt',
    dataType: 'uint64',
    value: 18446744073709551615n,
    output: {dataType: 'uint64'}
  },
  {
    name: '[constant] Test building a scalar constant with int8 data type',
    dataType: 'int8',
    value: -128,
    output: {dataType: 'int8'}
  },
  {
    name: '[constant] Test building a scalar constant with uint8 data type',
    dataType: 'uint8',
    value: 255,
    output: {dataType: 'uint8'}
  },
  {
    name: '[constant] Test building a scalar constant with zero value',
    dataType: 'float32',
    value: 0.0,
    output: {dataType: 'float32'}
  },
  {
    name: '[constant] Test building a scalar constant with negative value',
    dataType: 'int32',
    value: -42,
    output: {dataType: 'int32'}
  },
  {
    name: '[constant] Test building a scalar constant with large float32 value',
    dataType: 'float32',
    value: 3.4028235e+38,
    output: {dataType: 'float32'}
  },
  {
    name: '[constant] Test building a scalar constant with small float32 value',
    dataType: 'float32',
    value: 1.175494e-38,
    output: {dataType: 'float32'}
  },
  {
    name: '[constant] Throw if building a scalar constant with int4 data type',
    dataType: 'int4',
    value: -2
  },
  {
    name: '[constant] Throw if building a scalar constant with uint4 data type',
    dataType: 'uint4',
    value: 2
  },
  {
    name: '[constant] Throw if using operand data type that isn\'t of type MLOperandDataType',
    dataType: 'int16',
    value: 123
  },
  {
    name: '[constant] Throw if using BigInt value for float32 data type',
    dataType: 'float32',
    value: 123n
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const buffer = new ArrayBuffer(test.buffer.byteLength);
      const bufferView = new test.buffer.type(buffer);

      if (test.viewTestOnly === undefined || test.viewTestOnly === false) {
        // Test building constant from ArrayBuffer.
        if (test.output) {
          const constantOperand = builder.constant(test.descriptor, buffer);
          assert_equals(constantOperand.dataType, test.output.dataType);
          assert_array_equals(constantOperand.shape, test.output.shape);
        } else {
          assert_throws_js(
              TypeError, () => builder.constant(test.descriptor, buffer));
        }
        if ('SharedArrayBuffer' in globalThis) {
          // Test building constant from SharedArrayBuffer.
          const sharedBuffer = new SharedArrayBuffer(test.buffer.byteLength);
          if (test.output) {
            const constantOperand =
                builder.constant(test.descriptor, sharedBuffer);
            assert_equals(constantOperand.dataType, test.output.dataType);
            assert_array_equals(constantOperand.shape, test.output.shape);
          } else {
            assert_throws_js(
                TypeError,
                () => builder.constant(test.descriptor, sharedBuffer));
          }
        }
      }

      // Test building constant from ArrayBufferView.
      if (test.output) {
        const constantOperand = builder.constant(test.descriptor, bufferView);
        assert_equals(constantOperand.dataType, test.output.dataType);
        assert_array_equals(constantOperand.shape, test.output.shape);
      } else {
        assert_throws_js(
            TypeError, () => builder.constant(test.descriptor, bufferView));
      }
      if ('SharedArrayBuffer' in globalThis) {
        // Test building constant from shared ArrayBufferView.
        const sharedBuffer = new SharedArrayBuffer(test.buffer.byteLength);
        const sharedBufferView = new test.buffer.type(sharedBuffer);
        if (test.output) {
          const constantOperand =
              builder.constant(test.descriptor, sharedBufferView);
          assert_equals(constantOperand.dataType, test.output.dataType);
          assert_array_equals(constantOperand.shape, test.output.shape);
        } else {
          assert_throws_js(
              TypeError,
              () => builder.constant(test.descriptor, sharedBufferView));
        }
      }
    }, test.name));

// Test scalar constant cases
scalarTests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);

      if (test.output) {
        // Test successful case
        const constantOperand = builder.constant(test.dataType, test.value);
        assert_equals(constantOperand.dataType, test.output.dataType);
        assert_array_equals(constantOperand.shape, []);
      } else {
        // Test error case
        assert_throws_js(
            TypeError, () => builder.constant(test.dataType, test.value));
      }
    }, test.name));
