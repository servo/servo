// META: title=validation tests for WebNN API reduction operation
// META: global=window,dedicatedworker
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

const kFloatRestrictReductionOperators = [
  'reduceL2',
  'reduceLogSum',
  'reduceLogSumExp',
  'reduceMean',
];

const kFloatAllowReductionOperators = [
  'reduceL1',
  'reduceMax',
  'reduceMin',
  'reduceProduct',
  'reduceSum',
  'reduceSumSquare',
];

const allReductionOperatorsTests = [
  {
    name: '[reduce] Test reduce with default options.',
    input: {dataType: 'float32', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'float32', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is float16.',
    input: {dataType: 'float16', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'float16', dimensions: []}
  },
  {
    name: '[reduce] Test reduce with keepDimensions=true.',
    input: {dataType: 'float32', dimensions: [1, 3, 4, 4]},
    options: {
      keepDimensions: true,
    },
    output: {dataType: 'float32', dimensions: [1, 1, 1, 1]}
  },
  {
    name: '[reduce] Test reduce with axes=[0, 1] and keep_dimensions=false.',
    input: {dataType: 'float32', dimensions: [1, 3, 5, 5]},
    options: {axes: [0, 1]},
    output: {dataType: 'float32', dimensions: [5, 5]}
  },
  {
    name: '[reduce] Throw if a value in axes is out of range of [0, N-1].',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [4],
    },
  },
  {
    name: '[reduce] Throw if the two values are same in axes sequence.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [0, 1, 1],
    },
  },
];

const kFloatRestrictOperatorsTests = [
  {
    name:
        '[reduce] Throw if the input data type is not one of the floating point.',
    input: {dataType: 'int32', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [0, 1],
    },
  },
];

const kFloatAllowOperatorsTests = [
  {
    name:
        '[reduce] Test when the input data type is not one of the floating point.',
    input: {dataType: 'int32', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'int32', dimensions: []}
  },
];

function runReductionTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});

      if (test.output) {
        const output = builder[operatorName](input, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder[operatorName](input, test.options));
      }
    }, test.name.replace('[reduce]', `[${operatorName}]`));
  });
}

kReductionOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
  runReductionTests(operatorName, allReductionOperatorsTests);
});

kFloatRestrictReductionOperators.forEach((operatorName) => {
  runReductionTests(operatorName, kFloatRestrictOperatorsTests);
});

kFloatAllowReductionOperators.forEach((operatorName) => {
  runReductionTests(operatorName, kFloatAllowOperatorsTests);
});
