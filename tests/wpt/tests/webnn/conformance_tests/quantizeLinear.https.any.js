// META: title=test WebNN API quantizeLinear operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Calculate a floating-point input down to a low-precision integer
// (typically uint8 with a zero-point bias) following the expression:
//    output = clamp(roundToNearestEvens(input / scale) + zeroPoint, 0, 255).
//
// MLOperand quantizeLinear(
//     MLOperand input, MLOperand scale, MLOperand zeroPoint,
//     optional MLOperatorOptions options = {});


const getQuantizeLinearPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {int8: 1, uint8: 1, int4: 1, uint4: 1};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const quantizeLinearTests = [
  {
    'name':
        'quantizeLinear float32 0D scalar tensor with int8 scalar zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [10.794857501983643],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput':
            {'data': [11], 'descriptor': {shape: [], dataType: 'int8'}}
      }
    }
  },
  {
    'name': 'quantizeLinear float32 1D constant tensor broadcasting zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {shape: [4], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [
            9.343092918395996,
            0.2800687253475189,
            -4.617084980010986,
            1.1202747821807861,
          ],
          'descriptor': {shape: [4], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [128, 111, 126, 133],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 2D constant tensor broadcasting zeroPoint and scale',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [9.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128],
          'descriptor': {shape: [1], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [128, 127, 129, 129],
          'descriptor': {shape: [2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 4D constant tensor broadcasting scale and zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.2800687253475189, -4.617084980010986],
          'descriptor': {shape: [2, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128],
          'descriptor': {shape: [2, 1], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [119, 111, 126, 127],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 3D input with implicit block_size = [1, 2, 1].',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {shape: [1, 4, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.2800687253475189, -4.617084980010986],
          'descriptor': {shape: [2, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 189],
          'descriptor': {shape: [2, 1], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [119, 111, 187, 188],
          'descriptor': {shape: [1, 4, 1], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 tensor with int4 zeroPoint which has odd size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.794857501983643],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-4],
          'descriptor': {shape: [], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput':
            {'data': [-1], 'descriptor': {shape: [], dataType: 'int4'}}
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 tensor with int4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.794857501983643, 3.23434354545],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {shape: [2], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput':
            {'data': [-2, -2], 'descriptor': {shape: [2], dataType: 'int4'}}
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 2D tensor with int4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.794857501983643, 3.23434354545, 2.794857501983643,
            5.794857501983643, 0, 7.23434354545
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 2.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {shape: [2], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [-2, -3, -4, -3, -5, -2],
          'descriptor': {shape: [3, 2], dataType: 'int4'}
        }
      }
    }
  },
  {
    'name': 'quantizeLinear int4 zeroPoint with block_size = [3, 2]',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.794857501983643, 3.23434354545, 2.794857501983643,
            5.794857501983643, 0, 7.23434354545, 4.794857501983643,
            3.23434354545, 2.794857501983643, 5.794857501983643, 0,
            7.23434354545
          ],
          'descriptor': {shape: [3, 4], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 2.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {shape: [2], dataType: 'int4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [-2, -3, -4, -3, -5, 0, -2, -3, -4, -1, -5, -2],
          'descriptor': {shape: [3, 4], dataType: 'int4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 tensor with uint4 zeroPoint which has odd size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.794857501983643, 2.794857501983643, 1.794857501983643, 0,
            3.794857501983643
          ],
          'descriptor': {shape: [5], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [12],
          'descriptor': {shape: [], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [16, 14, 13, 12, 15],
          'descriptor': {shape: [5], dataType: 'uint4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 tensor with uint4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.794857501983643, 3.23434354545],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1, 5],
          'descriptor': {shape: [2], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput':
            {'data': [5, 8], 'descriptor': {shape: [2], dataType: 'uint4'}}
      }
    }
  },
  {
    'name': 'quantizeLinear uint4 zeroPoint with block_size = 3',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.794857501983643, 3.23434354545, 1.794857501983643, 2.23434354545,
            4.794857501983643, 3.23434354545
          ],
          'descriptor': {shape: [6], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 1.1202747821807861],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1, 5],
          'descriptor': {shape: [2], dataType: 'uint4'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [5, 4, 3, 7, 9, 8],
          'descriptor': {shape: [6], dataType: 'uint4'}
        }
      }
    }
  }
];

if (navigator.ml) {
  quantizeLinearTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getQuantizeLinearPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
