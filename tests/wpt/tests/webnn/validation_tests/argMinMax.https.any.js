// META: title=validation tests for WebNN API argMin/Max operations
// META: global=window
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
  {
    name: '[argMin/Max] Throw if outputDataType=float32',
    input: {dataType: 'float32', shape: [1, 2, 3, 4]},
    axis: 1,
    options: {outputDataType: 'float32', label: label}
  }
];

function runTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const axis = test.axis;
      if (!context.opSupportLimits()[operatorName].input.dataTypes.includes(test.input.dataType)){
        assert_throws_js(
          TypeError, () => builder[operatorName](input, axis, test.options));
        return;
      }
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

kArgMinMaxOperators.forEach((operatorName) => {
  validateOperandRank(operatorName, 'input', (builder, input) => {
    return builder[operatorName](input, 0);
  });

  // Accept every valid (rank, axis) pair where rank is in the opSupportLimits
  // rankRange for `input` and axis is in [0, rank - 1].
  promise_test(async t => {
    const opLimits = context.opSupportLimits()[operatorName];
    if (!opLimits)
      return;
    const {min: rankMin, max: rankMax} = opLimits.input.rankRange;
    const dataTypeInfo = pickSupportedDataTypeInfo(opLimits.input);
    if (!dataTypeInfo)
      return;
    for (let rank = rankMin; rank <= rankMax; rank++) {
      for (let axis = 0; axis < rank; axis++) {
        const shape = Array(rank).fill(kExampleDimSize);
        const builder = new MLGraphBuilder(context);
        const input = buildInputOperand(builder, 'input', dataTypeInfo, shape);
        const output = builder[operatorName](input, axis);
        assert_true(output instanceof MLOperand,
                    `rank ${rank} axis ${axis} should be accepted`);
      }
    }
  }, `[${operatorName}] accept every axis in [0, rank - 1] for every in-range input rank`);
});
