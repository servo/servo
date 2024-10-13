// META: title=test WebNN API element-wise logicalOr operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// MLOperand logicalOr(MLOperand a, MLOperand b, optional MLOperatorOptions options = {});

const logicalOrTests = [
  {
    'name': 'logicalOr uint8 0D scalar',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [1],
          'descriptor': {shape: [], dataType: 'uint8'}
        },
        'inputB': {
          'data': [1],
          'descriptor': {shape: [], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {'data': [1], 'descriptor': {shape: [], dataType: 'uint8'}}
      }
    }
  },
  {
    'name': 'logicalOr uint8 1D constant tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [24], dataType: 'uint8'},
          'constant': true
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [24], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 1D tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 2D tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 3D tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 4D tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 5D tensors',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [2, 2, 1, 2, 3], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 2, 1, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 1, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 broadcast 0D to 4D',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [1],
          'descriptor': {shape: [], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 broadcast 1D to 4D',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [1],
          'descriptor': {shape: [1], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 broadcast 2D to 4D',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 255, 255
          ],
          'descriptor': {shape: [2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 broadcast 3D to 4D',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 8, 8,
            0, 0, 8, 8, 0, 0, 255, 255, 0, 0, 255, 255
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 255
          ],
          'descriptor': {shape: [2, 2, 1], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalOr uint8 broadcast 4D to 4D',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [1],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'uint8'}
        },
        'inputB': {
          'data': [
            0, 1, 0, 1, 0, 2, 0, 2, 0, 8, 0, 8,
            0, 2, 0, 2, 0, 255, 0, 255, 0, 8, 0, 8
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalOr',
        'arguments': [{'a': 'inputA'}, {'b': 'inputB'}],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  }
];

if (navigator.ml) {
  logicalOrTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
