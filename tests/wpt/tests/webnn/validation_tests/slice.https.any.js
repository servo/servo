// META: title=validation tests for WebNN API slice operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const starts = [1, 1];
  const sizes = [1, 1];
  assert_throws_js(
      TypeError, () => builder.slice(inputFromOtherBuilder, starts, sizes));
}, '[slice] throw if input is from another builder');

const tests = [
  {
    name: '[slice] Test with starts=[0, 1, 2] and sizes=[1, 2, 3].',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [0, 1, 2],
    sizes: [1, 2, 3],
    output: {dataType: 'float32', dimensions: [1, 2, 3]}
  },
  {
    name: '[slice] Throw if input is a scalar.',
    input: {dataType: 'float32', dimensions: []},
    starts: [0],
    sizes: [1]
  },
  {
    name:
        '[slice] Throw if the length of sizes is not equal to the rank of the input tensor.',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [1, 2, 3],
    sizes: [1, 1]
  },
  {
    name:
        '[slice] Throw if the length of starts is not equal to the rank of the input tensor.',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [1, 2, 1, 3],
    sizes: [1, 1, 1]
  },
  {
    name:
        '[slice] Throw if the starting index is equal to or greater than input size in the same dimension.',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [0, 4, 4],
    sizes: [1, 1, 1]
  },
  {
    name: '[slice] Throw if the number of elements to slice is equal to 0.',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [1, 2, 3],
    sizes: [1, 0, 1]
  },
  {
    name:
        '[slice] Throw if the ending index to slice is greater than input size in the same dimension.',
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    starts: [0, 1, 2],
    sizes: [3, 4, 1]
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});

      if (test.output) {
        const output = builder.slice(input, test.starts, test.sizes);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.slice(input, test.starts, test.sizes));
      }
    }, test.name));
