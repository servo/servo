// META: title=validation tests for WebNN API tile operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('tile');

const label = 'xxx-tile';
const tests = [
  {
    name:
        '[tile] Test building tile with repetitions=[1, 1, 1, 1], float32 data type.',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    repetitions: [1, 1, 1, 1],
    output: {dataType: 'float32', shape: [1, 2, 3, 4]},
    options: {
      label: label,
    },
  },
  {
    name:
        '[tile] Test building tile with repetitions=[1, 2, 3, 4], uint32 data type.',
    input: {dataType: 'uint32', shape: [1, 2, 3, 4]},
    repetitions: [1, 2, 3, 4],
    output: {dataType: 'uint32', shape: [1, 4, 9, 16]},
  },
  {
    name:
        '[tile] Throw if repetitions\'s size is not the same as input\'s rank.',
    input: {dataType: 'int32', shape: [1, 2, 4]},
    repetitions: [1, 2, 3, 4],
  },
  {
    name: '[tile] Throw if any value in repetitions is zero.',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    repetitions: [0, 1, 2, 3],
  },
  {
    name: '[tile] Throw if any value in repetitions is negative.',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    repetitions: [-1, 1, 2, 3],
  },
  {
    name:
        '[tile] Throw if any value in repetitions causes tiled dimension size overflow.',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    repetitions: [1, 1, kMaxUnsignedLong, 3],
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      if (test.output) {
        const output = builder.tile(input, test.repetitions, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const options = {...test.options};
        if (options.label) {
          const regrexp = new RegExp('\\[' + label + '\\]');
          builder.tile(input, test.repetitions, options);
          assert_throws_with_label(
              () => builder.tile(input, test.repetitions, options), regrexp);
        } else {
          assert_throws_js(
              TypeError, () => builder.tile(input, test.repetitions, options));
        }
      }
    }, test.name));
