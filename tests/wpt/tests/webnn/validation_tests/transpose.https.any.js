// META: title=validation tests for WebNN API transpose operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('transpose');

const label = 'transpose-2';
const tests = [
  {
    name: '[transpose] Test building transpose with permutation=[0, 2, 3, 1].',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    options: {permutation: [0, 2, 3, 1]},
    output: {dataType: 'float32', shape: [1, 3, 4, 2]}
  },
  {
    name:
        '[transpose] Throw if permutation\'s size is not the same as input\'s rank.',
    input: {dataType: 'int32', shape: [1, 2, 4]},
    options: {
      permutation: [0, 2, 3, 1],
      label: label,
    },
  },
  {
    name: '[transpose] Throw if two values in permutation are same.',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    options: {
      permutation: [0, 2, 3, 2],
      label: label,
    },
  },
  {
    name:
        '[transpose] Throw if any value in permutation is not in the range [0,input\'s rank).',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    options: {
      permutation: [0, 1, 2, 4],
      label: label,
    },
  },
  {
    name: '[transpose] Throw if any value in permutation is negative.',
    input: {dataType: 'int32', shape: [1, 2, 3, 4]},
    options: {
      permutation: [0, -1, 2, 3],
    },
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      if (test.output) {
        const output = builder.transpose(input, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const options = {...test.options};
        if (options.label) {
          const regrexp = new RegExp('\\[' + label + '\\]');
          assert_throws_with_label(
              () => builder.transpose(input, options), regrexp);
        } else {
          assert_throws_js(TypeError, () => builder.transpose(input, options));
        }
      }
    }, test.name));

promise_test(async t => {
  for (let dataType of allWebNNOperandDataTypes) {
    if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
      continue;
    }
    const builder = new MLGraphBuilder(context);
    const shape = [1, 2, 3, 4];
    const input = builder.input(`input`, {dataType, shape});
    if (context.opSupportLimits().transpose.input.dataTypes.includes(
            dataType)) {
      const output = builder.transpose(input);
      assert_equals(output.dataType, dataType);
      assert_array_equals(output.shape, [4, 3, 2, 1]);
    } else {
      assert_throws_js(TypeError, () => builder.transpose(input));
    }
  }
}, `[transpose] Test transpose with all of the data types.`);
