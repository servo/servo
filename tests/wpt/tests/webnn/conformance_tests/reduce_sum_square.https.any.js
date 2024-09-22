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
  }
];

if (navigator.ml) {
  reduceSumSquareTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
