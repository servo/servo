// META: title=validation tests for WebNN API softmax operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests_without_axis = [
  {
    name: '[softmax] Test building Softmax with float32 input without axis.',
    input: {dataType: 'float32', shape: [4, 3]},
    output: {dataType: 'float32', shape: [4, 3]}
  },
  {
    name: '[softmax] Test building Softmax with float16 input without axis.',
    input: {dataType: 'float16', shape: [3, 5]},
    output: {dataType: 'float16', shape: [3, 5]}
  },
  {
    name: '[softmax] Throw if the input is not a non-floating point data.',
    input: {dataType: 'int32', shape: [3, 2]}
  },
  {
    name: '[softmax] Throw if the input dimensions is not 2.',
    input: {dataType: 'float32', shape: [1, 4, 3]}
  }
];

tests_without_axis.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      let input = builder.input(`input`, test.input);
      if (test.output) {
        const output = builder.softmax(input);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const options = {
          label: 'softmax_xxx',
        };
        try {
          builder.softmax(input, options);
        } catch (e) {
          assert_equals(e.name, 'TypeError');
          const error_message = e.message;
          const regrexp = /\[softmax_xxx\]/;
          assert_not_equals(error_message.match(regrexp), null);
        }
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const operandDescriptor = {dataType: 'float32', shape: [2, 3]};
  const inputFromOtherBuilder = otherBuilder.input('input', operandDescriptor);

  assert_throws_js(TypeError, () => builder.softmax(inputFromOtherBuilder));
}, '[softmax without axis] throw if any input is from another builder');

const tests = [
  {
    name: '[softmax] Test building Softmax with float32 input.',
    input: {dataType: 'float32', shape: [4, 4, 3]},
    axis: 1,
    output: {dataType: 'float32', shape: [4, 4, 3]}
  },
  {
    name: '[softmax] Test building Softmax with float16 input.',
    input: {dataType: 'float16', shape: [3, 1, 5, 2]},
    axis: 2,
    output: {dataType: 'float16', shape: [3, 1, 5, 2]}
  },
  {
    name: '[softmax] Throw if the input is not a non-floating-point data.',
    input: {dataType: 'int32', shape: [3, 1, 5, 2]},
    axis: 3
  },
  {
    name: '[softmax] Throw if the axis is greater than input rank - 1.',
    input: {dataType: 'float16', shape: [3, 1, 5, 2]},
    axis: 4
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      let input = builder.input(`input`, test.input);
      if (test.output) {
        const output = builder.softmax(input, test.axis);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'softmax_xxx';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.softmax(input, test.axis, options), regrexp);
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const operandDescriptor = {dataType: 'float32', shape: [1, 2, 3]};
  const inputFromOtherBuilder = otherBuilder.input('input', operandDescriptor);
  const axis = 1;

  assert_throws_js(
      TypeError, () => builder.softmax(inputFromOtherBuilder, axis));
}, '[softmax] throw if any input is from another builder');
