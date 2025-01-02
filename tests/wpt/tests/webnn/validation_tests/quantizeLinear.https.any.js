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
        '[quantizeLinear] Test scale\'s shape = [3, 2, 5] and zeroPoint\'s shape = [3, 2, 5] which is the same as input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [3, 2, 5]},
    zeroPoint: {dataType: 'int8', shape: [3, 2, 5]},
    output: {dataType: 'int8', shape: [3, 2, 5]},
  },
  {
    name:
        '[quantizeLinear] Test scale\'s shape = [5] and zeroPoint\'s shape = [5] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int8', shape: [5]},
    output: {dataType: 'int8', shape: [3, 2, 5]},
  },
  {
    name:
        '[quantizeLinear] Test scale\'s shape = [] and zeroPoint\'s shape = [] which is unidirectionally broadcastable to input\'s shape.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: []},
    zeroPoint: {dataType: 'int8', shape: []},
    output: {dataType: 'int8', shape: [3, 2, 5]},
  },
  {
    name:
        '[quantizeLinear] Test block-wise quantization with block_size = [3, 2, 5].',
    input: {dataType: 'float32', shape: [6, 4, 5]},
    scale: {dataType: 'float32', shape: [2, 2, 1]},
    zeroPoint: {dataType: 'int8', shape: [2, 2, 1]},
    output: {dataType: 'int8', shape: [6, 4, 5]},
  },
  {
    name:
        '[quantizeLinear] Throw if the scale size is not a factor of input size.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [2, 1, 5]},
    zeroPoint: {dataType: 'int8', shape: [2, 1, 5]},
  },
  {
    name:
        '[quantizeLinear] Throw if the shape of zero_point is not the same as the shape of input.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int8', shape: [2, 5]},
  },
  {
    name:
        '[quantizeLinear] Throw if the data type of input is not the same as scale.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float16', shape: [3, 1, 5]},
    zeroPoint: {dataType: 'int8', shape: [3, 1, 5]},
  },
  {
    name:
        '[quantizeLinear] Throw if the data type of input is not float32 or float16.',
    input: {dataType: 'int32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'int8', shape: [5]},
  },
  {
    name:
        '[quantizeLinear] Throw if the data type of scale is not float32 or float16.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'int32', shape: [5]},
    zeroPoint: {dataType: 'uint8', shape: [5]},
  },
  {
    name:
        '[dequantizeLinear] Throw if the data type of zeroPoint is not int8 or uint8.',
    input: {dataType: 'float32', shape: [3, 2, 5]},
    scale: {dataType: 'float32', shape: [5]},
    zeroPoint: {dataType: 'float16', shape: [5]},
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const scale = builder.input('scale', test.scale);
      const zeroPoint = builder.input('zeroPoint', test.zeroPoint);
      if (test.output) {
        const output = builder.quantizeLinear(input, scale, zeroPoint);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'quantize_linear_123';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.quantizeLinear(input, scale, zeroPoint, options),
            regrexp);
      }
    }, test.name));

const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: [2, 4]
};
const kExampleZeroPointDescriptor = {
  dataType: 'int8',
  shape: [2, 4]
};
multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const scale = builder.input('scale', kExampleInputDescriptor);
  const zeroPoint = builder.input('zeroPoint', kExampleZeroPointDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.quantizeLinear(inputFromOtherBuilder, scale, zeroPoint));
}, '[quantizeLinear] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExampleInputDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const zeroPoint = builder.input('zeroPoint', kExampleZeroPointDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.quantizeLinear(input, scaleFromOtherBuilder, zeroPoint));
}, '[quantizeLinear] throw if scale is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const zeroPointFromOtherBuilder =
      otherBuilder.input('zeroPoint', kExampleZeroPointDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const scale = builder.input('scale', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.quantizeLinear(input, scale, zeroPointFromOtherBuilder));
}, '[quantizeLinear] throw if zeroPoint is from another builder');
