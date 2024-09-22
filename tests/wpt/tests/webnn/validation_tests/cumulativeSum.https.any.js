// META: title=validation tests for WebNN API relu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[cumulativeSum] Test with default options',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    axis: 0,
    output: {dataType: 'float32', shape: [3, 2, 5]}
  },
  {
    name: '[cumulativeSum] Test with axis=1',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    axis: 1,
    output: {dataType: 'float32', shape: [3, 2, 5]}
  },
  {
    name: '[cumulativeSum] Test with exclusive=true',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    axis: 1,
    options: {exclusive: true},
    output: {dataType: 'float32', shape: [3, 2, 5]}
  },
  {
    name: '[cumulativeSum] Test with reversed=true',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    axis: 1,
    options: {reversed: true},
    output: {dataType: 'float32', shape: [3, 2, 5]}
  },
  {
    name: '[cumulativeSum] Throw if input is a scalar',
    input: {dataType: 'float32', shape: []},
    axis: 0
  },
  {
    name: '[cumulativeSum] Throw if axis is invalid',
    input: {dataType: 'float32', shape: [1, 2, 3]},
    axis: 3
  },
]

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);

      const options = {};
      if (test.options) {
        if (test.options.exclusive) {
          options.exclusive = test.options.exclusive;
        }
        if (test.options.reversed) {
          options.reversed = test.options.reversed;
        }
      }
      if (test.output) {
        const output = builder.cumulativeSum(input, test.axis, options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.shape);
      } else {
        const label = 'cumulative_sum';
        options.label = label;
        const regexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.cumulativeSum(input, test.axis, options), regexp);
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [3, 2, 5]});
  assert_throws_js(
      TypeError, () => builder.cumulativeSum(inputFromOtherBuilder, 0));
}, '[cumulativeSum] throw if input is from another builder');
