// META: title=validation tests for WebNN API gather operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[gather] Test gather with default options and 0-D indices',
    input: {dataType: 'int32', dimensions: [3]},
    indices: {dataType: 'int64', dimensions: []},
    output: {dataType: 'int32', dimensions: []}
  },
  {
    name: '[gather] Test gather with axis = 2',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'int64', dimensions: [5, 6]},
    axis: 2,
    output: {dataType: 'float32', dimensions: [1, 2, 5, 6, 4]}
  },
  {
    name: '[gather] Test gather with indices\'s dataType = uint32',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'uint32', dimensions: [5, 6]},
    axis: 2,
    output: {dataType: 'float32', dimensions: [1, 2, 5, 6, 4]}
  },
  {
    name: '[gather] Test gather with indices\'s dataType = int32',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'int32', dimensions: [5, 6]},
    axis: 2,
    output: {dataType: 'float32', dimensions: [1, 2, 5, 6, 4]}
  },
  {
    name: '[gather] TypeError is expected if the input is a scalar',
    input: {dataType: 'float16', dimensions: []},
    indices: {dataType: 'int64', dimensions: [1]}
  },
  {
    name:
        '[gather] TypeError is expected if the axis is greater than the rank of input',
    input: {dataType: 'float16', dimensions: [1, 2, 3]},
    indices: {dataType: 'int32', dimensions: [5, 6]},
    axis: 4
  },
  {
    name:
        '[gather] TypeError is expected if the data type of indices is float32 which is invalid',
    input: {dataType: 'float16', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'float32', dimensions: [5, 6]}
  },
  {
    name:
        '[gather] TypeError is expected if the data type of indices is uint64 which is invalid',
    input: {dataType: 'float16', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'uint64', dimensions: [5, 6]}
  }
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const indices = builder.input(
          'indices',
          {dataType: test.indices.dataType, dimensions: test.indices.dimensions});

      const options = {};
      if (test.axis) {
        options.axis = test.axis;
      }

      if (test.output) {
        const output = builder.gather(input, indices, options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.gather(input, indices, options));
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const indices =
      builder.input('indices', {dataType: 'int64', dimensions: [2, 2]});
  assert_throws_js(
      TypeError, () => builder.gather(inputFromOtherBuilder, indices));
}, '[gather] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const indicesFromOtherBuilder =
      otherBuilder.input('indices', {dataType: 'int64', dimensions: [2, 2]});

  const input =
      builder.input('input', {dataType: 'float32', dimensions: [2, 2]});
  assert_throws_js(
      TypeError, () => builder.gather(input, indicesFromOtherBuilder));
}, '[gather] throw if indices is from another builder');
