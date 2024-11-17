// META: title=test WebNN API dequantizeLinear operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Calculate a low precision integer operand
// (typically uint8 with a zero-point bias) to floating point:
//   output = (input - zeroPoint) * scale.
//
// MLOperand dequantizeLinear(
//     MLOperand input, MLOperand scale, MLOperand zeroPoint,
//     optional MLOperatorOptions options = {});


const getDequantizeLinearPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 1};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const dequantizeLinearTests = [
  {
    'name': 'dequantizeLinear int8 0D tensor with float32 scalar scale',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [123],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': false
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [3],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [134.43296813964844],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear constant input',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [123],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [3],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [134.43296813964844],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear uint8 1D constant tensor broadcasting zeroPoint',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [12, 24, 35, 123],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [
            9.343092918395996,
            0.2800687253475189,
            -4.617084980010986,
            1.1202747821807861,
          ],
          'descriptor': {shape: [4], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -1083.798828125, -29.127147674560547, 429.388916015625,
            -5.601373672485352
          ],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'dequantizeLinear uint8 1D constant tensor with implicit block_size = 2.',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [12, 24, 35, 123],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [
            9.343092918395996,
            -4.617084980010986,
          ],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [128, 110],
          'descriptor': {shape: [2], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -1083.798828125, -971.681640625, 346.2813720703125,
            -60.0221061706543
          ],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'dequantizeLinear int8 4D constant tensor broadcasting scale and zeroPoint',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [-124, 0, 23, 122],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'int8'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [0.2800687253475189, -4.617084980010986],
          'descriptor': {shape: [2, 1], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [12, 12],
          'descriptor': {shape: [2, 1], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -38.08934783935547, -3.3608245849609375, -50.787933349609375,
            -507.87933349609375
          ],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear int8 4D constant tensor with block_size = [3, 2]',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [
            -124, 0,   23,  122, 12, 23, 45, 36, 67, 78, -22, 0,
            -34,  -45, -56, -67, 89, 30, 12, 23, 56, 67, 56,  -12
          ],
          'descriptor': {shape: [6, 4], dataType: 'int8'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [
            0.2800687253475189, -4.617084980010986, 1.2800687253475189,
            -3.617084980010986
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [1, 3, 5, 12],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -35.00859069824219,
            -0.2800687253475189,
            -92.3416976928711,
            -549.43310546875,
            3.0807559490203857,
            6.1615118980407715,
            -193.91757202148438,
            -152.36380004882812,
            18.484535217285156,
            21.565292358398438,
            115.4271240234375,
            13.851255416870117,
            -49.92267990112305,
            -64.0034408569336,
            245.96177673339844,
            285.7497253417969,
            107.52577209472656,
            32.0017204284668,
            0,
            -39.787933349609375,
            65.28350830078125,
            79.36426544189453,
            -159.1517333984375,
            86.81004333496094
          ],
          'descriptor': {shape: [6, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear uint4 1D tensor with even input size',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [15, 0],
          'descriptor': {shape: [2], dataType: 'uint4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861, 1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [0, 1],
          'descriptor': {shape: [2], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [16.804121017456055, -1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear uint4 1D tensor with odd input size',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [10, 12, 14],
          'descriptor': {shape: [3], dataType: 'uint4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [2],
          'descriptor': {shape: [1], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [8.962198257446289, 11.202747344970703, 13.443297386169434],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear uint4 4D constant tensor broadcasting zeroPoint',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [0, 1, 10, 15],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'uint4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [
            9.343092918395996,
            -4.617084980010986,
          ],
          'descriptor': {shape: [2, 1], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [2, 3],
          'descriptor': {shape: [2, 1], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -18.686185836791992, -9.343092918395996, -32.31959533691406,
            -55.40502166748047
          ],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear uint4 3D input with block_size = [1, 1, 2]',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [0, 1, 10, 15],
          'descriptor': {shape: [1, 1, 4], dataType: 'uint4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [
            9.343092918395996,
            -4.617084980010986,
          ],
          'descriptor': {shape: [1, 2], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [2, 3],
          'descriptor': {shape: [1, 2], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [
            -18.686185836791992, -9.343092918395996, -32.31959533691406,
            -55.40502166748047
          ],
          'descriptor': {shape: [1, 1, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear int4 1D tensor with even size',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [-8, -3],
          'descriptor': {shape: [2], dataType: 'int4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861, 1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [0, -2],
          'descriptor': {shape: [2], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [-8.962198257446289, -1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear int4 1D tensor with odd size',
    'graph': {
      'inputs': {
        'dequantizeLinearInput': {
          'data': [-1, 7, 0],
          'descriptor': {shape: [3], dataType: 'int4'},
          'constant': true
        },
        'dequantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [-3],
          'descriptor': {shape: [1], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'dequantizeLinear',
        'arguments': [
          {'input': 'dequantizeLinearInput'},
          {'scale': 'dequantizeLinearScale'},
          {'zeroPoint': 'dequantizeLinearZeroPoint'}
        ],
        'outputs': 'dequantizeLinearOutput'
      }],
      'expectedOutputs': {
        'dequantizeLinearOutput': {
          'data': [2.2405495643615723, 11.202747344970703, 3.3608243465423584],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  dequantizeLinearTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getDequantizeLinearPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
