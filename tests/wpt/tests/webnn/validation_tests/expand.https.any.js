// META: title=validation tests for WebNN API expand operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [2, 1, 2]});

  const newShape = [2, 2, 2];
  assert_throws_js(
      TypeError, () => builder.expand(inputFromOtherBuilder, newShape));
}, '[expand] throw if input is from another builder');

const label = 'xxx_expand';

const tests = [
  {
    name: '[expand] Test with 0-D scalar to 3-D tensor.',
    input: {dataType: 'float32', shape: []},
    newShape: [3, 4, 5],
    output: {dataType: 'float32', shape: [3, 4, 5]}
  },
  {
    name: '[expand] Test with the new shapes that are the same as input.',
    input: {dataType: 'float32', shape: [4]},
    newShape: [4],
    output: {dataType: 'float32', shape: [4]}
  },
  {
    name: '[expand] Test with the new shapes that are broadcastable.',
    input: {dataType: 'float32', shape: [3, 1, 5]},
    newShape: [3, 4, 5],
    output: {dataType: 'float32', shape: [3, 4, 5]}
  },
  {
    name:
        '[expand] Test with the new shapes that are broadcastable and the rank of new shapes is larger than input.',
    input: {dataType: 'float32', shape: [2, 5]},
    newShape: [3, 2, 5],
    output: {dataType: 'float32', shape: [3, 2, 5]}
  },
  {
    name:
        '[expand] Throw if the input shapes are the same rank but not broadcastable.',
    input: {dataType: 'float32', shape: [3, 6, 2]},
    newShape: [4, 3, 5],
    options: {label}
  },
  {
    name: '[expand] Throw if the input shapes are not broadcastable.',
    input: {dataType: 'float32', shape: [5, 4]},
    newShape: [5],
    options: {label}
  },
  {
    name: '[expand] Throw if the number of new shapes is too large.',
    input: {dataType: 'float32', shape: [1, 2, 1, 1]},
    newShape: [1, 2, kMaxUnsignedLong, kMaxUnsignedLong],
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);

      if (test.output) {
        const output = builder.expand(input, test.newShape);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.shape);
      } else {
        const options = {...test.options};
        if (options.label) {
          const regrexp = new RegExp('\\[' + label + '\\]');
          assert_throws_with_label(
              () => builder.expand(input, test.newShape, options), regrexp);
        } else {
          assert_throws_js(
              TypeError, () => builder.expand(input, test.newShape, options));
        }
      }
    }, test.name));

promise_test(async t => {
  for (let dataType of allWebNNOperandDataTypes) {
    if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
      continue;
    }
    const builder = new MLGraphBuilder(context);
    const shape = [1];
    const newShape = [1, 2, 3];
    const input = builder.input(`input`, {dataType, shape});
    if (context.opSupportLimits().expand.input.dataTypes.includes(dataType)) {
      const output = builder.expand(input, newShape);
      assert_equals(output.dataType(), dataType);
      assert_array_equals(output.shape(), newShape);
    } else {
      assert_throws_js(TypeError, () => builder.expand(input, newShape));
    }
  }
}, `[expand] Test expand with all of the data types.`);
