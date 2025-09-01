// META: title=test WebNN API Element-wise logical isNaN operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-logical
// Check if the values of the input tensor are invalid numeric representations
// (NaN’s), element-wise.
//
// MLOperand isNaN(MLOperand input);

const isNaNTests = [
  // isNaN tests
  {
    'name': 'isNaN float32 positive 0D scalar',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0],
          'descriptor': {shape: [], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 1D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.5, NaN],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1],
          'descriptor': {shape: [2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 2D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.0, NaN, -2.5, 0.0],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 3D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.0, NaN, -2.5, 0.0, Infinity, -Infinity, 3.14, NaN],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1, 0, 0, 0, 0, 0, 1],
          'descriptor': {shape: [2, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 4D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.0, NaN, -2.5, 0.0, Infinity, -Infinity, 3.14, NaN,
                   -0.0, 42.0, NaN, -999.99, 1e10, -1e-10, NaN, 100.5],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 special values',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [NaN, Infinity, -Infinity, 0.0, -0.0, 1.0, -1.0],
          'descriptor': {shape: [7], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [1, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [7], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 all NaN values',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [NaN, NaN, NaN, NaN],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [1, 1, 1, 1],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float32 no NaN values',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.0, 2.5, -3.7, 0.0, Infinity, -Infinity],
          'descriptor': {shape: [6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'uint8'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'isNaN float16 positive 0D scalar',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0],
          'descriptor': {shape: [], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float16 1D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.5, NaN],
          'descriptor': {shape: [2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1],
          'descriptor': {shape: [2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float16 2D tensor',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [1.0, NaN, -2.5, 0.0],
          'descriptor': {shape: [2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [0, 1, 0, 0],
          'descriptor': {shape: [2, 2], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'isNaN float16 special values',
    'graph': {
      'inputs': {
        'isNaNInput': {
          'data': [NaN, Infinity, -Infinity, 0.0, -0.0, 1.0, -1.0],
          'descriptor': {shape: [7], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'isNaN',
        'arguments': [{'input': 'isNaNInput'}],
        'outputs': 'isNaNOutput'
      }],
      'expectedOutputs': {
        'isNaNOutput': {
          'data': [1, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [7], dataType: 'uint8'}
        }
      }
    }
  },
]

if (navigator.ml) {
  isNaNTests.filter(isTargetTest).forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getZeroULPTolerance, test,
        /*cast_to_supported_type=*/true);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
