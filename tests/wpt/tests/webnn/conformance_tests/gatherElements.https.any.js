// META: title=test WebNN API gatherElements operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-gatherElements
// Gather values of the input tensor along an axis according to the indices.
//
// dictionary MLGatherOptions {
//   [EnforceRange] unsigned long axis = 0;
// };
//
// MLOperand gatherElements(
//     MLOperand input, MLOperand indices,
//     optional MLGatherOptions options = {});

const gatherElementsTests = [
  {
    'name': 'gatherElements float32 2D input and uint32 indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {shape: [3, 2], dataType: 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 2D input and int32 indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {shape: [3, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 2D input and int32 non-constant indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {shape: [3, 2], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 2D input and int32 indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -26.158037185668945, -68.9197006225586, 51.79948425292969,
            43.84803771972656, 89.0337142944336, -77.02045440673828
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 3D input and int32 indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 0, 1],
          'descriptor': {shape: [1, 2, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            89.0337142944336, -68.9197006225586, -77.02045440673828,
            48.81806945800781
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 3D input and int32 negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, 0, 0, -1],
          'descriptor': {shape: [1, 2, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            89.0337142944336, -68.9197006225586, -77.02045440673828,
            48.81806945800781
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 3D input and int32 non-constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, 0, 0, -1],
          'descriptor': {shape: [1, 2, 2], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            89.0337142944336, -68.9197006225586, -77.02045440673828,
            48.81806945800781
          ],
          'descriptor': {shape: [1, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 2D input and int32 constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {shape: [3, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 2D input and int32 non-constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {shape: [3, 2], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -68.9197006225586, -66.05901336669922, -45.89653396606445,
            -45.89653396606445, 48.81806945800781, 43.84803771972656
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 2D input and int32 constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -26.158037185668945, -68.9197006225586, 51.79948425292969,
            43.84803771972656, 89.0337142944336, -77.02045440673828
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 2D input and int32 non-constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {shape: [2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -26.158037185668945, -68.9197006225586, 51.79948425292969,
            43.84803771972656, 89.0337142944336, -77.02045440673828
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 1D input and int32 constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [6], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -6, -3],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.79948425292969, -26.158037185668945, 43.84803771972656],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 1D input and int32 non-constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [6], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -6, -3],
          'descriptor': {shape: [3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.79948425292969, -26.158037185668945, 43.84803771972656],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 3D input and int32 constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -2, -2, -1],
          'descriptor': {shape: [2, 1, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -77.02045440673828, -68.9197006225586, 89.0337142944336,
            48.81806945800781
          ],
          'descriptor': {shape: [2, 1, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 3D input and int32 non-constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781
          ],
          'descriptor': {shape: [2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -2, -2, -1],
          'descriptor': {shape: [2, 1, 2], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [
            -77.02045440673828, -68.9197006225586, 89.0337142944336,
            48.81806945800781
          ],
          'descriptor': {shape: [2, 1, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 4D input and int32 constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data':
              [-1, -2, -1, -2, -1, -2, -1, -2, -2, -1, -2, -1, -2, -1, -2, -1],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 5, 14, 7, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 4D input and int32 non-constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data':
              [-1, -2, -1, -2, -1, -2, -1, -2, -2, -1, -2, -1, -2, -1, -2, -1],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 5, 14, 7, 16],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 5D input and int32 constant negative indices options.axis=2',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 2}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [5, 6, 1, 2, 11, 12, 7, 8],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float32 5D input and int32 non-constant negative indices options.axis=2',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {shape: [1, 2, 3, 2, 1], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 2}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [5, 6, 1, 2, 11, 12, 7, 8],
          'descriptor': {shape: [1, 2, 2, 2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float32 1D input and int32 out-of-bounds indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969
          ],
          'descriptor': {shape: [6], dataType: 'float32'}
        },
        'gatherElementsIndices': {
          'data': [7],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.79948425292969],
          'descriptor': {shape: [1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'gatherElements float16 2D input and uint32 indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {'shape': [3, 2], 'dataType': 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-68.9375, -66.0625, -45.90625, -45.90625, 48.8125, 43.84375],
          'descriptor': {'shape': [3, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float16 2D input and int32 indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {'shape': [3, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-68.9375, -66.0625, -45.90625, -45.90625, 48.8125, 43.84375],
          'descriptor': {'shape': [3, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float16 2D input and int32 indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 2, 2, 1, 0],
          'descriptor': {'shape': [2, 3], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-26.15625, -68.9375, 51.8125, 43.84375, 89.0625, -77],
          'descriptor': {'shape': [2, 3], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float16 3D input and int32 indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 0, 1],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [89.0625, -68.9375, -77, 48.8125],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 3D input and int32 non-constant indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [1, 0, 0, 1],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [89.0625, -68.9375, -77, 48.8125],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name': 'gatherElements float16 3D input and int32 negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, 0, 0, -1],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [89.0625, -68.9375, -77, 48.8125],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 3D input and int32 non-constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, 0, 0, -1],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [89.0625, -68.9375, -77, 48.8125],
          'descriptor': {'shape': [1, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 2D input and int32 constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {'shape': [3, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-68.9375, -66.0625, -45.90625, -45.90625, 48.8125, 43.84375],
          'descriptor': {'shape': [3, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 2D input and int32 non-constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {'shape': [3, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-68.9375, -66.0625, -45.90625, -45.90625, 48.8125, 43.84375],
          'descriptor': {'shape': [3, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 2D input and int32 constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {'shape': [2, 3], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-26.15625, -68.9375, 51.8125, 43.84375, 89.0625, -77],
          'descriptor': {'shape': [2, 3], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 2D input and int32 non-constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125, 51.8125
          ],
          'descriptor': {'shape': [3, 3], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-2, -3, -1, -1, -2, -3],
          'descriptor': {'shape': [2, 3], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-26.15625, -68.9375, 51.8125, 43.84375, 89.0625, -77],
          'descriptor': {'shape': [2, 3], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 1D input and int32 constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [-26.15625, 89.0625, -45.90625, 43.84375, 48.8125, 51.8125],
          'descriptor': {'shape': [6], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -6, -3],
          'descriptor': {'shape': [3], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.8125, -26.15625, 43.84375],
          'descriptor': {'shape': [3], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 1D input and int32 non-constant negative indices',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [-26.15625, 89.0625, -45.90625, 43.84375, 48.8125, 51.8125],
          'descriptor': {'shape': [6], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -6, -3],
          'descriptor': {'shape': [3], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'}, {'indices': 'gatherElementsIndices'}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [51.8125, -26.15625, 43.84375],
          'descriptor': {'shape': [3], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 3D input and int32 constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -2, -2, -1],
          'descriptor': {'shape': [2, 1, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-77, -68.9375, 89.0625, 48.8125],
          'descriptor': {'shape': [2, 1, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 3D input and int32 non-constant negative indices options.axis=1',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [
            -66.0625, -68.9375, -77, -26.15625, 89.0625, -45.90625, 43.84375,
            48.8125
          ],
          'descriptor': {'shape': [2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -2, -2, -1],
          'descriptor': {'shape': [2, 1, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 1}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [-77, -68.9375, 89.0625, 48.8125],
          'descriptor': {'shape': [2, 1, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 4D input and int32 constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data':
              [-1, -2, -1, -2, -1, -2, -1, -2, -2, -1, -2, -1, -2, -1, -2, -1],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 5, 14, 7, 16],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 4D input and int32 non-constant negative indices options.axis=0',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data':
              [-1, -2, -1, -2, -1, -2, -1, -2, -2, -1, -2, -1, -2, -1, -2, -1],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 0}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 5, 14, 7, 16],
          'descriptor': {'shape': [2, 2, 2, 2], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 5D input and int32 constant negative indices options.axis=2',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {'shape': [1, 2, 3, 2, 1], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {'shape': [1, 2, 2, 2, 1], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 2}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [5, 6, 1, 2, 11, 12, 7, 8],
          'descriptor': {'shape': [1, 2, 2, 2, 1], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name':
        'gatherElements float16 5D input and int32 non-constant negative indices options.axis=2',
    'graph': {
      'inputs': {
        'gatherElementsInput': {
          'data': [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          'descriptor': {'shape': [1, 2, 3, 2, 1], 'dataType': 'float16'}
        },
        'gatherElementsIndices': {
          'data': [-1, -1, -3, -3, -1, -1, -3, -3],
          'descriptor': {'shape': [1, 2, 2, 2, 1], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'gatherElements',
        'arguments': [
          {'input': 'gatherElementsInput'},
          {'indices': 'gatherElementsIndices'}, {'options': {'axis': 2}}
        ],
        'outputs': 'gatherElementsOutput'
      }],
      'expectedOutputs': {
        'gatherElementsOutput': {
          'data': [5, 6, 1, 2, 11, 12, 7, 8],
          'descriptor': {'shape': [1, 2, 2, 2, 1], 'dataType': 'float16'}
        }
      }
    }
  }
];

webnn_conformance_test(
    gatherElementsTests, buildAndExecuteGraph, getZeroULPTolerance);
