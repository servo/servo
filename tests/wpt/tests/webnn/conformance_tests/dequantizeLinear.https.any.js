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
          'data': [128],
          'descriptor': {shape: [], dataType: 'uint8'},
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
          'data': [12],
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
          'data': [0],
          'descriptor': {shape: [], dataType: 'uint4'},
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
          'data': [16.804121017456055, 0],
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
          'data': [2, 1, 4],
          'descriptor': {shape: [3], dataType: 'uint4'},
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
          'data': [8.962198257446289, 12.323022842407227, 11.202747344970703],
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
          'data': [
            -18.686185836791992, -18.686185836791992, -36.93667984008789,
            -55.40502166748047
          ],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'}
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
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
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
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'dequantizeLinearZeroPoint': {
          'data': [-3, 0, 0],
          'descriptor': {shape: [3], dataType: 'int4'},
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
          'data': [2.2405495643615723, 7.841923713684082, 0],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  dequantizeLinearTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getDequantizeLinearPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
