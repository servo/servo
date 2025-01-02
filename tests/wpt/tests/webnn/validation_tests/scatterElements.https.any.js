// META: title=validation tests for WebNN API scatterElements operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[scatterElements] Test scatterElements with default options',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float32', shape: [2, 3]},
    output: {dataType: 'float32', shape: [3, 3]}
  },
  {
    name: '[scatterElements] Test scatterElements with axis = 0',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float32', shape: [2, 3]},
    axis: 0,
    output: {dataType: 'float32', shape: [3, 3]}
  },
  {
    name: '[scatterElements] Test scatterElements with axis = 1',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [3, 2]},
    updates: {dataType: 'float32', shape: [3, 2]},
    axis: 1,
    output: {dataType: 'float32', shape: [3, 3]}
  },
  {
    name: '[scatterElements] Throw if axis is greater than input rank',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float32', shape: [2, 3]},
    axis: 2
  },
  {
    name:
        '[scatterElements] Throw if updates tensor data type is not the same as input data type',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float16', shape: [2, 3]},
  },
  {
    name: '[scatterElements] Throw if input, indices and updates are scalar',
    input: {dataType: 'float32', shape: []},
    indices: {dataType: 'int32', shape: []},
    updates: {dataType: 'float32', shape: []},
  },
  {
    name:
        '[scatterElements] Throw if indices rank is not the same as input rank',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3, 3]},
    updates: {dataType: 'float32', shape: [2, 3, 3]},
  },
  {
    name:
        '[scatterElements] Throw if indices size is not the same as input size along axis 1',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 4]},
    updates: {dataType: 'float32', shape: [2, 4]},
    axis: 0
  },
  {
    name:
        '[scatterElements] Throw if indices size is not the same as input size along axis 0',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 2]},
    updates: {dataType: 'float32', shape: [2, 2]},
    axis: 1
  },
  {
    name:
        '[scatterElements] Throw if indices rank is not the same as updates rank',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float32', shape: [2, 3, 3]},
  },
  {
    name:
        '[scatterElements] Throw if indices shape is not the same as updates shape',
    input: {dataType: 'float32', shape: [3, 3]},
    indices: {dataType: 'int32', shape: [2, 3]},
    updates: {dataType: 'float32', shape: [2, 4]},
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const indices = builder.input('indices', test.indices);
      const updates = builder.input('updates', test.updates);

      const options = {};
      if (test.axis) {
        options.axis = test.axis;
      }

      if (test.output) {
        const output =
            builder.scatterElements(input, indices, updates, options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'a_scatter_elements'
        options.label = label;
        const regexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.scatterElements(input, indices, updates, options),
            regexp);
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const input =
      otherBuilder.input('input', {dataType: 'float32', shape: [3, 3]});
  const indices = builder.input('indices', {dataType: 'int32', shape: [2, 3]});
  const updates =
      builder.input('updates', {dataType: 'float32', shape: [2, 3]});

  assert_throws_js(
      TypeError, () => builder.scatterElements(input, indices, updates));
}, '[scatterElements] Throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const input = builder.input('input', {dataType: 'float32', shape: [3, 3]});
  const indices =
      otherBuilder.input('indices', {dataType: 'int32', shape: [2, 3]});
  const updates =
      builder.input('updates', {dataType: 'float32', shape: [2, 3]});

  assert_throws_js(
      TypeError, () => builder.scatterElements(input, indices, updates));
}, '[scatterElements] Throw if indices is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const input = builder.input('input', {dataType: 'float32', shape: [3, 3]});
  const indices = builder.input('indices', {dataType: 'int32', shape: [2, 3]});
  const updates =
      otherBuilder.input('updates', {dataType: 'float32', shape: [2, 3]});

  assert_throws_js(
      TypeError, () => builder.scatterElements(input, indices, updates));
}, '[scatterElements] Throw if updates is from another builder');
