// META: title=validation tests for WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kArgMinMaxOperators = [
  'argMin',
  'argMax',
];

const label = 'arg_min_max_1_!';

const tests = [
  {
    name: '[argMin/Max] Test with default options.',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 0,
    output: {shape: [2, 3, 4]}
  },
  {
    name: '[argMin/Max] Test with axes=1.',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 1,
    output: {shape: [1, 3, 4]}
  },
  {
    name: '[argMin/Max] Test with outputDataType=int32',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 1,
    options: {
      outputDataType: 'int32',
    },
    output: {shape: [1, 3, 4]}
  },
  {
    name: '[argMin/Max] Test with outputDataType=int64',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 1,
    options: {
      outputDataType: 'int64',
    },
    output: {shape: [1, 3, 4]}
  },
  {
    name:
        '[argMin/Max] Throw if the value in axis is greater than or equal to input rank.',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 4,
    options: {
      label: label,
    },
  },
  {
    name: '[argMin/Max] Throw if input is a scalar and axis=0.',
    input: {dataType: 'float32', shape: []},
    axis: 0,
    options: {
      label: label,
    },
  },
];

function runTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const axis = test.axis;
      if (test.options && test.options.outputDataType !== undefined) {
        if (context.opSupportLimits()[operatorName].output.dataTypes.includes(
          test.options.outputDataType)) {
          const output = builder[operatorName](input, axis, test.options);
          assert_equals(output.dataType, test.options.outputDataType);
          assert_array_equals(output.shape, test.output.shape);
        } else {
          assert_throws_js(
            TypeError, () => builder[operatorName](input, axis, test.options));
        }
        return;
      }
      if (test.output) {
        const output = builder[operatorName](input, axis, test.options);
        assert_equals(output.dataType, 'int32');
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const regrexp = /\[arg_min_max_1_\!\]/;
        assert_throws_with_label(
            () => builder[operatorName](input, axis, test.options), regrexp);
      }
    }, test.name.replace('[argMin/Max]', `[${operatorName}]`));
  });
}

kArgMinMaxOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
  runTests(operatorName, tests);
});
