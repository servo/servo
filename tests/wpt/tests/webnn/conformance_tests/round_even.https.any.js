// META: title=test WebNN API element-wise roundEven operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-unary
// Round the input tensor with halves to the nearest even value, element-wise.
//
// MLOperand roundEven(MLOperand input);
const getRoundEvenPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const roundEvenTests = [
  // roundEven tests
  {
    'name': 'roundEven float32 positive 0D scalar',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [2],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'roundEven float32 negative 1D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-1.5],
          'descriptor': {shape: [1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2],
          'descriptor': {shape: [1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'roundEven float32 2D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-1.5, 0.5],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, 0],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'roundEven float32 3D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-2.5, -1.5, 0.5, 1.5],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, -2, 0, 2],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'roundEven float32 4D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-2.5, -1.5, 0.5, 1.5],
          'descriptor': {shape: [1, 2, 1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, -2, 0, 2],
          'descriptor': {shape: [1, 2, 1, 2], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'roundEven float16 positive 0D scalar',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [1.5],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [2],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'roundEven float16 negative 1D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-1.5],
          'descriptor': {shape: [1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2],
          'descriptor': {shape: [1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'roundEven float16 2D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-1.5, 0.5],
          'descriptor': {shape: [1, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, 0],
          'descriptor': {shape: [1, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'roundEven float16 3D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-2.5, -1.5, 0.5, 1.5],
          'descriptor': {shape: [1, 2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, -2, 0, 2],
          'descriptor': {shape: [1, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'roundEven float16 4D tensor',
    'graph': {
      'inputs': {
        'roundEvenInput': {
          'data': [-2.5, -1.5, 0.5, 1.5],
          'descriptor': {shape: [1, 2, 1, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'roundEven',
        'arguments': [{'input': 'roundEvenInput'}],
        'outputs': 'roundEvenOutput'
      }],
      'expectedOutputs': {
        'roundEvenOutput': {
          'data': [-2, -2, 0, 2],
          'descriptor': {shape: [1, 2, 1, 2], dataType: 'float16'}
        }
      }
    }
  },
]

webnn_conformance_test(
    roundEvenTests, buildAndExecuteGraph, getRoundEvenPrecisionTolerance);
