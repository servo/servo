// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducel1
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceL1(MLOperand input, optional MLReduceOptions options = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources),
  };
};

const reduceL1Tests = [
  {
    'name': 'reduceL1 float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [5.50882625579834],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 5.50882625579834,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [5.50882625579834],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}, {'options': {'axes': []}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 5.50882625579834,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            -98.83928680419922,  -57.66743850708008,  -57.101200103759766,
            -6.693042278289795,  -45.30584716796875,  -86.68338775634766,
            -74.71875,           -76.46739959716797,  -75.37677001953125,
            -18.22093963623047,  -54.64426803588867,  -36.45240020751953,
            -18.322681427001953, -47.94379425048828,  -40.19978332519531,
            -15.830483436584473, -48.883358001708984, -41.600242614746094,
            -20.6556339263916,   -92.2993392944336,   -46.28858184814453,
            -80.57186126708984,  -25.49472999572754,  -48.96730041503906
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1215.228515625,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            18, 29, 35, 36, 4,  76, 41, 18, 53, 29, 25, 94,
            26, 1,  3,  68, 39, 25, 87, 30, 39, 75, 76, 66
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': 993, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceL1 float32 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            -92, -52, -88, -78, -20, -73, -42, -57, -39, -75, -17, -36,
            -81, -24, -88, -91, -76, -5,  -44, -66, -96, -8,  -69, -27
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': 1344, 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'reduceL1 float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [
            142.01541137695312, 106.62430572509766, 175.39280700683594,
            286.7269592285156, 169.36322021484375, 212.59750366210938
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [
            258.57110595703125, 174.42807006835938, 102.19830322265625,
            134.52191162109375, 207.92910766601562, 215.07168579101562
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [1092.72021484375],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': 1092.72021484375,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [1092.72021484375],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceL1 float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [
            108.43173217773438, 315.6007995605469, 359.5506591796875,
            309.13702392578125
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceL1 float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.50882625579834,   50.61575698852539,  1.6773051023483276,
            84.2135238647461,   15.664374351501465, 52.89714813232422,
            9.125157356262207,  28.937623977661133, 12.567061424255371,
            11.39999008178711,  86.91246032714844,  64.51329803466797,
            71.2834243774414,   76.34410858154297,  41.53409194946289,
            97.5653305053711,   31.803831100463867, 6.089754581451416,
            61.70843505859375,  69.76119232177734,  38.919403076171875,
            52.288333892822266, 22.31783676147461,  99.0719223022461
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [
            108.43173217773438, 315.6007995605469, 359.5506591796875,
            309.13702392578125
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'reduceL1 float16 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [5.5078125],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [5.5078125],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float16 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [5.5078125],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}, {'options': {'axes': []}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [5.5078125],
          'descriptor': {shape: [], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float16 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            -98.8125,   -57.65625, -57.09375,  -6.69140625, -45.3125,
            -86.6875,   -74.75,    -76.4375,   -75.375,     -18.21875,
            -54.65625,  -36.4375,  -18.328125, -47.9375,    -40.1875,
            -15.828125, -48.875,   -41.59375,  -20.65625,   -92.3125,
            -46.28125,  -80.5625,  -25.5,      -48.96875
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1215], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            18, 29, 35, 36, 4,  76, 41, 18, 53, 29, 25, 94,
            26, 1,  3,  68, 39, 25, 87, 30, 39, 75, 76, 66
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [993], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            -92, -52, -88, -78, -20, -73, -42, -57, -39, -75, -17, -36,
            -81, -24, -88, -91, -76, -5,  -44, -66, -96, -8,  -69, -27
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1344], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [{'input': 'reduceL1Input'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [142, 106.625, 175.375, 286.75, 169.375, 212.5],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float16 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [258.5, 174.5, 102.1875, 134.5, 208, 215],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float16 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [1093],
          'descriptor': {shape: [1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL1 float16 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output':
            {'data': [1093], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL1 float16 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments':
            [{'input': 'reduceL1Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [1093],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceL1 float16 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [108.4375, 315.5, 359.5, 309],
          'descriptor': {shape: [2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceL1 float16 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL1Input': {
          'data': [
            5.5078125, 50.625,  1.677734375, 84.1875,    15.6640625, 52.90625,
            9.125,     28.9375, 12.5703125,  11.3984375, 86.9375,    64.5,
            71.3125,   76.375,  41.53125,    97.5625,    31.796875,  6.08984375,
            61.71875,  69.75,   38.90625,    52.28125,   22.3125,    99.0625
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL1',
        'arguments': [
          {'input': 'reduceL1Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceL1Output'
      }],
      'expectedOutputs': {
        'reduceL1Output': {
          'data': [108.4375, 315.5, 359.5, 309],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float16'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceL1Tests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
