// META: title=test WebNN API lstm operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-lstm
// Long Short-Term Memory [LSTM] recurrent network uses an input, output,
// forget, and cell gate to compute the output state that rolls into the output
// across the temporal sequence of the network.
// enum MLRecurrentNetworkDirection {
//   "forward",
//   "backward",
//   "both"
// };
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
// dictionary MLLstmOptions {
//   MLOperand bias;
//   MLOperand recurrentBias;
//   MLOperand peepholeWeight;
//   MLOperand initialHiddenState;
//   MLOperand initialCellState;
//   boolean returnSequence = false;
//   MLRecurrentNetworkDirection direction = "forward";
//   MLLstmWeightLayout layout = "iofg";
//   sequence<MLRecurrentNetworkActivation> activations;
// };
//
// sequence<MLOperand> lstm(MLOperand input,
//                          MLOperand weight,
//                          MLOperand recurrentWeight,
//                          [EnforceRange] unsigned long steps,
//                          [EnforceRange] unsigned long hiddenSize,
//                          optional MLLstmOptions options = {});


const getLstmPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 1};
  const expectedDataType =
      graphResources
          .expectedOutputs[Object.keys(graphResources.expectedOutputs)[0]]
          .descriptor.dataType;
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const lstmTests = [
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias and options.activations=[\'relu\', \'relu\', \'relu\']',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.peepholeWeight',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmPeepholeWeight': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'peepholeWeight': 'lstmPeepholeWeight',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.initialHiddenState',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmInitialHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'initialHiddenState': 'lstmInitialHiddenState',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.initialCellState',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmInitialCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'initialCellState': 'lstmInitialCellState',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and explicit options.returnSequence=false',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'returnSequence': false,
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.returnSequence=true',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'returnSequence': true,
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2', 'lstmOutput3']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput3': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and explicit options.direction=\'forward\'',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'direction': 'forward',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and explicit options.layout=\'iofg\'',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'layout': 'iofg',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=1 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.layout=\'ifgo\'',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'layout': 'ifgo',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'lstm float32 tensors steps=1 with all options',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmPeepholeWeight': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 6], dataType: 'float32'}
        },
        'lstmInitialHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmInitialCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 1},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'peepholeWeight': 'lstmPeepholeWeight',
              'initialHiddenState': 'lstmInitialHiddenState',
              'initialCellState': 'lstmInitialCellState',
              'returnSequence': true,
              'direction': 'forward',
              'layout': 'iofg',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2', 'lstmOutput3']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [1, 4, 9, 36],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput3': {
          'data': [1, 8, 27, 216],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'lstm float32 tensors steps=2 with options.bias, options.recurrentBias, options.activations=[\'relu\', \'relu\', \'relu\'] and options.direction=\'backward\'',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1, 3, 4, 1, 2],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 2},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'direction': 'backward',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [
            10.469000816345215, 58.02900695800781, 74.52900695800781,
            518.948974609375
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [
            5.510000228881836, 20.01000213623047, 19.110000610351564,
            75.20999908447266
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'lstm float32 tensors steps=2 with all options',
    'graph': {
      'inputs': {
        'lstmInput': {
          'data': [1, 2, 2, 1, 3, 4, 1, 2],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'lstmWeight': {
          'data': [1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2, 1, -1, 2, -2],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmRecurrentWeight': {
          'data': [
            0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            0.1, 0.1, 0.1
          ],
          'descriptor': {shape: [1, 8, 2], dataType: 'float32'}
        },
        'lstmBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmRecurrentBias': {
          'data': [1, 2, 1, 2, 1, 2, 1, 2],
          'descriptor': {shape: [1, 8], dataType: 'float32'}
        },
        'lstmPeepholeWeight': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 6], dataType: 'float32'}
        },
        'lstmInitialHiddenState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmInitialCellState': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'lstm',
        'arguments': [
          {'input': 'lstmInput'}, {'weight': 'lstmWeight'},
          {'recurrentWeight': 'lstmRecurrentWeight'}, {'steps': 2},
          {'hiddenSize': 2}, {
            'options': {
              'bias': 'lstmBias',
              'recurrentBias': 'lstmRecurrentBias',
              'peepholeWeight': 'lstmPeepholeWeight',
              'initialHiddenState': 'lstmInitialHiddenState',
              'initialCellState': 'lstmInitialCellState',
              'returnSequence': true,
              'direction': 'backward',
              'layout': 'iofg',
              'activations': ['relu', 'relu', 'relu']
            }
          }
        ],
        'outputs': ['lstmOutput1', 'lstmOutput2', 'lstmOutput3']
      }],
      'expectedOutputs': {
        'lstmOutput1': {
          'data': [
            10.469000816345215, 58.02900695800781, 74.52900695800781,
            518.948974609375
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput2': {
          'data': [
            5.510000228881836, 20.01000213623047, 19.110000610351564,
            75.20999908447266
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        },
        'lstmOutput3': {
          'data': [
            10.469000816345215, 58.02900695800781, 74.52900695800781,
            518.948974609375, 1, 8, 1, 8
          ],
          'descriptor': {shape: [2, 1, 2, 2], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  lstmTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getLstmPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
