// META: title=validation tests for WebNN API scatterND operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[scatterND] Test scatterND with valid tensors',
    input: {dataType: 'float32', shape: [4, 4, 4]},
    indices: {dataType: 'int32', shape: [2, 1]},
    updates: {dataType: 'float32', shape: [2, 4, 4]},
    output: {dataType: 'float32', shape: [4, 4, 4]}
  },
  {
    name:
        '[scatterND] Throw if updates tensor data type is not the same as input data type',
    input: {dataType: 'float32', shape: [4, 4, 4]},
    indices: {dataType: 'int32', shape: [2, 1]},
    updates: {dataType: 'float16', shape: [2, 4, 4]},
  },
  {
    name: '[scatterND] Throw if input is a scalar',
    input: {dataType: 'float32', shape: []},
    indices: {dataType: 'int32', shape: [2, 1]},
    updates: {dataType: 'float32', shape: [2, 4, 4]},
  },
  {
    name: '[scatterND] Throw if indices is a scalar',
    input: {dataType: 'float32', shape: [4, 4, 4]},
    indices: {dataType: 'int32', shape: []},
    updates: {dataType: 'float32', shape: [2, 4, 4]},
  },
  {
    name:
        '[scatterND] Throw if the size of last dimension of indices tensor is greater than input rank',
    input: {dataType: 'float32', shape: [4, 4, 4]},
    indices: {dataType: 'int32', shape: [2, 4]},
    updates: {dataType: 'float32', shape: [2, 4, 4]},
  },
  {
    name: '[scatterND] Throw if updates tensor shape is invalid.',
    input: {dataType: 'float32', shape: [4, 4, 4]},
    indices: {dataType: 'int32', shape: [2, 1]},
    // Updates tensor shape should be [2, 4, 4].
    updates: {dataType: 'float32', shape: [2, 3, 4]},
  }
];

tests.forEach(test => promise_test(async t => {
                const builder = new MLGraphBuilder(context);
                const input = builder.input('input', test.input);
                const indices = builder.input('indices', test.indices);
                const updates = builder.input('updates', test.updates);

                if (test.output) {
                  const output = builder.scatterND(input, indices, updates);
                  assert_equals(output.dataType(), test.output.dataType);
                  assert_array_equals(output.shape(), test.output.shape);
                } else {
                  const label = 'a_scatter_nd'
                  const options = {label};
                  const regexp = new RegExp('\\[' + label + '\\]');
                  assert_throws_with_label(
                      () => builder.scatterND(input, indices, updates, options),
                      regexp);
                }
              }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const input = otherBuilder.input('input', {dataType: 'float32', shape: [8]});
  const indices = builder.input('indices', {dataType: 'int32', shape: [4, 1]});
  const updates = builder.input('indices', {dataType: 'int32', shape: [4]});

  assert_throws_js(TypeError, () => builder.scatterND(input, indices, updates));
}, '[scatterND] Throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const input = builder.input('input', {dataType: 'float32', shape: [8]});
  const indices =
      otherBuilder.input('indices', {dataType: 'int32', shape: [4, 1]});
  const updates = builder.input('indices', {dataType: 'int32', shape: [4]});

  assert_throws_js(TypeError, () => builder.scatterND(input, indices, updates));
}, '[scatterND] Throw if indcies is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const input = builder.input('input', {dataType: 'float32', shape: [8]});
  const indices = builder.input('indices', {dataType: 'int32', shape: [4, 1]});
  const updates =
      otherBuilder.input('indices', {dataType: 'int32', shape: [4]});

  assert_throws_js(TypeError, () => builder.scatterND(input, indices, updates));
}, '[scatterND] Throw if updates is from another builder');
