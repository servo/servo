// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducesumsquare
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceSumSquare(MLOperand input, optional MLReduceOptions options
// = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources) * 2,
  };
};

const reduceSumSquareTests = [
  {
    'name': 'reduceSumSquare float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [52.5615348815918],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 2762.71484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [52.5615348815918],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 2762.71484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float32 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            -21.45201301574707,    -57.30725860595703,  -72.8390121459961,
            -0.059761520475149155, -71.73678588867188,  -44.61909103393555,
            -43.12002182006836,    -91.3373794555664,   -33.17243957519531,
            -48.555931091308594,   -95.6286392211914,   -20.876630783081055,
            -16.690837860107422,   -39.52110290527344,  -7.5107855796813965,
            -90.59027099609375,    -42.21683120727539,  -76.74274444580078,
            -98.22420501708984,    -60.272953033447266, -74.73202514648438,
            -8.543684005737305,    -59.888736724853516, -17.99894142150879
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 80052.015625,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float32 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52, 48, 2,  66, 30, 39, 14, 23, 81, 94, 78, 64,
            38, 16, 63, 11, 46, 95, 17, 47, 40, 53, 87, 43
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': 71347, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name':
        'reduceSumSquare float32 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            -10, -60, -69, -88, -35, -84, -74, -42, -93, -26, -40, -55,
            -92, -26, -39, -30, -61, -16, -16, -36, -9,  -89, -45, -29
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': 73634, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 1D tensor with empty axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput':
            {'data': [2, 3], 'descriptor': {shape: [2], dataType: 'float32'}}
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [4, 9], 'descriptor': {shape: [2], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            9709.013671875, 10817.7685546875, 14548.470703125, 16385.8515625,
            9287.357421875, 12527.3974609375
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            8585.87109375, 7700.654296875, 19889.1796875, 7113.0439453125,
            16775.708984375, 13211.3994140625
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [73275.859375],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': 73275.859375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [73275.859375],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            12302.474609375, 22772.77734375, 26919.09765625, 11281.5068359375
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            52.5615348815918,   2.6261062622070312, 82.04877471923828,
            14.401411056518555, 33.96051788330078,  83.9383773803711,
            47.445045471191406, 19.177289962768555, 13.493006706237793,
            44.152381896972656, 86.53118133544922,  70.20919799804688,
            25.67262840270996,  79.73770141601562,  66.42284393310547,
            70.40363311767578,  13.503327369689941, 41.225399017333984,
            6.654552936553955,  85.79743957519531,  89.91349029541016,
            53.55647277832031,  39.48537063598633,  3.9460408687591553
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            12302.474609375, 22772.77734375, 26919.09765625, 11281.5068359375
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'reduceSumSquare float16 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [52.5625],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [2762], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [52.5625],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [2762], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name':
        'reduceSumSquare float16 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            -1.646484375,    -1.2998046875,    -0.57763671875,
            -0.5869140625,   -1.740234375,     -0.2020263671875,
            -1.28125,        -1.92578125,      -0.63671875,
            -0.5068359375,   -1.9462890625,    -1.5078125,
            -1.212890625,    -0.6669921875,    -1.1337890625,
            -0.450439453125, -0.7978515625,    -0.2196044921875,
            -0.221923828125, -0.1463623046875, -0.75537109375,
            -1.0830078125,   -1.3740234375,    -0.059600830078125
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [28.015625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float16 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            2, 4, 2, 6, 3, 9, 1, 2, 1, 4, 7, 6,
            3, 1, 3, 1, 6, 5, 1, 4, 4, 3, 8, 3
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [453], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name':
        'reduceSumSquare float16 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            -10, -6, -9, -8, -3, -4, -4, -2, -3, -2, -4, -5,
            -2,  -2, -3, -3, -1, -6, -1, -3, -9, -8, -5, -2
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [627], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 1D tensor with empty axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput':
            {'data': [2, 3], 'descriptor': {shape: [2], dataType: 'float16'}}
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput':
            {'data': [4, 9], 'descriptor': {shape: [2], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [{'input': 'reduceSumSquareInput'}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            4.94140625, 4.30859375, 1.4765625, 3.458984375, 11.359375,
            4.97265625
          ],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments':
            [{'input': 'reduceSumSquareInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [
            3.72265625, 5.4453125, 5.75, 5.00390625, 8.6796875, 1.9130859375
          ],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceSumSquare float16 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [30.515625],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float16 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [8.828125, 1.8974609375, 9.625, 10.1640625],
          'descriptor': {shape: [2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceSumSquare float16 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumSquareInput': {
          'data': [
            1.3935546875,  1.20703125,      1.18359375,   0.3759765625,
            0.69677734375, 0.75244140625,   1.068359375,  1.455078125,
            0.87890625,    0.2149658203125, 0.7998046875, 0.135986328125,
            1.099609375,   0.77685546875,   1.1025390625, 0.65625,
            1.703125,      1.6025390625,    1.5185546875, 1.892578125,
            0.8408203125,  1.2294921875,    1.529296875,  0.64404296875
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceSumSquare',
        'arguments': [
          {'input': 'reduceSumSquareInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceSumSquareOutput'
      }],
      'expectedOutputs': {
        'reduceSumSquareOutput': {
          'data': [8.828125, 1.8974609375, 9.625, 10.1640625],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float16'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceSumSquareTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
