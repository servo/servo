// META: title=test WebNN API slice operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-slice
// Produce a slice of the input tensor.
//
// MLOperand slice(
//     MLOperand input, sequence<[EnforceRange] unsigned long>starts,
//     sequence<[EnforceRange] unsigned long>sizes);


const getSlicePrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const sliceTests = [
  {
    'name': 'slice float32 1D constant tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.5424534678459167, 80.44634246826172, 28.32765007019043,
            74.02619171142578, -74.54559326171875, -27.306041717529297,
            -70.42774200439453, 59.82632064819336, -58.46095275878906,
            79.80570983886719, -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 1D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.5424534678459167, 80.44634246826172, 28.32765007019043,
            74.02619171142578, -74.54559326171875, -27.306041717529297,
            -70.42774200439453, 59.82632064819336, -58.46095275878906,
            79.80570983886719, -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [12], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 2D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [2, 2]}, {'sizes': [2, 4]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            28.32765007019043, 74.02619171142578, -74.54559326171875,
            -27.306041717529297, -58.46095275878906, 79.80570983886719,
            -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 3D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [4, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 1, 1]}, {'sizes': [3, 2, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            57.4397087097168, -4.513182163238525, 74.02619171142578,
            -27.306041717529297, 79.80570983886719, 42.665199279785156
          ],
          'descriptor': {shape: [3, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 4D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1]},
          {'sizes': [1, 2, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.306041717529297, 42.665199279785156],
          'descriptor': {shape: [1, 2, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 5D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 3, 2, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1, 0]},
          {'sizes': [1, 2, 1, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.306041717529297, 42.665199279785156],
          'descriptor': {shape: [1, 2, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 2D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -70.42774200439453, 42.665199279785156
          ],
          'descriptor': {shape: [2, 12], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [0, 2]}, {'sizes': [2, 10]},
          {'options': {'strides': [1, 4]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            -68.15961456298828, -24.556316375732422, 74.41775512695312,
            28.32765007019043, -70.42774200439453, -70.42774200439453
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 3D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [4, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [0, 0, 1]}, {'sizes': [4, 3, 1]},
          {'options': {'strides': [3, 2, 1]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            97.95414733886719, 76.59095764160156, 59.82632064819336,
            42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'slice float32 4D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.846250534057617,  97.95414733886719,  -68.15961456298828,
            14.978987693786621,  90.23090362548828,  76.59095764160156,
            -24.556316375732422, 79.58749389648438,  65.21376037597656,
            57.4397087097168,    74.41775512695312,  -4.513182163238525,
            0.5424534678459167,  80.44634246826172,  28.32765007019043,
            74.02619171142578,   -74.54559326171875, -27.306041717529297,
            -70.42774200439453,  59.82632064819336,  -58.46095275878906,
            79.80570983886719,   -9.857853889465332, 42.665199279785156
          ],
          'descriptor': {shape: [2, 2, 3, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 1, 1, 1]},
          {'sizes': [1, 1, 1, 1]}, {'options': {'strides': [2, 2, 2, 2]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [79.80570983886719],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'slice float16 1D constant tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.54248046875, 80.4375, 28.328125, 74, -74.5625, -27.3125, -70.4375,
            59.8125, -58.46875, 79.8125, -9.859375, 42.65625
          ],
          'descriptor': {shape: [12], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 1D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [12]}, {'sizes': [12]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            0.54248046875, 80.4375, 28.328125, 74, -74.5625, -27.3125, -70.4375,
            59.8125, -58.46875, 79.8125, -9.859375, 42.65625
          ],
          'descriptor': {shape: [12], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 2D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments':
            [{'input': 'sliceInput'}, {'starts': [2, 2]}, {'sizes': [2, 4]}],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [
            28.328125, 74, -74.5625, -27.3125, -58.46875, 79.8125, -9.859375,
            42.65625
          ],
          'descriptor': {shape: [2, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 3D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [4, 3, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 1, 1]}, {'sizes': [3, 2, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [57.4375, -4.51171875, 74, -27.3125, 79.8125, 42.65625],
          'descriptor': {shape: [3, 2, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 4D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [2, 2, 3, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1]},
          {'sizes': [1, 2, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.3125, 42.65625],
          'descriptor': {shape: [1, 2, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 5D tensor',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [2, 2, 3, 2, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 0, 2, 1, 0]},
          {'sizes': [1, 2, 1, 1, 1]}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-27.3125, 42.65625],
          'descriptor': {shape: [1, 2, 1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 2D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -70.4375,      42.65625
          ],
          'descriptor': {shape: [2, 12], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [0, 2]}, {'sizes': [2, 10]},
          {'options': {'strides': [1, 4]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [-68.1875, -24.5625, 74.4375, 28.328125, -70.4375, -70.4375],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 3D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [4, 3, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [0, 0, 1]}, {'sizes': [4, 3, 1]},
          {'options': {'strides': [3, 2, 1]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [97.9375, 76.5625, 59.8125, 42.65625],
          'descriptor': {shape: [2, 2, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'slice float16 4D tensor with strides',
    'graph': {
      'inputs': {
        'sliceInput': {
          'data': [
            28.84375,  97.9375,     -68.1875,      14.9765625, 90.25,
            76.5625,   -24.5625,    79.5625,       65.1875,    57.4375,
            74.4375,   -4.51171875, 0.54248046875, 80.4375,    28.328125,
            74,        -74.5625,    -27.3125,      -70.4375,   59.8125,
            -58.46875, 79.8125,     -9.859375,     42.65625
          ],
          'descriptor': {shape: [2, 2, 3, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'slice',
        'arguments': [
          {'input': 'sliceInput'}, {'starts': [1, 1, 1, 1]},
          {'sizes': [1, 1, 1, 1]}, {'options': {'strides': [2, 2, 2, 2]}}
        ],
        'outputs': 'sliceOutput'
      }],
      'expectedOutputs': {
        'sliceOutput': {
          'data': [79.8125],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float16'}
        }
      }
    }
  }
];

if (navigator.ml) {
  sliceTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getSlicePrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
