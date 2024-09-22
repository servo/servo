// META: title=test WebNN API gatherElements operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-gatherElements
// Gather values of the input tensor along an axis according to the indices.
//
// dictionary MLGatherOptions {
//   [EnforceRange] unsigned long axis = 0;
// };
//
// MLOperand gatherElements(
//     MLOperand input, MLOperand indices,
//     optional MLGatherOptions options = {});


const getGatherElementsPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const gatherElementsTests = [
  {
    'name': 'gatherElements float32 2D input and uint32 indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {shape: [3, 2], dataType: 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 3D input and int32 negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, 0, 0, -1],
          'descriptor': {shape: [1, 2, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            89.0337142944336, -68.9197006225586, -77.02045440673828,
            48.81806945800781
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 1D input and uint32 out-of-bounds indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [6], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [7],
          'descriptor': {shape: [1], dataType: 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.79948425292969],
          'descriptor': {shape: [1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  gatherElementsTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getGatherElementsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
