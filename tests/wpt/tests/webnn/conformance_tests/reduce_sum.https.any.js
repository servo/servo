// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducesum
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceSum(MLOperand input, optional MLReduceOptions options = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources),
  };
};

const reduceSumTests = [
  {
    'name': 'reduceSum float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [69.6038589477539],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 69.6038589477539,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [69.6038589477539],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 69.6038589477539,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            -51.77016830444336,  -34.46467971801758,  -40.98350524902344,
            -83.34922790527344,  -67.67525482177734,  -18.7031192779541,
            -20.28106117248535,  -20.12305450439453,  -83.63451385498047,
            -23.651084899902344, -10.208438873291016, -36.2129020690918,
            -76.26201629638672,  -9.094745635986328,  -53.889339447021484,
            -67.52340698242188,  -71.14580535888672,  -82.04484558105469,
            -96.29924774169922,  -68.46700286865234,  -26.107192993164062,
            -68.0182113647461,   -4.8330769538879395, -48.900699615478516
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': -1163.642578125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            56, 90, 67, 33, 20, 58, 22, 15, 86, 79, 59, 99,
            16, 95, 67, 11, 60, 89, 50, 57, 77, 89, 10, 2
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput':
            {'data': 1307, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceSum float32 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            -55, -36, -74, -17, -67, -95, -3,  -67, -95, -13, -45, -9,
            -33, -98, -86, -11, -70, -44, -31, -68, -79, -24, -60, -36
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput':
            {'data': -1216, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceSum float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [{'input': 'reduceSumInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [
            210.44241333007812, 104.54179382324219, 226.2177734375,
            239.025390625, 260.405029296875, 273.2470397949219
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments':
            [{'input': 'reduceSumInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [
            179.63900756835938, 260.37457275390625, 219.3611297607422,
            246.83712768554688, 157.4895782470703, 250.1780242919922
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [1313.87939453125],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': 1313.87939453125,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceSum float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [1313.87939453125],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSum float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [
            355.21942138671875, 185.98255920410156, 362.3598937988281,
            410.3175354003906
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceSum float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceSumInput': {
          'data': [
            69.6038589477539,  99.17485809326172,  32.78234100341797,
            8.881362915039062, 16.094295501708984, 11.80689525604248,
            32.64223861694336, 43.99836349487305,  77.01776885986328,
            79.79425811767578, 45.00794982910156,  24.397796630859375,
            57.502685546875,   57.60173034667969,  80.26985931396484,
            43.65110778808594, 87.5036849975586,   94.50203704833984,
            35.54289627075195, 42.856414794921875, 88.58631896972656,
            98.85772705078125, 25.626853942871094, 60.1761360168457
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceSum',
        'arguments': [
          {'input': 'reduceSumInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceSumOutput'
      }],
      'expectedOutputs': {
        'reduceSumOutput': {
          'data': [
            355.21942138671875, 185.98255920410156, 362.3598937988281,
            410.3175354003906
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceSumTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
