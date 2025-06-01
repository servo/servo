// META: title=test WebNN API relu operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-relu-method
// Compute the rectified linear function of the input tensor.
//
// MLOperand relu(MLOperand input);

const reluTests = [
  {
    'name': 'relu float32 1D constant tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 0D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [79.04724884033203],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
          ],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 1D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 2D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 3D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 4D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'relu float32 5D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.04724884033203,  2.2503609657287598,  80.73938751220703,
            63.9039192199707,   77.67340850830078,   -71.0915756225586,
            -82.74703216552734, -26.81442642211914,  -99.16788482666016,
            -35.71083450317383, 18.361658096313477,  -37.36091613769531,
            -52.8386116027832,  -10.408374786376953, 60.6029167175293,
            -13.64419937133789, -76.5425033569336,   -8.132338523864746,
            51.51447296142578,  -51.63370132446289,  -64.56800079345703,
            -5.093302249908447, 15.354103088378906,  90.03858947753906
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.04724884033203,
            2.2503609657287598,
            80.73938751220703,
            63.9039192199707,
            77.67340850830078,
            0,
            0,
            0,
            0,
            0,
            18.361658096313477,
            0,
            0,
            0,
            60.6029167175293,
            0,
            0,
            0,
            51.51447296142578,
            0,
            0,
            0,
            15.354103088378906,
            90.03858947753906
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'relu float16 1D constant tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'relu float16 0D tensor',
    'graph': {
      'inputs': {
        'reluInput':
            {'data': [79.0625], 'descriptor': {shape: [], dataType: 'float16'}}
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput':
            {'data': [79.0625], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'relu float16 1D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'relu float16 2D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'relu float16 3D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'relu float16 4D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'relu float16 5D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            79.0625,   2.25,      80.75,    63.90625,   77.6875,    -71.0625,
            -82.75,    -26.8125,  -99.1875, -35.71875,  18.359375,  -37.375,
            -52.84375, -10.40625, 60.59375, -13.640625, -76.5625,   -8.1328125,
            51.5,      -51.625,   -64.5625, -5.09375,   15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [
            79.0625, 2.25,      80.75, 63.90625, 77.6875,    0,        0, 0, 0,
            0,       18.359375, 0,     0,        0,          60.59375, 0, 0, 0,
            51.5,    0,         0,     0,        15.3515625, 90.0625
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      }
    }
  },

  // int8 tests
  {
    'name': 'relu int8 4D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            // int8 range: [/* -(2**7) */ -128, /* 2**7 - 1 */ 127]
            -128, 0, 126, 127
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [0, 0, 126, 127],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'int8'}
        }
      }
    }
  },

  // int32 tests
  {
    'name': 'relu int32 4D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            // int32 range: [/* -(2**31) */ -2147483648, /* 2**31 - 1 */ 2147483647]
            -2147483648, 0, 2147483646, 2147483647
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [0, 0, 2147483646, 2147483647],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'int32'}
        }
      }
    }
  },

  // int64 tests
  {
    'name': 'relu int64 4D tensor',
    'graph': {
      'inputs': {
        'reluInput': {
          'data': [
            // int64 range: [/* -(2**63) */ –9223372036854775808,
            //               /* 2**63 - 1 */ 92233720368547758087]
            BigInt(-(2**63)) + 1n, -100n, 0n, 100n, BigInt(2**63) - 1n
          ],
          'descriptor': {shape: [1, 1, 1, 5], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'relu',
        'arguments': [{'input': 'reluInput'}],
        'outputs': 'reluOutput'
      }],
      'expectedOutputs': {
        'reluOutput': {
          'data': [0n, 0n, 0n, 100n, BigInt(2**63) - 1n],
          'descriptor': {shape: [1, 1, 1, 5], dataType: 'int64'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reluTests.forEach((test) => {
    webnn_conformance_test(buildAndExecuteGraph, getPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
