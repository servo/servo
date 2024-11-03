// META: title=validation tests for WebNN API reduction operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kReductionOperators = [
  'reduceL1',
  'reduceL2',
  'reduceLogSum',
  'reduceLogSumExp',
  'reduceMax',
  'reduceMean',
  'reduceMin',
  'reduceProduct',
  'reduceSum',
  'reduceSumSquare',
];

const label = 'reduce_op_xxx';

const allReductionOperatorsTests = [
  {
    name: '[reduce] Test reduce with keepDimensions=true.',
    input: {dataType: 'float32', shape: [1, 3, 4, 4]},
    options: {
      keepDimensions: true,
    },
    output: {dataType: 'float32', shape: [1, 1, 1, 1]}
  },
  {
    name: '[reduce] Test reduce with axes=[0, 1] and keep_dimensions=false.',
    input: {dataType: 'float32', shape: [1, 3, 5, 5]},
    options: {axes: [0, 1]},
    output: {dataType: 'float32', shape: [5, 5]}
  },
  {
    name: '[reduce] Throw if a value in axes is out of range of [0, N-1].',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      axes: [4],
      label: label,
    },
  },
  {
    name: '[reduce] Throw if the two values are same in axes sequence.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      axes: [0, 1, 1],
      label: label,
    },
  },
];

function runReductionTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);

      if (test.output) {
        const output = builder[operatorName](input, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder[operatorName](input, test.options), regrexp);
      }
    }, test.name.replace('[reduce]', `[${operatorName}]`));
  });
}

kReductionOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
  runReductionTests(operatorName, allReductionOperatorsTests);
});

kReductionOperators.forEach((operatorName) => {
  promise_test(async t => {
    for (let dataType of allWebNNOperandDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        continue;
      }
      const builder = new MLGraphBuilder(context);
      const input = builder.input(`input`, {dataType, shape: shape3D});
      if (context.opSupportLimits()[operatorName].input.dataTypes.includes(
              dataType)) {
        const output = builder[operatorName](input);
        assert_equals(output.dataType, dataType);
        assert_array_equals(output.shape, []);
      } else {
        assert_throws_js(TypeError, () => builder[operatorName](input));
      }
    }
  }, `[${operatorName}] Test reduce with all of the data types.`);
});
