// META: title=validation tests for WebNN API prelu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateTwoInputsFromMultipleBuilders('prelu');

const tests = [
  {
    name:
        '[prelu] Test slope\'s shape = [3, 2, 5] which is the same as input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'float32', shape: [3, 2, 5]},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'float32', shape: [5]},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'float32', shape: []},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [2, 5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'float32', shape: [2, 5]},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Throw if the shape of slope is not broadcastable to the shape of input.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'float32', shape: [2]},
  },
  {
    name:
        '[prelu] Throw if the data type of slope does not match the data type of input.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    slope: {dataType: 'int32', shape: [3, 2, 5]},
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const slope = builder.input('input', test.slope);
      if (test.output) {
        const output = builder.prelu(input, slope);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'prelu_123';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.prelu(input, slope, options), regrexp);
      }
    }, test.name));

promise_test(async t => {
  for (let dataType of allWebNNOperandDataTypes) {
    if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
      continue;
    }
    const builder = new MLGraphBuilder(context);
    const shape = [1];
    const input = builder.input(`input`, {dataType, shape});
    if (context.opSupportLimits().prelu.input.dataTypes.includes(dataType)) {
      const output = builder.prelu(input, input);
      assert_equals(output.dataType, dataType);
      assert_array_equals(output.shape, shape);
    } else {
      assert_throws_js(TypeError, () => builder.prelu(input, input));
    }
  }
}, `[prelu] Test prelu with all of the data types.`);
