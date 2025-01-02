// META: title=test WebNN API lstmCell operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-lstmcell
// A single time step of the Long Short-Term Memory [LSTM] recurrent network
// using a cell state, an input, output, and forget gate to compute the cell
// state and the hidden state of the next time step that rolls into the output
// across the temporal sequence of the network.
//
// enum MLRecurrentNetworkActivation {
//   "relu",
//   "sigmoid",
//   "tanh"
// };
//
// enum MLLstmWeightLayout {
//   "iofg", // input-output-forget-cell gate ordering
//   "ifgo"  // input-forget-cell-output gate ordering
// };
//
// dictionary MLLstmCellOptions : MLOperatorOptions {
//   MLOperand bias;
//   MLOperand recurrentBias;
//   MLOperand peepholeWeight;
//   MLLstmWeightLayout layout = "iofg";
//   sequence<MLRecurrentNetworkActivation> activations;
// };
//
// sequence<MLOperand> lstmCell(MLOperand input,
//                              MLOperand weight,
//                              MLOperand recurrentWeight,
//                              MLOperand hiddenState,
//                              MLOperand cellState,
//                              [EnforceRange] unsigned long hiddenSize,
//                              optional MLLstmCellOptions options = {});


const getLstmCellPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 1};
  const expectedDataType =
      graphResources
          .expectedOutputs[Object.keys(graphResources.expectedOutputs)[0]]
          .descriptor.dataType;
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const lstmCellTests = [
  {
    'name':
        'lstmCell float32 tensors with options.bias, options.recurrentBias and options.activations=[\'relu\', \'relu\', \'relu\']',
    'graph': {
      'inputs': {
        'lstmCellInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'lstmCell',
        'arguments': [
          {'input': 'lstmCellInput'}, {'weight': 'lstmCellWeight'},
          {'recurrentWeight': 'lstmCellRecurrentWeight'},
          {'hiddenState': 'lstmCellHiddenState'},
          {'cellState': 'lstmCellCellState'}, {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmCellBias',
              'recurrentBias': 'lstmCellRecurrentBias',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmCellOutput1', 'lstmCellOutput2']
      }],
      'expectedOutputs': {
        'lstmCellOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstmCell float32 tensors with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.peepholeWeight',
    'graph': {
      'inputs': {
        'lstmCellInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellPeepholeWeight': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'lstmCell',
        'arguments': [
          {'input': 'lstmCellInput'}, {'weight': 'lstmCellWeight'},
          {'recurrentWeight': 'lstmCellRecurrentWeight'},
          {'hiddenState': 'lstmCellHiddenState'},
          {'cellState': 'lstmCellCellState'}, {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmCellBias',
              'recurrentBias': 'lstmCellRecurrentBias',
              'peepholeWeight': 'lstmCellPeepholeWeight',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmCellOutput1', 'lstmCellOutput2']
      }],
      'expectedOutputs': {
        'lstmCellOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstmCell float32 tensors with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and explicit options.layout=\'iofg\'',
    'graph': {
      'inputs': {
        'lstmCellInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'lstmCell',
        'arguments': [
          {'input': 'lstmCellInput'}, {'weight': 'lstmCellWeight'},
          {'recurrentWeight': 'lstmCellRecurrentWeight'},
          {'hiddenState': 'lstmCellHiddenState'},
          {'cellState': 'lstmCellCellState'}, {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmCellBias',
              'recurrentBias': 'lstmCellRecurrentBias',
              'layout': 'iofg',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmCellOutput1', 'lstmCellOutput2']
      }],
      'expectedOutputs': {
        'lstmCellOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstmCell float32 tensors with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.layout=\'ifgo\'',
    'graph': {
      'inputs': {
        'lstmCellInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'lstmCell',
        'arguments': [
          {'input': 'lstmCellInput'}, {'weight': 'lstmCellWeight'},
          {'recurrentWeight': 'lstmCellRecurrentWeight'},
          {'hiddenState': 'lstmCellHiddenState'},
          {'cellState': 'lstmCellCellState'}, {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmCellBias',
              'recurrentBias': 'lstmCellRecurrentBias',
              'layout': 'ifgo',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmCellOutput1', 'lstmCellOutput2']
      }],
      'expectedOutputs': {
        'lstmCellOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'lstmCell float32 tensors with all options',
    'graph': {
      'inputs': {
        'lstmCellInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [8, 2], dataType: 'float32'},
          'constant': true
        },
        'lstmCellHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [8], dataType: 'float32'},
          'constant': true
        },
        'lstmCellPeepholeWeight': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'lstmCell',
        'arguments': [
          {'input': 'lstmCellInput'}, {'weight': 'lstmCellWeight'},
          {'recurrentWeight': 'lstmCellRecurrentWeight'},
          {'hiddenState': 'lstmCellHiddenState'},
          {'cellState': 'lstmCellCellState'}, {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmCellBias',
              'recurrentBias': 'lstmCellRecurrentBias',
              'peepholeWeight': 'lstmCellPeepholeWeight',
              'layout': 'iofg',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmCellOutput1', 'lstmCellOutput2']
      }],
      'expectedOutputs': {
        'lstmCellOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        },
        'lstmCellOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  lstmCellTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getLstmCellPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
