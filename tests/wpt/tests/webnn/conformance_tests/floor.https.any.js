// META: title=test WebNN API element-wise floor operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-unary
// Compute the floor of the input tensor, element-wise.
//
// MLOperand floor(MLOperand input);


const getFloorPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 0, float16: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const floorTests = [
  {
    'name': 'floor float32 0D scalar',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [89.69458770751953],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput':
            {'data': [89], 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'floor float32 1D constant tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'floor float32 1D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'floor float32 2D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'floor float32 3D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'floor float32 4D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'floor float32 5D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.69458770751953,   -79.67150115966797,  -66.80949401855469,
            -71.88439178466797,  86.33935546875,      6.823808670043945,
            24.908447265625,     0.9734055399894714,  19.948184967041016,
            0.8437776565551758,  -24.752939224243164, 77.76482391357422,
            -33.644466400146484, 80.7762451171875,    44.47844314575195,
            -37.65005874633789,  -83.78780364990234,  65.840087890625,
            -39.83677673339844,  32.5257568359375,    -21.213542938232422,
            -80.30911254882812,  16.674850463867188,  -72.88893127441406
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'floor float16 0D scalar',
    'graph': {
      'inputs': {
        'floorInput':
            {'data': [89.6875], 'descriptor': {shape: [], dataType: 'float16'}}
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput':
            {'data': [89], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'floor float16 1D constant tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'floor float16 1D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'floor float16 2D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'floor float16 3D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'floor float16 4D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'floor float16 5D tensor',
    'graph': {
      'inputs': {
        'floorInput': {
          'data': [
            89.6875,    -79.6875, -66.8125,     -71.875,   86.3125,
            6.82421875, 24.90625, 0.9736328125, 19.953125, 0.84375,
            -24.75,     77.75,    -33.65625,    80.75,     44.46875,
            -37.65625,  -83.8125, 65.8125,      -39.84375, 32.53125,
            -21.21875,  -80.3125, 16.671875,    -72.875
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'floor',
        'arguments': [{'input': 'floorInput'}],
        'outputs': 'floorOutput'
      }],
      'expectedOutputs': {
        'floorOutput': {
          'data': [
            89,  -80, -67, -72, 86,  6,  24,  0,  19,  0,   -25, 77,
            -34, 80,  44,  -38, -84, 65, -40, 32, -22, -81, 16,  -73
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      }
    }
  }
];

webnn_conformance_test(
    floorTests, buildAndExecuteGraph, getFloorPrecisionTolerance);
