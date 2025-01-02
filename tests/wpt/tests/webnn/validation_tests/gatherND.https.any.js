// META: title=validation tests for WebNN API gatherND operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[gatherND] Test gatherND with 5D input 3D indices',
    input: {dataType: 'float32', shape: [2, 2, 3, 3, 4]},
    indices: {dataType: 'int32', shape: [5, 4, 3]},
    output: {dataType: 'float32', shape: [5, 4, 3, 4]}
  },
  {
    name: '[gatherND] Throw if input is a scalar',
    input: {dataType: 'float32', shape: []},
    indices: {dataType: 'int32', shape: [1, 1, 1]}
  },
  {
    name: '[gatherND] Throw if indices is a scalar',
    input: {dataType: 'float32', shape: [1, 1, 1]},
    indices: {dataType: 'int32', shape: []}
  },
  {
    name: '[gatherND] Throw if indices data type is float32',
    input: {dataType: 'float32', shape: [1, 2, 3]},
    indices: {dataType: 'float32', shape: [1, 1, 1]},
  },
  {
    name:
        '[gatherND] Throw if indices.shape[-1] is greater than the input rank',
    input: {dataType: 'float32', shape: [1, 2, 3]},
    indices: {dataType: 'int32', shape: [1, 1, 4]}
  }
];

tests.forEach(test => promise_test(async t => {
                const builder = new MLGraphBuilder(context);
                const input = builder.input('input', test.input);
                const indices = builder.input('indices', test.indices);

                if (test.output &&
                    context.opSupportLimits().gatherND.input.dataTypes.includes(
                        test.input.dataType)) {
                  const output = builder.gatherND(input, indices);
                  assert_equals(output.dataType, test.output.dataType);
                  assert_array_equals(output.shape, test.output.shape);
                } else {
                  const label = 'gatherND_';
                  const options = {label: label};
                  const regexp = new RegExp('\\[' + label + '\\]');
                  assert_throws_with_label(
                      () => builder.gatherND(input, indices, options), regexp);
                }
              }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [2, 2]});

  const indices = builder.input('indices', {dataType: 'int32', shape: [2, 1]});
  assert_throws_js(
      TypeError, () => builder.gatherND(inputFromOtherBuilder, indices));
}, '[gatherND] Throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const indicesFromOtherBuilder =
      otherBuilder.input('indices', {dataType: 'int32', shape: [2, 2]});

  const input = builder.input('input', {dataType: 'float32', shape: [2, 1]});
  assert_throws_js(
      TypeError, () => builder.gatherND(input, indicesFromOtherBuilder));
}, '[gatherND] Throw if indices is from another builder');
