// META: title=test WebNN API Element-wise logical isInfinite operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-logical
// Check if the values of the input tensor are infinite, element-wise.
//
// MLOperand isInfinite(MLOperand input);

const isInfiniteTests = [
  // isInfinite tests
  {
    'name': 'isInfinite float32 positive 0D scalar',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0],
          'descriptor': {shape: [], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 1D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, -Infinity],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 1],
          'descriptor': {shape: [2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 2D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, Infinity, -2.5, -Infinity],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 1, 0, 1],
          'descriptor': {shape: [2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 3D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, Infinity, -2.5, 0.0, NaN, -Infinity, 3.14, Infinity],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 1, 0, 0, 0, 1, 0, 1],
          'descriptor': {shape: [2, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 4D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, Infinity, -2.5, 0.0, NaN, -Infinity, 3.14, Infinity,
                   -0.0, 42.0, -Infinity, -999.99, 1e10, -1e-10, Infinity, 100.5],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 special values',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, -Infinity, NaN, 0.0,
            -0.0, 1.0, -1.0, 3.40282346638528859811704e+38],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 1, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [8], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 all infinite values',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, -Infinity, Infinity, -Infinity],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 1, 1, 1],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 no infinite values',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, 2.5, -3.7, 0.0, -0.0, NaN],
          'descriptor': {shape: [6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 positive infinity only',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, 1.0, 2.5, -3.7],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 0, 0, 0],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 negative infinity only',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, -Infinity, 2.5, -3.7],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 1, 0, 0],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float32 large finite values',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1e38, -1e38, 3.40282346638528859811704e+38,
            -3.40282346638528859811704e+38, 1e39, -1e39],
          'descriptor': {shape: [6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 0, 0, 0, 1, 1],
          'descriptor': {shape: [6], dataType: 'uint8'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'isInfinite float16 positive 0D scalar',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0],
          'descriptor': {shape: [], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float16 1D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, -Infinity],
          'descriptor': {shape: [2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 1],
          'descriptor': {shape: [2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float16 2D tensor',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [1.0, Infinity, -2.5, -Infinity],
          'descriptor': {shape: [2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [0, 1, 0, 1],
          'descriptor': {shape: [2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isInfinite float16 special values',
    'graph': {
      'inputs': {
        'isInfiniteInput': {
          'data': [Infinity, -Infinity, NaN, 0.0, -0.0, 1.0, -1.0],
          'descriptor': {shape: [7], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isInfinite',
        'arguments': [{'input': 'isInfiniteInput'}],
        'outputs': 'isInfiniteOutput'
      }],
      'expectedOutputs': {
        'isInfiniteOutput': {
          'data': [1, 1, 0, 0, 0, 0, 0],
          'descriptor': {shape: [7], dataType: 'uint8'}
        }
      }
    }
  },
]

webnn_conformance_test(
    isInfiniteTests, buildAndExecuteGraph, getZeroULPTolerance);
