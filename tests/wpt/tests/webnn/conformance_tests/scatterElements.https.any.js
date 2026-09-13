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
    'name': 'scatterElements float32 2D tensors along axis 0',
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
    'name': 'scatterElements float32 2D tensors along axis 0 and constant indices',
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
    'name': 'scatterElements float32 2D tensors along axis 1',
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
    'name': 'scatterElements float32 2D tensors along axis 1 and constant indices',
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
  {
    'name': 'scatterElements float32 2D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'indices': {
          'data': [-2, -3, -1, -3, -1, -2],
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
          'data': [2.0, 1.1, 3.0, 1.0, 5.0, 2.2, 7.0, 2.1, 1.2],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 1D tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data': [10, 20, 30, 40, 50],
          'descriptor': {shape: [5], dataType: 'float32'}
        },
        'indices': {
          'data': [4, 0, 2],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 200, 300],
          'descriptor': {shape: [3], dataType: 'float32'}
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
          'data': [200, 20, 300, 40, 100],
          'descriptor': {shape: [5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 1D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [10, 20, 30, 40, 50],
          'descriptor': {shape: [5], dataType: 'float32'}
        },
        'indices': {
          'data': [-1, -5, -3],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 200, 300],
          'descriptor': {shape: [3], dataType: 'float32'}
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
          'data': [200, 20, 300, 40, 100],
          'descriptor': {shape: [5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 3D tensors along axis 1',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [0, 1, 2, 0, 1, 2, 0, 1],
          'descriptor': {shape: [2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 101, 102, 103, 200, 201, 202, 203],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
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
          'data': [100, 103, 3, 101, 102, 6, 202, 8, 200, 203, 11, 201],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 3D tensors along axis 1 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [-3, -2, -1, -3, -2, -1, -3, -2],
          'descriptor': {shape: [2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 101, 102, 103, 200, 201, 202, 203],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
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
          'data': [100, 103, 3, 101, 102, 6, 202, 8, 200, 203, 11, 201],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 4D tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [
            100, 101, 102, 103, 104, 105, 106, 107,
            200, 201, 202, 203, 204, 205, 206, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
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
            200, 101, 202, 103, 204, 105, 206, 107,
            100, 201, 102, 203, 104, 205, 106, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 4D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [
            -1, -2, -1, -2, -1, -2, -1, -2,
            -2, -1, -2, -1, -2, -1, -2, -1
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [
            100, 101, 102, 103, 104, 105, 106, 107,
            200, 201, 202, 203, 204, 205, 206, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
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
            200, 101, 202, 103, 204, 105, 206, 107,
            100, 201, 102, 203, 104, 205, 106, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 5D tensors along axis 2',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        },
        'indices': {
          'data': [2, 2, 0, 0, 2, 2, 0, 0],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [11, 12, 13, 14, 15, 16, 17, 18],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 2}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [13, 14, 3, 4, 11, 12, 17, 18, 9, 10, 15, 16],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float32 5D tensors along axis 2 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        },
        'indices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [11, 12, 13, 14, 15, 16, 17, 18],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 2}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [13, 14, 3, 4, 11, 12, 17, 18, 9, 10, 15, 16],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'scatterElements float16 2D tensors along axis 0',
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
    'name': 'scatterElements float16 2D tensors along axis 0 and constant indices',
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
    'name': 'scatterElements float16 2D tensors along axis 1',
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
    'name': 'scatterElements float16 2D tensors along axis 1 and constant indices',
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
  },
  {
    'name': 'scatterElements float16 2D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        },
        'indices': {
          'data': [-2, -3, -1, -3, -1, -2],
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
            2, 1.099609375, 3, 1, 5, 2.19921875, 7, 2.099609375, 1.2001953125
          ],
          'descriptor': {shape: [3, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 1D tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data': [10, 20, 30, 40, 50],
          'descriptor': {shape: [5], dataType: 'float16'}
        },
        'indices': {
          'data': [4, 0, 2],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 200, 300],
          'descriptor': {shape: [3], dataType: 'float16'}
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
          'data': [200, 20, 300, 40, 100],
          'descriptor': {shape: [5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 1D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [10, 20, 30, 40, 50],
          'descriptor': {shape: [5], dataType: 'float16'}
        },
        'indices': {
          'data': [-1, -5, -3],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 200, 300],
          'descriptor': {shape: [3], dataType: 'float16'}
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
          'data': [200, 20, 300, 40, 100],
          'descriptor': {shape: [5], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 3D tensors along axis 1',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [2, 3, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [0, 1, 2, 0, 1, 2, 0, 1],
          'descriptor': {shape: [2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 101, 102, 103, 200, 201, 202, 203],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
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
          'data': [100, 103, 3, 101, 102, 6, 202, 8, 200, 203, 11, 201],
          'descriptor': {shape: [2, 3, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 3D tensors along axis 1 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [2, 3, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [-3, -2, -1, -3, -2, -1, -3, -2],
          'descriptor': {shape: [2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [100, 101, 102, 103, 200, 201, 202, 203],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
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
          'data': [100, 103, 3, 101, 102, 6, 202, 8, 200, 203, 11, 201],
          'descriptor': {shape: [2, 3, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 4D tensors along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [
            100, 101, 102, 103, 104, 105, 106, 107,
            200, 201, 202, 203, 204, 205, 206, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
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
            200, 101, 202, 103, 204, 105, 206, 107,
            100, 201, 102, 203, 104, 205, 106, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 4D tensors along axis 0 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [
            -1, -2, -1, -2, -1, -2, -1, -2,
            -2, -1, -2, -1, -2, -1, -2, -1
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [
            100, 101, 102, 103, 104, 105, 106, 107,
            200, 201, 202, 203, 204, 205, 206, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
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
            200, 101, 202, 103, 204, 105, 206, 107,
            100, 201, 102, 203, 104, 205, 106, 207
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 5D tensors along axis 2',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float16'}
        },
        'indices': {
          'data': [2, 2, 0, 0, 2, 2, 0, 0],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [11, 12, 13, 14, 15, 16, 17, 18],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 2}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [13, 14, 3, 4, 11, 12, 17, 18, 9, 10, 15, 16],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterElements float16 5D tensors along axis 2 and constant negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float16'}
        },
        'indices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [11, 12, 13, 14, 15, 16, 17, 18],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 2}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [13, 14, 3, 4, 11, 12, 17, 18, 9, 10, 15, 16],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float16'}
        }
      }
    }
  }
];

webnn_conformance_test(
    scatterElementsTests, buildAndExecuteGraph, getZeroULPTolerance);
