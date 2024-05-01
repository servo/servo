// META: title=validation tests for WebNN API pad operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const beginningPadding = [1, 1];
  const endingPadding = [1, 1];
  assert_throws_js(
      TypeError,
      () =>
          builder.pad(inputFromOtherBuilder, beginningPadding, endingPadding));
}, '[pad] throw if input is from another builder');

const tests = [
  {
    name:
        '[pad] Test with default options, beginningPadding=[1, 2] and endingPadding=[1, 2].',
    input: {dataType: 'float32', dimensions: [2, 3]},
    beginningPadding: [1, 2],
    endingPadding: [1, 2],
    options: {
      mode: 'constant',
      value: 0,
    },
    output: {dataType: 'float32', dimensions: [4, 7]}
  },
  {
    name: '[pad] Throw if building pad for scalar input.',
    input: {dataType: 'float32', dimensions: []},
    beginningPadding: [],
    endingPadding: [],
  },
  {
    name:
        '[pad] Throw if the length of beginningPadding is not equal to the input rank.',
    input: {dataType: 'float32', dimensions: [2, 3]},
    beginningPadding: [1],
    endingPadding: [1, 2],
    options: {
      mode: 'edge',
      value: 0,
    },
  },
  {
    name:
        '[pad] Throw if the length of endingPadding is not equal to the input rank.',
    input: {dataType: 'float32', dimensions: [2, 3]},
    beginningPadding: [1, 0],
    endingPadding: [1, 2, 0],
    options: {
      mode: 'reflection',
    },
  },
  {
    name: '[pad] Throw if the padding of one dimension is too large.',
    input: {dataType: 'float32', dimensions: [2, 3]},
    beginningPadding: [2294967295, 0],
    endingPadding: [3294967295, 2],
    options: {
      mode: 'reflection',
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      if (test.output) {
        const output = builder.pad(
            input, test.beginningPadding, test.endingPadding, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError,
            () => builder.pad(
                input, test.beginningPadding, test.endingPadding,
                test.options));
      }
    }, test.name));
