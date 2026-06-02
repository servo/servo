// META: title=validation tests for WebNN API softmax operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

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
  },
  {
    name: '[softmax] Throw if the input is a scalar.',
    input: {dataType: 'float32', shape: []},
    axis: 0
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
