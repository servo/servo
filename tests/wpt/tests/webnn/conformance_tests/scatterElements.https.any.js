// META: title=test WebNN API scatterElements operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

const scatterElementsTests = [
  {
    'name': 'scatterElements float32 tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [1.0, 1.1, 1.2, 2.0, 2.1, 2.2],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [2.0, 1.1, 0.0, 1.0, 0.0, 2.2, 0.0, 2.1, 1.2],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 tensors along axis 0 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1.0, 1.1, 1.2, 2.0, 2.1, 2.2],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [2.0, 1.1, 0.0, 1.0, 0.0, 2.2, 0.0, 2.1, 1.2],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 tensors along axis 1',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        },
        'indices':
            {'data': [1, 3], 'descriptor': {shape: [1, 2], dataType: 'int32'}},
        'updates': {
          'data': [1.1, 2.1],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1.0, 1.1, 3.0, 2.1, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 tensors along axis 1 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 3],
          'descriptor': {shape: [1, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1.1, 2.1],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1.0, 1.1, 3.0, 2.1, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'scatterElements float16 tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [1, 1.099609375, 1.2001953125, 2, 2.099609375, 2.19921875],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            2, 1.099609375, 0, 1, 0, 2.19921875, 0, 2.099609375, 1.2001953125
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 tensors along axis 0 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1, 1.099609375, 1.2001953125, 2, 2.099609375, 2.19921875],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            2, 1.099609375, 0, 1, 0, 2.19921875, 0, 2.099609375, 1.2001953125
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 tensors along axis 1',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5],
          'descriptor': {shape: [1, 5], dataType: 'float16'}
        },
        'indices':
            {'data': [1, 3], 'descriptor': {shape: [1, 2], dataType: 'int32'}},
        'updates': {
          'data': [1.099609375, 2.099609375],
          'descriptor': {shape: [1, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 1.099609375, 3, 2.099609375, 5],
          'descriptor': {shape: [1, 5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 tensors along axis 1 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5],
          'descriptor': {shape: [1, 5], dataType: 'float16'}
        },
        'indices': {
          'data': [1, 3],
          'descriptor': {shape: [1, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1.099609375, 2.099609375],
          'descriptor': {shape: [1, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 1.099609375, 3, 2.099609375, 5],
          'descriptor': {shape: [1, 5], dataType: 'float16'}
        }
      }
    }
  }
];

webnn_conformance_test(
    scatterElementsTests, buildAndExecuteGraph, getZeroULPTolerance);
