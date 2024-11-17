// META: title=test WebNN API gatherND operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-gatherND
// Gather values of the input tensor along a range of dimensions according to
// the indices.
//
// MLOperand gatherND(
//     MLOperand input, MLOperand indices,
//     optional MLOperatorOptions options = {});


const getGatherNDPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const gatherNDTests = [
  {
    'name': 'gatherND float32 3D input and 2D indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 4], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [1, 0, 0, 1, 1, 1],
          'descriptor': {shape: [3, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [
            51.79948425292969, 41.94132614135742, -1.1303654909133911,
            -50.42131042480469, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 90.2870101928711,
            55.620765686035156, 44.92119598388672, 56.828636169433594
          ],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 4D input and 1D int32 indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [1, 0, 0],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [51.79948425292969, 41.94132614135742],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 4D input and 1D uint32 indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [1, 0, 0],
          'descriptor': {shape: [3], dataType: 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [51.79948425292969, 41.94132614135742],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 4D input and 1D int64 indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [1, 0, 0],
          'descriptor': {shape: [3], dataType: 'int64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [51.79948425292969, 41.94132614135742],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 4D input and 1D minimum indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [-2, -2, -2],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [-66.05901336669922, -68.9197006225586],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 4D input and 1D maximum indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [2, 2, 2, 2], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [1, 1, 1],
          'descriptor': {shape: [3], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [44.92119598388672, 56.828636169433594],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 2D input and 2D negative indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [4, 4], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [-1, -2, -3, -4],
          'descriptor': {shape: [2, 2], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [44.92119598388672, 89.0337142944336],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    // TODO(crbug.com/366412395): Need to finalize the behavior of out-of-bounds
    // indices. The result of this case is calculated by assuming the indices is
    // being clamped.
    'name': 'gatherND float32 1D input and 2D out-of-bounds indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [16], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [16, 20],
          'descriptor': {shape: [2, 1], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [56.828636169433594, 56.828636169433594],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'gatherND float32 2D input and 2D out-of-bounds indices',
    'graph': {
      'inputs': {
        'gatherNDInput': {
          'data': [
            -66.05901336669922, -68.9197006225586, -77.02045440673828,
            -26.158037185668945, 89.0337142944336, -45.89653396606445,
            43.84803771972656, 48.81806945800781, 51.79948425292969,
            41.94132614135742, -1.1303654909133911, -50.42131042480469,
            90.2870101928711, 55.620765686035156, 44.92119598388672,
            56.828636169433594
          ],
          'descriptor': {shape: [16, 1], dataType: 'float32'}
        },
        'gatherNDIndices': {
          'data': [16, 20],
          'descriptor': {shape: [2, 1], dataType: 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'gatherND',
        'arguments':
            [{'input': 'gatherNDInput'}, {'indices': 'gatherNDIndices'}],
        'outputs': 'gatherNDOutput'
      }],
      'expectedOutputs': {
        'gatherNDOutput': {
          'data': [56.828636169433594, 56.828636169433594],
          'descriptor': {shape: [2, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  gatherNDTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getGatherNDPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
