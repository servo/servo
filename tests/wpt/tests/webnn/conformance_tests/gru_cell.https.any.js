// META: title=test WebNN API gruCell operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-grucell
// A single time step of the Gated Recurrent Unit recurrent network using an
// update gate and a reset gate to compute the hidden state that rolls into the
// output across the temporal sequence of a recurrent network.
//
// enum MLGruWeightLayout {
//   "zrn",  // update-reset-new gate ordering
//   "rzn"   // reset-update-new gate ordering
// };
//
// enum MLRecurrentNetworkActivation {
//   "relu",
//   "sigmoid",
//   "tanh"
// };
//
// dictionary MLGruCellOptions {
//   MLOperand bias;
//   MLOperand recurrentBias;
//   boolean resetAfter = true;
//   MLGruWeightLayout layout = "zrn";
//   sequence<MLRecurrentNetworkActivation> activations;
// };
//
// MLOperand gruCell(MLOperand input,
//                   MLOperand weight,
//                   MLOperand recurrentWeight,
//                   MLOperand hiddenState,
//                   [EnforceRange] unsigned long hiddenSize,
//                   optional MLGruCellOptions options = {});


const getGruCellPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 3};
  const expectedDataType =
      graphResources
          .expectedOutputs[Object.keys(graphResources.expectedOutputs)[0]]
          .descriptor.dataType;
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const gruCellTests = [
  {
    'name':
        'gruCell float32 tensors with options.bias, options.recurrentBias and options.activations=[\'relu\', \'relu\']',
    'graph': {
      'inputs': {
        'gruCellInput': {
          'data': [1, 2, 2, 1, 1, 1],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        },
        'gruCellWeight': {
          'data': [
            1,   -1,   2, -2,  0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,
            0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,  0.5, -0.5, 0, 0.1
          ],
          'descriptor': {shape: [12, 2], dataType: 'float32'}
        },
        'gruCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [12, 4], dataType: 'float32'}
        },
        'gruCellHiddenState': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        },
        'gruCellBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        },
        'gruCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'gruCell',
        'arguments': [
          {'input': 'gruCellInput'}, {'weight': 'gruCellWeight'},
          {'recurrentWeight': 'gruCellRecurrentWeight'},
          {'hiddenState': 'gruCellHiddenState'}, {'hiddenSize': 4}, {
            'options': {
              'bias': 'gruCellBias',
              'recurrentBias': 'gruCellRecurrentBias',
              'resetAfter': false,
              'activations': ['relu', 'relu']
            }
          }
        ],
        'outputs': 'gruCellOutput'
      }],
      'expectedOutputs': {
        'gruCellOutput': {
          'data':
              [0, 0, -0.25, -3.84, -4, -15, -2.25, -3.41, -1, -3, -1, -3.41],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gruCell float32 tensors with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\'] and and explicit options.layout=\'zrn\'',
    'graph': {
      'inputs': {
        'gruCellInput': {
          'data': [1, 2, 2, 1, 1, 1],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        },
        'gruCellWeight': {
          'data': [
            1,   -1,   2, -2,  0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,
            0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,  0.5, -0.5, 0, 0.1
          ],
          'descriptor': {shape: [12, 2], dataType: 'float32'}
        },
        'gruCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [12, 4], dataType: 'float32'}
        },
        'gruCellHiddenState': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        },
        'gruCellBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        },
        'gruCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'gruCell',
        'arguments': [
          {'input': 'gruCellInput'}, {'weight': 'gruCellWeight'},
          {'recurrentWeight': 'gruCellRecurrentWeight'},
          {'hiddenState': 'gruCellHiddenState'}, {'hiddenSize': 4}, {
            'options': {
              'bias': 'gruCellBias',
              'recurrentBias': 'gruCellRecurrentBias',
              'resetAfter': false,
              'layout': 'zrn',
              'activations': ['relu', 'relu']
            }
          }
        ],
        'outputs': 'gruCellOutput'
      }],
      'expectedOutputs': {
        'gruCellOutput': {
          'data':
              [0, 0, -0.25, -3.84, -4, -15, -2.25, -3.41, -1, -3, -1, -3.41],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gruCell float32 tensors with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\'] and and options.layout=\'rzn\'',
    'graph': {
      'inputs': {
        'gruCellInput': {
          'data': [1, 2, 2, 1, 1, 1],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        },
        'gruCellWeight': {
          'data': [
            1,   -1,   2, -2,  0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,
            0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,  0.5, -0.5, 0, 0.1
          ],
          'descriptor': {shape: [12, 2], dataType: 'float32'}
        },
        'gruCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
          ],
          'descriptor': {shape: [12, 4], dataType: 'float32'}
        },
        'gruCellHiddenState': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        },
        'gruCellBias': {
          'data': [1, 1, 1, 1, 1, 2, 1, 2, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        },
        'gruCellRecurrentBias': {
          'data': [1, 1, 1, 1, 1, 2, 1, 2, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'gruCell',
        'arguments': [
          {'input': 'gruCellInput'}, {'weight': 'gruCellWeight'},
          {'recurrentWeight': 'gruCellRecurrentWeight'},
          {'hiddenState': 'gruCellHiddenState'}, {'hiddenSize': 4}, {
            'options': {
              'bias': 'gruCellBias',
              'recurrentBias': 'gruCellRecurrentBias',
              'resetAfter': false,
              'layout': 'rzn',
              'activations': ['relu', 'relu']
            }
          }
        ],
        'outputs': 'gruCellOutput'
      }],
      'expectedOutputs': {
        'gruCellOutput': {
          'data':
              [0, 0, -0.25, -3.84, -4, -15, -2.25, -3.41, -1, -3, -1, -3.41],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gruCell float32 tensors with all options',
    'graph': {
      'inputs': {
        'gruCellInput': {
          'data': [1, 2, 2, 1, 1, 1],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        },
        'gruCellWeight': {
          'data': [
            1,   -1,   2, -2,  0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,
            0.5, -0.5, 0, 0.1, 1,   -1,   2, -2,  0.5, -0.5, 0, 0.1
          ],
          'descriptor': {shape: [12, 2], dataType: 'float32'}
        },
        'gruCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
          ],
          'descriptor': {shape: [12, 4], dataType: 'float32'}
        },
        'gruCellHiddenState': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        },
        'gruCellBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        },
        'gruCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 1, 1, 1, 0.5, 0.5, 0.5, 0.5],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'gruCell',
        'arguments': [
          {'input': 'gruCellInput'}, {'weight': 'gruCellWeight'},
          {'recurrentWeight': 'gruCellRecurrentWeight'},
          {'hiddenState': 'gruCellHiddenState'}, {'hiddenSize': 4}, {
            'options': {
              'bias': 'gruCellBias',
              'recurrentBias': 'gruCellRecurrentBias',
              'resetAfter': false,
              'layout': 'zrn',
              'activations': ['relu', 'relu']
            }
          }
        ],
        'outputs': 'gruCellOutput'
      }],
      'expectedOutputs': {
        'gruCellOutput': {
          'data':
              [0, 0, -0.25, -3.84, -4, -15, -2.25, -3.41, -1, -3, -1, -3.41],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
];

if (navigator.ml) {
  gruCellTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getGruCellPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
