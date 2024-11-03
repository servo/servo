// META: title=validation tests for WebNN API reshape operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [1, 2, 3]});

  const newShape = [3, 2, 1];
  assert_throws_js(
      TypeError, () => builder.reshape(inputFromOtherBuilder, newShape));
}, '[reshape] throw if input is from another builder');

const tests = [
  {
    name: '[reshape] Test with new shape=[3, 8].',
    input: {dataType: 'float32', shape: [2, 3, 4]},
    newShape: [3, 8],
    output: {dataType: 'float32', shape: [3, 8]}
  },
  {
    name: '[reshape] Test with new shape=[24], src shape=[2, 3, 4].',
    input: {dataType: 'float32', shape: [2, 3, 4]},
    newShape: [24],
    output: {dataType: 'float32', shape: [24]}
  },
  {
    name: '[reshape] Test with new shape=[1], src shape=[1].',
    input: {dataType: 'float32', shape: [1]},
    newShape: [1],
    output: {dataType: 'float32', shape: [1]}
  },
  {
    name: '[reshape] Test reshaping a 1-D 1-element tensor to scalar.',
    input: {dataType: 'float32', shape: [1]},
    newShape: [],
    output: {dataType: 'float32', shape: []}
  },
  {
    name: '[reshape] Test reshaping a scalar to 1-D 1-element tensor.',
    input: {dataType: 'float32', shape: []},
    newShape: [1],
    output: {dataType: 'float32', shape: [1]}
  },
  {
    name: '[reshape] Throw if one value of new shape is 0.',
    input: {dataType: 'float32', shape: [2, 4]},
    newShape: [2, 4, 0],
  },
  {
    name:
        '[reshape] Throw if the number of elements implied by new shape is not equal to the number of elements in the input tensor when new shape=[].',
    input: {dataType: 'float32', shape: [2, 3, 4]},
    newShape: [],
  },
  {
    name:
        '[reshape] Throw if the number of elements implied by new shape is not equal to the number of elements in the input tensor.',
    input: {dataType: 'float32', shape: [2, 3, 4]},
    newShape: [3, 9],
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      if (test.output) {
        const output = builder.reshape(input, test.newShape);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'reshape_xxx';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.reshape(input, test.newShape, options), regrexp);
      }
    }, test.name));
