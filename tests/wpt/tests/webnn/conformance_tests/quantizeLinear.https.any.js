// META: title=test WebNN API quantizeLinear operation
// META: global=window
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
  const toleranceValueDict = {int32: 1, int8: 1, uint8: 1, int4: 1, uint4: 1};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const quantizeLinearTests = [
  // float32 tests
  {
    'name':
        'quantizeLinear float32 0D tensor with int8 0D zeroPoint',
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
    'name': 'quantizeLinear float32 1D constant tensor with uint8 1D zeroPoint',
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
            4.617084980010986,
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
          'data': [128, 111, 130, 133],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'quantizeLinear float32 1D constant tensor with negative scale',
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
          'descriptor': {shape: [1, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128],
          'descriptor': {shape: [1, 1], dataType: 'uint8'},
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
          'data': [0.2800687253475189, 4.617084980010986],
          'descriptor': {shape: [1, 1, 2, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128],
          'descriptor': {shape: [1, 1, 2, 1], dataType: 'uint8'},
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
          'data': [119, 111, 130, 129],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'per-tensor quantizeLinear for float32 4D constant',
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
          'data': [
            0.2800687253475189, -4.617084980010986, 0.2800687253475189,
            -4.617084980010986
          ],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {shape: [1, 1, 2, 2], dataType: 'uint8'},
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
          'data': [119, 129, 158, 127],
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
          'descriptor': {shape: [1, 2, 1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 189],
          'descriptor': {shape: [1, 2, 1], dataType: 'uint8'},
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
            {'data': [0], 'descriptor': {shape: [], dataType: 'int4'}}
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
          'descriptor': {shape: [1, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {shape: [1, 2], dataType: 'int4'},
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
          'data': [-2, -3, -4, -2, -6, -2],
          'descriptor': {shape: [3, 2], dataType: 'int4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 2D tensor with int4 zeroPoint with block_size = [3, 2]',
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
          'descriptor': {shape: [1, 2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {shape: [1, 2], dataType: 'int4'},
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
          'data': [-2, -3, -4, -2, -6, 0, -3, -3, -4, -1, -5, -2],
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
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [10],
          'descriptor': {shape: [1], dataType: 'uint4'},
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
          'data': [14, 12, 12, 10, 13],
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
    'name':
        'quantizeLinear float32 1D tensor with uint4 zeroPoint with block_size = 3',
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
  },
  {
    'name': 'quantizeLinear float32 tensor with int32 zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-22405.495643615723, 7391418.921366602],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': false
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861, 0.2800687253475189],
          'descriptor': {shape: [2], dataType: 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [32345, -2445234],
          'descriptor': {shape: [2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'},
          {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [12345, 23946213],
          'descriptor': {shape: [2], dataType: 'int32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name':
        'quantizeLinear float16 0D tensor with int8 0D zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [10.796875],
          'descriptor': {'shape': [], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875],
          'descriptor': {'shape': [], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1],
          'descriptor': {'shape': [], 'dataType': 'int8'},
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
            {'data': [11], 'descriptor': {'shape': [], 'dataType': 'int8'}}
      }
    }
  },
  {
    'name': 'quantizeLinear float16 1D constant tensor with uint8 1D zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [4], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [9.34375, 0.280029296875, 4.6171875, 1.1201171875],
          'descriptor': {'shape': [4], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {'shape': [4], 'dataType': 'uint8'},
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
          'data': [128, 111, 130, 133],
          'descriptor': {'shape': [4], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name': 'quantizeLinear float16 1D constant tensor with negative scale',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [4], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [9.34375, 0.280029296875, -4.6171875, 1.1201171875],
          'descriptor': {'shape': [4], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {'shape': [4], 'dataType': 'uint8'},
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
          'descriptor': {'shape': [4], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 2D constant tensor broadcasting zeroPoint and scale',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [2, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [9.34375],
          'descriptor': {'shape': [1, 1], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128],
          'descriptor': {'shape': [1, 1], 'dataType': 'uint8'},
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
          'descriptor': {'shape': [2, 2], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 4D constant tensor broadcasting scale and zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.280029296875, 4.6171875],
          'descriptor': {'shape': [1, 1, 2, 1], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128],
          'descriptor': {'shape': [1, 1, 2, 1], 'dataType': 'uint8'},
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
          'data': [119, 111, 130, 129],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name': 'per-tensor quantizeLinear for float16 4D constant',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.280029296875, -4.6171875, 0.280029296875, -4.6171875],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 128, 128, 128],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'uint8'},
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
          'data': [119, 129, 158, 127],
          'descriptor': {'shape': [1, 1, 2, 2], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 3D input with implicit block_size = [1, 2, 1].',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-2.548828125, -4.79296875, 8.4140625, 6.109375],
          'descriptor': {'shape': [1, 4, 1], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.280029296875, -4.6171875],
          'descriptor': {'shape': [1, 2, 1], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128, 189],
          'descriptor': {'shape': [1, 2, 1], 'dataType': 'uint8'},
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
          'descriptor': {'shape': [1, 4, 1], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 tensor with int4 zeroPoint which has odd size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.79296875],
          'descriptor': {'shape': [], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875],
          'descriptor': {'shape': [], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-4],
          'descriptor': {'shape': [], 'dataType': 'int4'},
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
            {'data': [0], 'descriptor': {'shape': [], 'dataType': 'int4'}}
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 tensor with int4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.79296875, 3.234375],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 1.1201171875],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {'shape': [2], 'dataType': 'int4'},
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
            {'data': [-2, -2], 'descriptor': {'shape': [2], 'dataType': 'int4'}}
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 2D tensor with int4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.79296875, 3.234375, 2.794921875, 5.79296875, 0, 7.234375],
          'descriptor': {'shape': [3, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 2.12109375],
          'descriptor': {'shape': [1, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {'shape': [1, 2], 'dataType': 'int4'},
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
          'data': [-2, -3, -4, -2, -6, -2],
          'descriptor': {'shape': [3, 2], 'dataType': 'int4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 2D tensor with int4 zeroPoint with block_size = [3, 2]',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.79296875, 3.234375, 2.794921875, 5.79296875, 0, 7.234375,
            4.79296875, 3.234375, 2.794921875, 5.79296875, 0, 7.234375
          ],
          'descriptor': {'shape': [3, 4], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 2.12109375],
          'descriptor': {'shape': [1, 2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [-6, -5],
          'descriptor': {'shape': [1, 2], 'dataType': 'int4'},
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
          'data': [-2, -3, -4, -2, -6, 0, -3, -3, -4, -1, -5, -2],
          'descriptor': {'shape': [3, 4], 'dataType': 'int4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 tensor with uint4 zeroPoint which has odd size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.79296875, 2.794921875, 1.794921875, 0, 3.794921875],
          'descriptor': {'shape': [5], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875],
          'descriptor': {'shape': [1], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [10],
          'descriptor': {'shape': [1], 'dataType': 'uint4'},
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
          'data': [14, 12, 12, 10, 13],
          'descriptor': {'shape': [5], 'dataType': 'uint4'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 tensor with uint4 zeroPoint which has even size',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [4.79296875, 3.234375],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 1.1201171875],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1, 5],
          'descriptor': {'shape': [2], 'dataType': 'uint4'},
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
            {'data': [5, 8], 'descriptor': {'shape': [2], 'dataType': 'uint4'}}
      }
    }
  },
  {
    'name':
        'quantizeLinear float16 1D tensor with uint4 zeroPoint with block_size = 3',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            4.794857501983643, 3.23434354545, 1.794857501983643, 2.23434354545,
            4.794857501983643, 3.23434354545
          ],
          'descriptor': {'shape': [6], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 1.1201171875],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1, 5],
          'descriptor': {'shape': [2], 'dataType': 'uint4'},
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
          'descriptor': {'shape': [6], 'dataType': 'uint4'}
        }
      }
    }
  },
  {
    'name': 'quantizeLinear float16 tensor with int32 zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [-22400, 8.4140625],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': false
        },
        'quantizeLinearScale': {
          'data': [1.1201171875, 0.280029296875],
          'descriptor': {'shape': [2], 'dataType': 'float16'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [32345, -2445234],
          'descriptor': {'shape': [2], 'dataType': 'int32'},
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
          'data': [12347, -2445204],
          'descriptor': {'shape': [2], 'dataType': 'int32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  quantizeLinearTests.filter(isTargetTest).forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getQuantizeLinearPrecisionTolerance, test,
        /*cast_to_supported_type=*/true);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
