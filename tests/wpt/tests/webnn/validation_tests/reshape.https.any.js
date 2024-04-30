// META: title=validation tests for WebNN API reshape operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});

  const newShape = [3, 2, 1];
  assert_throws_js(
      TypeError, () => builder.reshape(inputFromOtherBuilder, newShape));
}, '[reshape] throw if input is from another builder');

const tests = [
  {
    name: '[reshape] Test with new shape=[3, 8].',
    input: {dataType: 'float32', dimensions: [2, 3, 4]},
    newShape: [3, 8],
    output: {dataType: 'float32', dimensions: [3, 8]}
  },
  {
    name: '[reshape] Test with new shape=[24], src shape=[2, 3, 4].',
    input: {dataType: 'float32', dimensions: [2, 3, 4]},
    newShape: [24],
    output: {dataType: 'float32', dimensions: [24]}
  },
  {
    name: '[reshape] Test with new shape=[1], src shape=[1].',
    input: {dataType: 'float32', dimensions: [1]},
    newShape: [1],
    output: {dataType: 'float32', dimensions: [1]}
  },
  {
    name: '[reshape] Test reshaping a 1-D 1-element tensor to scalar.',
    input: {dataType: 'float32', dimensions: [1]},
    newShape: [],
    output: {dataType: 'float32', dimensions: []}
  },
  {
    name: '[reshape] Test reshaping a scalar to 1-D 1-element tensor.',
    input: {dataType: 'float32', dimensions: []},
    newShape: [1],
    output: {dataType: 'float32', dimensions: [1]}
  },
  {
    name: '[reshape] Throw if one value of new shape is 0.',
    input: {dataType: 'float32', dimensions: [2, 4]},
    newShape: [2, 4, 0],
  },
  {
    name:
        '[reshape] Throw if the number of elements implied by new shape is not equal to the number of elements in the input tensor when new shape=[].',
    input: {dataType: 'float32', dimensions: [2, 3, 4]},
    newShape: [],
  },
  {
    name:
        '[reshape] Throw if the number of elements implied by new shape is not equal to the number of elements in the input tensor.',
    input: {dataType: 'float32', dimensions: [2, 3, 4]},
    newShape: [3, 9],
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      if (test.output) {
        const output = builder.reshape(input, test.newShape);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.reshape(input, test.newShape));
      }
    }, test.name));
