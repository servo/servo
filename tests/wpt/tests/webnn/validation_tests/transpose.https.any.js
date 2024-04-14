// META: title=validation tests for WebNN API transpose operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('transpose');

const tests = [
  {
    name: '[transpose] Test building transpose with default options.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    output: {dataType: 'float32', dimensions: [4, 3, 2, 1]}
  },
  {
    name: '[transpose] Test building transpose with permutation=[0, 2, 3, 1].',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {permutation: [0, 2, 3, 1]},
    output: {dataType: 'float32', dimensions: [1, 3, 4, 2]}
  },
  {
    name:
        '[transpose] Throw if permutation\'s size is not the same as input\'s rank.',
    input: {dataType: 'int32', dimensions: [1, 2, 4]},
    options: {permutation: [0, 2, 3, 1]},
  },
  {
    name: '[transpose] Throw if two values in permutation are same.',
    input: {dataType: 'int32', dimensions: [1, 2, 3, 4]},
    options: {permutation: [0, 2, 3, 2]},
  },
  {
    name:
        '[transpose] Throw if any value in permutation is not in the range [0,input\'s rank).',
    input: {dataType: 'int32', dimensions: [1, 2, 3, 4]},
    options: {permutation: [0, 1, 2, 4]},
  },
  {
    name: '[transpose] Throw if any value in permutation is negative.',
    input: {dataType: 'int32', dimensions: [1, 2, 3, 4]},
    options: {permutation: [0, -1, 2, 3]},
  }
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      if (test.output) {
        const output = builder.transpose(input, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.transpose(input, test.options));
      }
    }, test.name));
