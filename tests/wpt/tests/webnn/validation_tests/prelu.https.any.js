// META: title=validation tests for WebNN API prelu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateTwoInputsFromMultipleBuilders('prelu');

const tests = [
  {
    name:
        '[prelu] Test slope\'s shape = [3, 2, 5] which is the same as input\'s shape.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'float32', dimensions: [3, 2, 5]},
    output: {dataType: 'float32', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'float32', dimensions: [5]},
    output: {dataType: 'float32', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'float32', dimensions: []},
    output: {dataType: 'float32', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test slope\'s shape = [2, 5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'float32', dimensions: [2, 5]},
    output: {dataType: 'float32', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test with  input\'s dataType = int32 and slope\'s dataType = int32.',
    input: {dataType: 'int32', dimensions: [3, 2, 5]},
    slope: {dataType: 'int32', dimensions: [2, 5]},
    output: {dataType: 'int32', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Test with  input\'s dataType = int8 and slope\'s dataType = int8.',
    input: {dataType: 'int8', dimensions: [3, 2, 5]},
    slope: {dataType: 'int8', dimensions: [2, 5]},
    output: {dataType: 'int8', dimensions: [3, 2, 5]},
  },
  {
    name:
        '[prelu] Throw if the shape of slope is not broadcastable to the shape of input.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'float32', dimensions: [2]},
  },
  {
    name:
        '[prelu] Throw if the data type of slope does not match the data type of input.',
    input: {dataType: 'float32', dimensions: [3, 2, 5]},
    slope: {dataType: 'int32', dimensions: [3, 2, 5]},
  },
  {
    name: '[prelu] Throw if the data type of input is int64.',
    input: {dataType: 'int64', dimensions: [3, 2, 5]},
    slope: {dataType: 'int64', dimensions: [3, 2, 5]},
  },
  {
    name: '[prelu] Throw if the data type of input is uint32.',
    input: {dataType: 'uint32', dimensions: [3, 2, 5]},
    slope: {dataType: 'uint32', dimensions: [3, 2, 5]},
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const slope = builder.input(
          'input',
          {dataType: test.slope.dataType, dimensions: test.slope.dimensions});
      if (test.output) {
        const output = builder.prelu(input, slope);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder.prelu(input, slope));
      }
    }, test.name));
