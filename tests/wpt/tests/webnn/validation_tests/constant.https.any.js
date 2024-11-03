// META: title=validation tests for WebNN API constant interface
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  // Tests for constant(descriptor, bufferView)
  {
    name:
        '[constant] Test building a 0-D scalar constant with empty dimensions',
    descriptor: {dataType: 'float32', shape: []},
    bufferView: {type: Float32Array, byteLength: 1 * 4},
    output: {dataType: 'float32', shape: []}
  },
  {
    name: '[constant] Test building a constant with float32 data type',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    bufferView: {type: Float32Array, byteLength: 6 * 4},
    output: {dataType: 'float32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for float32 doesn\'t match the given dimensions',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    bufferView: {
      type: Float32Array,
      byteLength: 6 * 4 - 4  // The bufferView's byte length is less than the
                             // one by given dimensions
    }
  },
  // TODO (crbug.com/329702838): Test building a constant with float16 data type
  {
    name: '[constant] Test building a constant with int32 data type',
    descriptor: {dataType: 'int32', shape: [2, 3]},
    bufferView: {type: Int32Array, byteLength: 6 * 4},
    output: {dataType: 'int32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for int32 doesn\'t match the given dimensions',
    descriptor: {dataType: 'int32', shape: [2, 3]},
    bufferView: {
      type: Int32Array,
      byteLength: 6 * 4 + 4  // The bufferView's byte length is greater than the
                             // one by given dimensions
    }
  },
  {
    name: '[constant] Test building a constant with uint32 data type',
    descriptor: {dataType: 'uint32', shape: [2, 3]},
    bufferView: {type: Uint32Array, byteLength: 6 * 4},
    output: {dataType: 'uint32', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for uint32 doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint32', shape: [2, 3]},
    bufferView: {type: Uint32Array, byteLength: 6 * 4 + 4}
  },
  {
    name: '[constant] Test building a constant with int64 data type',
    descriptor: {dataType: 'int64', shape: [2, 3]},
    bufferView: {type: BigInt64Array, byteLength: 6 * 8},
    output: {dataType: 'int64', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for int64 doesn\'t match the given dimensions',
    descriptor: {dataType: 'int64', shape: [2, 3]},
    bufferView: {type: BigInt64Array, byteLength: 6 * 8 + 8}
  },
  {
    name: '[constant] Test building a constant with uint64 data type',
    descriptor: {dataType: 'uint64', shape: [2, 3]},
    bufferView: {type: BigUint64Array, byteLength: 6 * 8},
    output: {dataType: 'uint64', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for uint64 doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint64', shape: [2, 3]},
    bufferView: {type: BigUint64Array, byteLength: 6 * 8 + 8}
  },
  {
    name: '[constant] Test building a constant with int8 data type',
    descriptor: {dataType: 'int8', shape: [2, 3]},
    bufferView: {type: Int8Array, byteLength: 6 * 1},
    output: {dataType: 'int8', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for int8 doesn\'t match the given dimensions',
    descriptor: {dataType: 'int8', shape: [2, 3]},
    bufferView: {type: Int8Array, byteLength: 6 * 4 - 4}
  },
  {
    name: '[constant] Test building a constant with uint8 data type',
    descriptor: {dataType: 'uint8', shape: [2, 3]},
    bufferView: {type: Uint8Array, byteLength: 6 * 1},
    output: {dataType: 'uint8', shape: [2, 3]}
  },
  {
    name:
        '[constant] Throw if byte length of bufferView for uint8 doesn\'t match the given dimensions',
    descriptor: {dataType: 'uint8', shape: [2, 3]},
    bufferView: {type: Uint8Array, byteLength: 6 * 4 - 4}
  },
  {
    name: '[constant] Throw if a dimension is 0',
    descriptor: {dataType: 'float32', shape: [2, 0]},
    bufferView: {type: Float32Array, byteLength: 2 * 4}
  },
  {
    name:
        '[constant] Throw if bufferView type doesn\'t match the operand data type',
    descriptor: {dataType: 'float32', shape: [2, 3]},
    bufferView: {type: Int32Array, byteLength: 6 * 4}
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const buffer = new ArrayBuffer(test.bufferView.byteLength);
      const bufferView = new test.bufferView.type(buffer);
      if (test.output) {
        const constantOperand = builder.constant(test.descriptor, bufferView);
        assert_equals(constantOperand.dataType, test.output.dataType);
        assert_array_equals(constantOperand.shape, test.output.shape);
      } else {
        assert_throws_js(
            TypeError, () => builder.constant(test.descriptor, bufferView));
      }
    }, test.name));
