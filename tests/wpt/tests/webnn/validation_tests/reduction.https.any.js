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

const kFloatRestrictReductionOperators = [
  'reduceL2',
  'reduceLogSum',
  'reduceLogSumExp',
  'reduceMean',
];

const kFloatInt32Uint32Int64Uint64RestrictReductionOperators = [
  'reduceL1',
  'reduceProduct',
  'reduceSum',
  'reduceSumSquare',
];

const kNoTypeRestrictReductionOperators = [
  'reduceMax',
  'reduceMin',
];

const label = 'reduce_op_xxx';

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
      label: label,
    },
  },
  {
    name: '[reduce] Throw if the two values are same in axes sequence.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [0, 1, 1],
      label: label,
    },
  },
];

const kFloatRestrictOperatorsTests = [
  {
    name: '[reduce] Throw if the input data type is int32.',
    input: {dataType: 'int32', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [0, 1],
      label: label,
    },
  },
];

const kFloatInt32Uint32Int64Uint64RestrictOperatorsTests = [
  {
    name: '[reduce] Test reduce when input\'s datatype is int32.',
    input: {dataType: 'int32', dimensions: [1, 2, 5, 5]},
    output: {dataType: 'int32', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is uint32.',
    input: {dataType: 'uint32', dimensions: [1, 2, 5, 5]},
    output: {dataType: 'uint32', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is int64.',
    input: {dataType: 'int64', dimensions: [1, 2, 5, 5]},
    output: {dataType: 'int64', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is uint64.',
    input: {dataType: 'uint64', dimensions: [1, 2, 5, 5]},
    output: {dataType: 'uint64', dimensions: []}
  },
  {
    name:
        '[reduce] Throw if the input data type is not one of the {float32, float16, int32, uint32, int64, uint64}.',
    input: {dataType: 'int8', dimensions: [1, 2, 5, 5]},
    options: {
      axes: [0, 1],
      label: label,
    },
  },
];

const kNoTypeRestrictOperatorsTests = [
  {
    name: '[reduce] Test reduce when input\'s datatype is int64.',
    input: {dataType: 'int64', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'int64', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is uint64.',
    input: {dataType: 'uint64', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'uint64', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is int8.',
    input: {dataType: 'int8', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'int8', dimensions: []}
  },
  {
    name: '[reduce] Test reduce when input\'s datatype is uint8.',
    input: {dataType: 'uint8', dimensions: [1, 3, 4, 4]},
    output: {dataType: 'uint8', dimensions: []}
  },
];

function runReductionTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});

      if (test.output) {
        const output = builder[operatorName](input, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
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

kFloatRestrictReductionOperators.forEach((operatorName) => {
  runReductionTests(operatorName, kFloatRestrictOperatorsTests);
});

kFloatInt32Uint32Int64Uint64RestrictReductionOperators.forEach(
    (operatorName) => {
      runReductionTests(
          operatorName, kFloatInt32Uint32Int64Uint64RestrictOperatorsTests);
    });

kNoTypeRestrictReductionOperators.forEach((operatorName) => {
  runReductionTests(operatorName, kNoTypeRestrictOperatorsTests);
});
