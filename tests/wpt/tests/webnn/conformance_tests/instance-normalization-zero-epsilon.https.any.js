// META: title=WebNN instance normalization honors zero epsilon
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Both channels have nonzero spatial variance, even though their inputs are
// float16 subnormals. With epsilon zero, normalization produces exactly +/-1.
// Substituting the default epsilon instead produces values near zero.
const zeroEpsilonTests = [];
const zeroEpsilonExpected = {
  nchw: {
    plain: [1, -1, 1, -1, 1, -1, 1, -1],
    affine: [0.75, -0.25, 0.75, -0.25, 1.5, -2.5, 1.5, -2.5]
  },
  nhwc: {
    plain: [1, 1, -1, -1, 1, 1, -1, -1],
    affine: [0.75, 1.5, -0.25, -2.5, 0.75, 1.5, -0.25, -2.5]
  }
};
const unit = 2 ** -24;
for (const dataType of ['float32', 'float16']) {
  for (const layout of ['nchw', 'nhwc']) {
    const data = layout === 'nchw' ?
        [unit, -unit, unit, -unit, 2 * unit, -2 * unit, 2 * unit, -2 * unit] :
        [unit, 2 * unit, -unit, -2 * unit, unit, 2 * unit, -unit, -2 * unit];
    for (const affine of [false, true]) {
      const inputs = {
        input: {data, descriptor: {shape: [1, 2, 2, 2], dataType}}
      };
      const options = {layout, epsilon: 0};
      if (affine) {
        inputs.scale = {
          data: [0.5, 2], descriptor: {shape: [2], dataType}, constant: true
        };
        inputs.bias = {
          data: [0.25, -0.5], descriptor: {shape: [2], dataType}, constant: true
        };
        options.scale = 'scale';
        options.bias = 'bias';
      }
      zeroEpsilonTests.push({
        name: `instanceNormalization ${dataType} ${layout} zero epsilon ${
            affine ? 'constant affine' : 'no affine'}`,
        graph: {
          inputs,
          operators: [{
            name: 'instanceNormalization',
            arguments: [{input: 'input'}, {options}], outputs: 'result'
          }],
          expectedOutputs: {
            result: {
              data: zeroEpsilonExpected[layout][affine ? 'affine' : 'plain'],
              descriptor: {shape: [1, 2, 2, 2], dataType}
            }
          }
        }
      });
    }
  }
}

webnn_conformance_test(
    zeroEpsilonTests, buildAndExecuteGraph, getInstanceNormPrecisionTolerance);
