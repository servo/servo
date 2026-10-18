// META: title=WebNN nearest resampling half-pixel coordinates
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// WebNN uses half-pixel coordinates, clamped to the input bounds, then
// ceil(coordinate - 0.5) for nearest sampling (ties select the smaller index).
// Integer upscaling alone does not distinguish this from asymmetric sampling.
const nearestHalfPixelCases = [
  {
    name: 'noninteger upscaling with scales',
    input: [10, 20, 30], shape: [1, 1, 1, 3],
    options: {scales: [1, 1.5]},
    expected: [10, 20, 20, 30], outputShape: [1, 1, 1, 4]
  },
  {
    name: 'noninteger downscaling with scales',
    input: [10, 20, 30, 40], shape: [1, 1, 1, 4],
    options: {scales: [1, 0.75]},
    expected: [10, 20, 40], outputShape: [1, 1, 1, 3]
  },
  {
    name: 'downscaling with sizes',
    input: [10, 20, 30, 40, 50], shape: [1, 1, 1, 5],
    options: {sizes: [1, 2]},
    expected: [20, 40], outputShape: [1, 1, 1, 2]
  },
  {
    name: 'ties select the smaller index',
    input: [10, 20], shape: [1, 1, 1, 2],
    options: {sizes: [1, 3]},
    expected: [10, 10, 20], outputShape: [1, 1, 1, 3]
  },
  {
    name: 'singleton destination selects the center',
    input: [10, 20, 30], shape: [1, 1, 1, 3],
    options: {sizes: [1, 1]},
    expected: [20], outputShape: [1, 1, 1, 1]
  },
  {
    name: 'singleton source clamps every coordinate',
    input: [10], shape: [1, 1, 1, 1],
    options: {sizes: [1, 4]},
    expected: [10, 10, 10, 10], outputShape: [1, 1, 1, 4]
  },
  {
    name: 'NHWC axes with sizes taking precedence over scales',
    input: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    shape: [1, 2, 3, 2],
    options: {axes: [1, 2], sizes: [3, 4], scales: [99, 99]},
    expected: [
      1, 2, 3, 4, 3, 4, 5, 6,
      1, 2, 3, 4, 3, 4, 5, 6,
      7, 8, 9, 10, 9, 10, 11, 12
    ],
    outputShape: [1, 3, 4, 2]
  },
  {
    name: 'reversed axes preserve their sizes order',
    input: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    shape: [1, 2, 3, 2],
    options: {axes: [3, 2], sizes: [3, 4]},
    expected: [
      1, 1, 2, 3, 3, 4, 3, 3, 4, 5, 5, 6,
      7, 7, 8, 9, 9, 10, 9, 9, 10, 11, 11, 12
    ],
    outputShape: [1, 2, 4, 3]
  }
];

const nearestHalfPixelTests = ['float32', 'float16'].flatMap(dataType =>
  nearestHalfPixelCases.map(test => ({
    name: `resample2d ${dataType} nearest ${test.name}`,
    graph: {
      inputs: {
        input: {
          data: test.input,
          descriptor: {shape: test.shape, dataType}
        }
      },
      operators: [{
        name: 'resample2d',
        arguments: [
          {input: 'input'},
          {options: {mode: 'nearest-neighbor', ...test.options}}
        ],
        outputs: 'result'
      }],
      expectedOutputs: {
        result: {
          data: test.expected,
          descriptor: {shape: test.outputShape, dataType}
        }
      }
    }
  })));

webnn_conformance_test(
    nearestHalfPixelTests, buildAndExecuteGraph, getPrecisionTolerance);
