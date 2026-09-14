// META: title=test WebNN API scatterND operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

const scatterNDTests = [
  {
    'name':
        'scatterND 1D float32 tensors (Insert individual elements in a tensor by index)',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float32'}
        },
        'indices': {
          'data': [4, 3, 1, 7],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 3D float32 tensors (Insert entire slices of a higher rank tensor)',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4,
            5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float32'}
        },
        'indices':
            {'data': [0, 2], 'descriptor': {shape: [2, 1], dataType: 'int32'}},
        'updates': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4
          ],
          'descriptor': {shape: [2, 4, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            4, 4, 4, 4, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 3D float32 tensors by negative indices (Insert entire slices of a higher rank tensor)',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4,
            5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float32'}
        },
        'indices':
            {'data': [-4, -2], 'descriptor': {shape: [2, 1], dataType: 'int32'}},
        'updates': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4
          ],
          'descriptor': {shape: [2, 4, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            4, 4, 4, 4, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 1D float32 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float32'}
        },
        'indices': {
          'data': [-4, -5, -7, -1],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      }
    }
  },
    {
    'name':
        'scatterND 1D float32 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float32'}
        },
        'indices': {
          'data': [4, 3, 1, 7],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 2D float32 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        },
        'indices': {
          'data': [-4, -4, -3, -3, -2, -2, -1, -1],
          'descriptor': {shape: [4, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 200, 300, 400],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data':
              [100, 2, 3, 4, 5, 200, 7, 8, 9, 10, 300, 12, 13, 14, 15, 400],
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 2D float32 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        },
        'indices': {
          'data': [0, 0, 1, 1, 2, 2, 3, 3],
          'descriptor': {shape: [4, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 200, 300, 400],
          'descriptor': {shape: [4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data':
              [100, 2, 3, 4, 5, 200, 7, 8, 9, 10, 300, 12, 13, 14, 15, 400],
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 4D float32 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [-2, -2, -1, -1],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [50, 60, 70, 80, 130, 140, 150, 160],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            50, 60, 70, 80, 5, 6, 7, 8, 9, 10, 11, 12, 130, 140, 150, 160
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 4D float32 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [0, 0, 1, 1],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [50, 60, 70, 80, 130, 140, 150, 160],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            50, 60, 70, 80, 5, 6, 7, 8, 9, 10, 11, 12, 130, 140, 150, 160
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 5D float32 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [-2, -2, -2, -1, -1, -1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 101, 102, 103, 110, 111, 112, 113],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            100, 101, 102, 103, 5,   6,   7,   8,   9,   10,  11,
            12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,
            23,  24,  25,  26,  27,  28,  110, 111, 112, 113
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 5D float32 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float32'}
        },
        'indices': {
          'data': [0, 0, 0, 1, 1, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 101, 102, 103, 110, 111, 112, 113],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            100, 101, 102, 103, 5,   6,   7,   8,   9,   10,  11,
            12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,
            23,  24,  25,  26,  27,  28,  110, 111, 112, 113
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name':
        'scatterND 1D float16 tensors (Insert individual elements in a tensor by index)',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float16'}
        },
        'indices': {
          'data': [4, 3, 1, 7],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 3D float16 tensors (Insert entire slices of a higher rank tensor)',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4,
            5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float16'}
        },
        'indices':
            {'data': [0, 2], 'descriptor': {shape: [2, 1], dataType: 'int32'}},
        'updates': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4
          ],
          'descriptor': {shape: [2, 4, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            4, 4, 4, 4, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 3D float16 tensors by negative indices (Insert entire slices of a higher rank tensor)',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1, 2, 3, 4, 5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4,
            5, 6, 7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float16'}
        },
        'indices':
            {'data': [-4, -2], 'descriptor': {shape: [2, 1], dataType: 'int32'}},
        'updates': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4
          ],
          'descriptor': {shape: [2, 4, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 1, 2, 3, 4, 5, 6,
            7, 8, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            4, 4, 4, 4, 8, 7, 6, 5, 4, 3, 2, 1, 1, 2, 3, 4, 5, 6, 7, 8
          ],
          'descriptor': {shape: [4, 4, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 1D float16 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float16'}
        },
        'indices': {
          'data': [-4, -5, -7, -1],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 1D float16 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8],
          'descriptor': {shape: [8], dataType: 'float16'}
        },
        'indices': {
          'data': [4, 3, 1, 7],
          'descriptor': {shape: [4, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [9, 10, 11, 12],
          'descriptor': {shape: [4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1, 11, 3, 10, 9, 6, 7, 12],
          'descriptor': {shape: [8], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 2D float16 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [4, 4], dataType: 'float16'}
        },
        'indices': {
          'data': [-4, -4, -3, -3, -2, -2, -1, -1],
          'descriptor': {shape: [4, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 200, 300, 400],
          'descriptor': {shape: [4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data':
              [100, 2, 3, 4, 5, 200, 7, 8, 9, 10, 300, 12, 13, 14, 15, 400],
          'descriptor': {shape: [4, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 2D float16 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [4, 4], dataType: 'float16'}
        },
        'indices': {
          'data': [0, 0, 1, 1, 2, 2, 3, 3],
          'descriptor': {shape: [4, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 200, 300, 400],
          'descriptor': {shape: [4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data':
              [100, 2, 3, 4, 5, 200, 7, 8, 9, 10, 300, 12, 13, 14, 15, 400],
          'descriptor': {shape: [4, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 4D float16 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [-2, -2, -1, -1],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [50, 60, 70, 80, 130, 140, 150, 160],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            50, 60, 70, 80, 5, 6, 7, 8, 9, 10, 11, 12, 130, 140, 150, 160
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 4D float16 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data':
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [0, 0, 1, 1],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        },
        'updates': {
          'data': [50, 60, 70, 80, 130, 140, 150, 160],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            50, 60, 70, 80, 5, 6, 7, 8, 9, 10, 11, 12, 130, 140, 150, 160
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 5D float16 tensors by negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [-2, -2, -2, -1, -1, -1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 101, 102, 103, 110, 111, 112, 113],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            100, 101, 102, 103, 5,   6,   7,   8,   9,   10,  11,
            12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,
            23,  24,  25,  26,  27,  28,  110, 111, 112, 113
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'scatterND 5D float16 tensors by non-negative indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float16'}
        },
        'indices': {
          'data': [0, 0, 0, 1, 1, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        },
        'updates': {
          'data': [100, 101, 102, 103, 110, 111, 112, 113],
          'descriptor': {shape: [2, 2, 2], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [
            100, 101, 102, 103, 5,   6,   7,   8,   9,   10,  11,
            12,  13,  14,  15,  16,  17,  18,  19,  20,  21,  22,
            23,  24,  25,  26,  27,  28,  110, 111, 112, 113
          ],
          'descriptor': {shape: [2, 2, 2, 2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'scatterND 2D int8 tensors with index out of bound',
    'graph': {
      'inputs': {
        'input': {
          'data': [0, 0],
          'descriptor': {shape: [2, 1], dataType: 'int8'}
        },
        'indices': {
          'data': [2147483647 /* INT32_MAX */],
          'descriptor': {shape: [1, 1], dataType: 'int32'}
        },
        'updates': {
          'data': [1],
          'descriptor': {shape: [1, 1], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'scatterND',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [0, 1],
          'descriptor': {shape: [2, 1], dataType: 'int8'}
        }
      }
    }
  }
];

webnn_conformance_test(
    scatterNDTests, buildAndExecuteGraph, getZeroULPTolerance);
