// META: title=WebNN reduceLogSum small, zero, and large sums
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// reduceLogSum is log(sum(x)), without an added epsilon. All input values
// below are exactly representable in both input types. The large-sum case
// has a finite float16 result even though the sum exceeds float16's range.
// That case assumes at least float32 intermediate accumulation, or another
// algorithm that avoids float16 overflow before taking the logarithm.
const logSumSmallValueTests = [];
for (const dataType of ['float32', 'float16']) {
  const expected = dataType === 'float32' ? {
    tiny: -16.63553237915039,
    small: [-13.862943649291992, -12.476649284362793],
    large: 12.476161003112793,
    two: 0.6931471824645996
  } : {
    tiny: -16.640625,
    small: [-13.859375, -12.4765625],
    large: 12.4765625,
    two: 0.693359375
  };
  for (const constant of [false, true]) {
    for (const test of [
      {
        label: 'small positive sums', shape: [2, 4],
        data: [
          2 ** -24, 3 * 2 ** -24, 5 * 2 ** -24, 7 * 2 ** -24,
          4 * 2 ** -24, 12 * 2 ** -24, 20 * 2 ** -24, 28 * 2 ** -24
        ],
        options: {axes: [1]}, outputShape: [2], output: expected.small
      },
      {
        label: 'zero and cancelled sums with keepDimensions', shape: [2, 4],
        data: [0, 0, 0, 0, 1, -1, 0, 0],
        options: {axes: [1], keepDimensions: true}, outputShape: [2, 1],
        output: [-Infinity, -Infinity]
      },
      {
        label: 'finite result after large sum', shape: [4],
        data: [65504, 65504, 65504, 65504], options: {}, outputShape: [],
        output: [expected.large]
      },
      {
        label: 'small scalar', shape: [], data: [2 ** -24], options: {},
        outputShape: [], output: [expected.tiny]
      },
      {
        label: 'empty axes', shape: [3], data: [2 ** -24, 0, 2],
        options: {axes: []}, outputShape: [3],
        output: [expected.tiny, -Infinity, expected.two]
      }
    ]) {
      logSumSmallValueTests.push({
        name: `reduceLogSum ${dataType} ${constant ? 'constant' : 'runtime'} ` +
            test.label,
        graph: {
          inputs: {
            input: {
              data: test.data, descriptor: {shape: test.shape, dataType},
              constant
            }
          },
          operators: [{
            name: 'reduceLogSum',
            arguments: [{input: 'input'}, {options: test.options}],
            outputs: 'result'
          }],
          expectedOutputs: {
            result: {
              data: test.output,
              descriptor: {shape: test.outputShape, dataType}
            }
          }
        }
      });
    }
  }
}

webnn_conformance_test(logSumSmallValueTests, buildAndExecuteGraph,
                       getPrecisionTolerance);
