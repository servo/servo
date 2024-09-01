// META: title=validation tests for WebNN API gatherElements operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[gatherElements] Test gatherElements with default options',
    input: {dataType: 'float32', dimensions: [1, 2, 3]},
    indices: {dataType: 'int32', dimensions: [2, 2, 3]},
    output: {dataType: 'float32', dimensions: [2, 2, 3]}
  },
  {
    name: '[gatherElements] Test gatherElements with axis = 2',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'int32', dimensions: [1, 2, 1, 4]},
    axis: 2,
    output: {dataType: 'float32', dimensions: [1, 2, 1, 4]}
  },
  {
    name: '[gatherElements] Throw if input is a scalar',
    input: {dataType: 'float32', dimensions: []},
    indices: {dataType: 'int32', dimensions: []}
  },
  {
    name:
        '[gatherElements] Throw if the axis is greater than the rank of input',
    input: {dataType: 'float32', dimensions: [1, 2, 3]},
    indices: {dataType: 'int32', dimensions: [1, 2, 3]},
    axis: 4
  },
  {
    name: '[gatherElements] Throw if indices data type is float32',
    input: {dataType: 'float32', dimensions: [1, 2, 3]},
    indices: {dataType: 'float32', dimensions: [1, 2, 3]}
  },
  {
    name: '[gatherElements] Throw if input rank is not equal to indices rank',
    input: {dataType: 'float32', dimensions: [1, 2, 3]},
    indices: {dataType: 'int32', dimensions: [1, 2]}
  },
  {
    name: '[gatherElements] Throw if indices shape is incorrect',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    indices: {dataType: 'int32', dimensions: [3, 2, 3, 4]},
    axis: 3
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const indices = builder.input('indices', {
        dataType: test.indices.dataType,
        dimensions: test.indices.dimensions
      });

      const options = {};
      if (test.axis) {
        options.axis = test.axis;
      }

      if (test.output) {
        const output = builder.gatherElements(input, indices, options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        const label = 'gatherElements_'
        options.label = label;
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.gatherElements(input, indices, options), regrexp);
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const indices =
      builder.input('indices', {dataType: 'int32', dimensions: [2, 2]});
  assert_throws_js(
      TypeError, () => builder.gatherElements(inputFromOtherBuilder, indices));
}, '[gatherElements] Throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const indicesFromOtherBuilder =
      otherBuilder.input('indices', {dataType: 'int32', dimensions: [2, 2]});

  const input =
      builder.input('input', {dataType: 'float32', dimensions: [2, 2]});
  assert_throws_js(
      TypeError, () => builder.gatherElements(input, indicesFromOtherBuilder));
}, '[gatherElements] Throw if indices is from another builder');
