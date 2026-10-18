// META: title=WebNN instance normalization affine operands and NHWC layout
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Each channel has spatial mean 3 and variance 5. With epsilon 4, the
// denominator is exactly 3. Affine parameters are runtime operands unless
// explicitly marked constant in the computed-parameter case.
const affineLayoutTests = [];
const affineLayoutExpected = {
  both: [-2.75, -5.5, -0.75, -1.5, 1.25, 2.5, 3.25, 6.5],
  scale: [-3, -6, -1, -2, 1, 2, 3, 6],
  bias: [
    -0.75, -0.5, -0.08333333333333331, 0.16666666666666669,
    0.5833333333333333, 0.8333333333333333, 1.25, 1.5
  ]
};
for (const dataType of ['float32', 'float16']) {
  for (const parameter of ['both', 'scale', 'bias']) {
    const scale = parameter !== 'bias';
    const bias = parameter !== 'scale';
    const inputs = {
      input: {
        data: [0, 0, 2, 2, 4, 4, 6, 6],
        descriptor: {shape: [1, 2, 2, 2], dataType}
      }
    };
    const options = {layout: 'nhwc', epsilon: 4};
    if (scale) {
      inputs.scale = {data: [3, 6], descriptor: {shape: [2], dataType}};
      options.scale = 'scale';
    }
    if (bias) {
      inputs.bias = {data: [0.25, 0.5], descriptor: {shape: [2], dataType}};
      options.bias = 'bias';
    }
    const graph = {
      inputs,
      operators: [{
        name: 'instanceNormalization',
        arguments: [{input: 'input'}, {options}],
        outputs: 'result'
      }],
      expectedOutputs: {
        result: {
          data: affineLayoutExpected[parameter],
          descriptor: {shape: [1, 2, 2, 2], dataType}
        }
      }
    };
    affineLayoutTests.push({
      name: `instanceNormalization ${dataType} NHWC runtime ${parameter}`,
      graph
    });
    if (parameter === 'both') {
      const constants = structuredClone(graph);
      constants.inputs.scale.constant = true;
      constants.inputs.bias.constant = true;
      affineLayoutTests.push({
        name: `instanceNormalization ${dataType} NHWC constant affine control`,
        graph: constants
      });
      const composed = structuredClone(graph);
      composed.inputs.filter = {
        data: [1, 0, 0, 1],
        descriptor: {shape: [2, 1, 1, 2], dataType},
        constant: true
      };
      composed.operators[0].outputs = 'normalized';
      composed.operators.push({
        name: 'conv2d',
        arguments: [
          {input: 'normalized'}, {filter: 'filter'},
          {options: {inputLayout: 'nhwc', filterLayout: 'ohwi'}}
        ],
        outputs: 'result'
      });
      affineLayoutTests.push({
        name: `instanceNormalization ${dataType} NHWC runtime affine then conv2d`,
        graph: composed
      });
    }
  }
}

affineLayoutTests.push({
  name: 'instanceNormalization float32 NHWC computed affine operands',
  graph: {
    inputs: {
      input: {
        data: [0, 0, 2, 2, 4, 4, 6, 6],
        descriptor: {shape: [1, 2, 2, 2], dataType: 'float32'}
      },
      halfScale: {
        data: [3, 6], descriptor: {shape: [2], dataType: 'float16'},
        constant: true
      },
      halfBias: {
        data: [0.25, 0.5], descriptor: {shape: [2], dataType: 'float16'},
        constant: true
      }
    },
    operators: [
      {
        name: 'cast', arguments: [{input: 'halfScale'}, {type: 'float32'}],
        outputs: 'scale'
      },
      {
        name: 'cast', arguments: [{input: 'halfBias'}, {type: 'float32'}],
        outputs: 'bias'
      },
      {
        name: 'instanceNormalization',
        arguments: [
          {input: 'input'},
          {options: {layout: 'nhwc', epsilon: 4, scale: 'scale', bias: 'bias'}}
        ],
        outputs: 'result'
      }
    ],
    expectedOutputs: {
      result: {
        data: [-2.75, -5.5, -0.75, -1.5, 1.25, 2.5, 3.25, 6.5],
        descriptor: {shape: [1, 2, 2, 2], dataType: 'float32'}
      }
    }
  }
});

const getAffineLayoutTolerance = (graphResources, intermediateOperands) => ({
  metricType: 'ULP',
  value: getInstanceNormPrecisionTolerance(graphResources).value +
      getPrecisionTolerance(graphResources, intermediateOperands).value
});

webnn_conformance_test(
    affineLayoutTests, buildAndExecuteGraph, getAffineLayoutTolerance);
