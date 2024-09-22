// META: title=validation tests for pooling and reduction operators keep dimensions
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: script=../resources/utils_validation.js
// META: timeout=long

'use strict';

// This is used to reproduce an issue(crbug.com/331841268) of averagePool2d in
// ResNetV2 50 model.
//      [input]
//         |
//  [globalAveragePool]
//         |
//     [conv2d]
//         |
//     [reshape]
//         |
//     [output]
promise_test(async t => {
  const builder = new MLGraphBuilder(context);

  const avgPool2dInputShape = [1, 7, 7, 2048];
  const avgPool2dInput = builder.input(
      `avgPool2dInput`, {dataType: 'float32', shape: avgPool2dInputShape});
  const avgPool2dOutput =
      builder.averagePool2d(avgPool2dInput, {layout: 'nhwc'});
  const conv2dFilterShape = [1001, 1, 1, 2048];
  const conv2dFilter = builder.constant(
      {dataType: 'float32', shape: conv2dFilterShape},
      new Float32Array(sizeOfShape(conv2dFilterShape)).fill(1));
  const conv2dBias = builder.constant(
      {dataType: 'float32', shape: [1001]}, new Float32Array(1001).fill(0.01));
  const conv2dOutput = builder.conv2d(avgPool2dOutput, conv2dFilter, {
    inputLayout: 'nhwc',
    filterLayout: 'ohwi',
    padding: [0, 0, 0, 0],
    bias: conv2dBias
  });
  const newShape = [1, 1001];
  const reshapeOutput = builder.reshape(conv2dOutput, newShape);
  assert_equals(reshapeOutput.dataType(), avgPool2dInput.dataType());
  assert_array_equals(reshapeOutput.shape(), newShape);
  const graph = await builder.build({reshapeOutput});
  const result = await context.compute(
      graph, {
        'avgPool2dInput':
            new Float32Array(sizeOfShape(avgPool2dInputShape)).fill(0.1)
      },
      {'reshapeOutput': new Float32Array(1001)});
}, 'Test global average pool operator\'s output shape for ResNetV2 50 model.');

// This is used to reproduce an issue(crbug.com/331841268) of reduceMean in
// ResNetV2 50 model.
//      [input]
//         |
//    [reduceMean]
//         |
//     [conv2d]
//         |
//     [reshape]
//         |
//     [output]
promise_test(async t => {
  const builder = new MLGraphBuilder(context);

  const reduceMeanInputShape = [1, 7, 7, 2048];
  const reduceMeanInput = builder.input(
      `reduceMeanInput`, {dataType: 'float32', shape: reduceMeanInputShape});
  const reduceMeanOutput =
      builder.reduceMean(reduceMeanInput, {axes: [1, 2], keepDimensions: true});
  const conv2dFilterShape = [1001, 1, 1, 2048];
  const conv2dFilter = builder.constant(
      {dataType: 'float32', shape: conv2dFilterShape},
      new Float32Array(sizeOfShape(conv2dFilterShape)).fill(1));
  const conv2dBias = builder.constant(
      {dataType: 'float32', shape: [1001]}, new Float32Array(1001).fill(0.01));
  const conv2dOutput = builder.conv2d(reduceMeanOutput, conv2dFilter, {
    inputLayout: 'nhwc',
    filterLayout: 'ohwi',
    padding: [0, 0, 0, 0],
    bias: conv2dBias
  });
  const newShape = [1, 1001];
  const reshapeOutput = builder.reshape(conv2dOutput, newShape);
  assert_equals(reshapeOutput.dataType(), reduceMeanInput.dataType());
  assert_array_equals(reshapeOutput.shape(), newShape);
  const graph = await builder.build({reshapeOutput});
  const result = await context.compute(
      graph, {
        'reduceMeanInput':
            new Float32Array(sizeOfShape(reduceMeanInputShape)).fill(0.1)
      },
      {'reshapeOutput': new Float32Array(1001)});
}, 'Test reduceMean operator\'s output shape for ResNetV2 50 model.');
