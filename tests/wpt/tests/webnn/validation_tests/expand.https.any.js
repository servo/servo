// META: title=validation tests for WebNN API expand operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 1, 2]});

  const newShape = [2, 2, 2];
  assert_throws_js(
      TypeError, () => builder.expand(inputFromOtherBuilder, newShape));
}, '[expand] throw if input is from another builder');

const tests = [
  {
    name: '[expand] Test with 0-D scalar to 3-D tensor.',
    input: {dataType: 'float32', dimensions: []},
    newShape: [3, 4, 5],
    output: {dataType: 'float32', dimensions: [3, 4, 5]}
  },
  {
    name: '[expand] Test with the new shapes that are the same as input.',
    input: {dataType: 'float32', dimensions: [4]},
    newShape: [4],
    output: {dataType: 'float32', dimensions: [4]}
  },
  {
    name: '[expand] Test with the new shapes that are broadcastable.',
    input: {dataType: 'int32', dimensions: [3, 1, 5]},
    newShape: [3, 4, 5],
    output: {dataType: 'int32', dimensions: [3, 4, 5]}
  },
  {
    name:
        '[expand] Test with the new shapes that are broadcastable and the rank of new shapes is larger than input.',
    input: {dataType: 'int32', dimensions: [2, 5]},
    newShape: [3, 2, 5],
    output: {dataType: 'int32', dimensions: [3, 2, 5]}
  },
  {
    name:
        '[expand] Throw if the input shapes are the same rank but not broadcastable.',
    input: {dataType: 'uint32', dimensions: [3, 6, 2]},
    newShape: [4, 3, 5],
  },
  {
    name: '[expand] Throw if the input shapes are not broadcastable.',
    input: {dataType: 'uint32', dimensions: [5, 4]},
    newShape: [5],
  },
  {
    name: '[expand] Throw if the number of new shapes is too large.',
    input: {dataType: 'float32', dimensions: [1, 2, 1, 1]},
    newShape: [1, 2, kMaxUnsignedLong, kMaxUnsignedLong],
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const options = {};
      if (test.axis) {
        options.axis = test.axis;
      }

      if (test.output) {
        const output = builder.expand(input, test.newShape);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder.expand(input, test.newShape));
      }
    }, test.name));
