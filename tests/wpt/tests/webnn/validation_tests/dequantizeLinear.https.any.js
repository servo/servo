// META: title=validation tests for WebNN API prelu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name:
        '[dequantizeLinear] Test scale\'s shape = [3, 2, 5] and zeroPoint\'s shape = [3, 2, 5] which is the same as input\'s shape.',
    input: {dataType: 'int8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [3, 2, 5]},
    zeroPoint: {dataType: 'int8', shape: [3, 2, 5]},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[dequantizeLinear] Test scale\'s shape = [5] and zeroPoint\'s shape = [5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'int8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int8', shape: [5]},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[dequantizeLinear] Test scale\'s shape = [] and zeroPoint\'s shape = [] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'uint8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: []},
    zeroPoint: {dataType: 'uint8', shape: []},
    output: {dataType: 'float32', shape: [3, 2, 5]},
  },
  {
    name:
        '[dequantizeLinear] Test block-wise quantization with block_size = [2, 2, 5].',
    input: {dataType: 'uint8', shape: [6, 4, 5]},
    scale: {dataType: 'float32', shape: [3, 2, 1]},
    zeroPoint: {dataType: 'uint8', shape: [3, 2, 1]},
    output: {dataType: 'float32', shape: [6, 4, 5]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the scale size is not a factor of input size.',
    input: {dataType: 'uint8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [2]},
    zeroPoint: {dataType: 'uint8', shape: [2]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the shape of zero_point is not the same as the shape of input.',
    input: {dataType: 'uint8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'uint8', shape: [2]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the data type of zeroPoint is not the same as the data type of input.',
    input: {dataType: 'int8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'uint8', shape: [5]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the data type of input is not one of {int4, uint4, int8, uint8}.',
    input: {dataType: 'float16', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int8', shape: [5]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the data type of zero_point is not one of {int4, uint4, int8, uint8}.',
    input: {dataType: 'int8', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int32', shape: [5]},
  },
  {
    name: '[dequantizeLinear] Throw if the data type of scale is float32.',
    input: {dataType: 'uint8', shape: [3, 2, 5]},
    scale: {dataType: 'int32', shape: [5]},
    zeroPoint: {dataType: 'uint8', shape: [5]},
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const scale = builder.input('scale', test.scale);
      const zeroPoint = builder.input('zeroPoint', test.zeroPoint);
      if (test.output) {
        const output = builder.dequantizeLinear(input, scale, zeroPoint);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'dequantize_linear_123';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.dequantizeLinear(input, scale, zeroPoint, options),
            regrexp);
      }
    }, test.name));

const kExampleInputDescriptor = {
  dataType: 'int8',
  shape: [2, 4]
};
const kExampleScaleDescriptor = {
  dataType: 'float32',
  shape: [2, 4]
};
multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const scale = builder.input('scale', kExampleScaleDescriptor);
  const zeroPoint = builder.input('zeroPoint', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.dequantizeLinear(inputFromOtherBuilder, scale, zeroPoint));
}, '[dequantizeLinear] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExampleScaleDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const zeroPoint = builder.input('zeroPoint', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.dequantizeLinear(input, scaleFromOtherBuilder, zeroPoint));
}, '[dequantizeLinear] throw if scale is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const zeroPointFromOtherBuilder =
      otherBuilder.input('zeroPoint', kExampleInputDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const scale = builder.input('scale', kExampleScaleDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.dequantizeLinear(input, scale, zeroPointFromOtherBuilder));
}, '[dequantizeLinear] throw if zeroPoint is from another builder');
