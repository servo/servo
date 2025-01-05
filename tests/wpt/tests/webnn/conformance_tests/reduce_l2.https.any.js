// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reducel2
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceL2(MLOperand input, optional MLReduceOptions options = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources) * 2 + 1,
  };
};

const reduceL2Tests = [
  // reduceL2 tests
  {
    'name': 'reduceL2 float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [4.860228061676025],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 4.860228061676025,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [4.860228061676025],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}, {'options': {'axes': []}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 4.860228061676025,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            -66.80043029785156,  -53.00004959106445,  -59.58587646484375,
            -46.14392852783203,  -49.60614013671875,  -12.832738876342773,
            -88.05061340332031,  -75.56246185302734,  -50.76777648925781,
            -36.96630096435547,  -26.344043731689453, -58.90546417236328,
            -94.28752899169922,  -22.7802791595459,   -84.3487777709961,
            -60.47734451293945,  -41.455806732177734, -92.84781646728516,
            -85.05448913574219,  -30.235260009765625, -47.33808135986328,
            -25.268428802490234, -78.11959075927734,  -28.330944061279297
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 292.57574462890625,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4, 29, 8,  56, 42, 78, 89, 64, 56, 81, 85, 18,
            6, 39, 35, 63, 87, 50, 81, 89, 5,  8,  37, 37
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 274.4029846191406,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            -70, -78, -65, -77, -25, -47, -63, -67, -66, -15, -28, -75,
            -88, -54, -13, -27, -5,  -18, -68, -71, -50, -56, -99, -99
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 300.3830871582031,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [
            122.352783203125, 124.8213119506836, 128.20062255859375,
            128.14801025390625, 87.18083953857422, 55.043975830078125
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [
            114.44775390625, 110.26422882080078, 133.47344970703125,
            64.96752166748047, 128.0914764404297, 101.677734375
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [272.0996398925781],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': 272.0996398925781,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [272.0996398925781],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceL2 float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [
            138.580078125, 166.67791748046875, 149.91552734375, 67.6578598022461
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceL2 float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.860228061676025,  88.23184204101562,  54.489688873291016,
            64.75027465820312,  6.855991363525391,  91.39871215820312,
            41.88857650756836,  73.65444946289062,  35.31573486328125,
            48.345428466796875, 82.39190673828125,  77.86200714111328,
            93.31141662597656,  62.48688507080078,  60.29290008544922,
            13.230599403381348, 20.535987854003906, 53.45161819458008,
            11.320085525512695, 64.75763702392578,  43.6589469909668,
            0.8374307155609131, 0.6848266124725342, 33.504703521728516
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [
            138.580078125, 166.67791748046875, 149.91552734375, 67.6578598022461
          ],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  },

  // float16 tests
  {
    'name': 'reduceL2 float16 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [4.859375],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [4.859375], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [4.859375],
          'descriptor': {shape: [], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}, {'options': {'axes': []}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [4.859375], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 1D constant tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 1D tensor all positive default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 1D tensor all negative default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            -66.8125, -53,        -59.59375, -46.15625,  -49.59375, -12.8359375,
            -88.0625, -75.5625,   -50.78125, -36.96875,  -26.34375, -58.90625,
            -94.3125, -22.78125,  -84.375,   -60.46875,  -41.46875, -92.875,
            -85.0625, -30.234375, -47.34375, -25.265625, -78.125,   -28.328125
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [292.5], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 1D tensor all positive integers default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4, 29, 8,  56, 42, 78, 89, 64, 56, 81, 85, 18,
            6, 39, 35, 63, 87, 50, 81, 89, 5,  8,  37, 37
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [274.5], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 1D tensor all negative integers default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            -70, -78, -65, -77, -25, -47, -63, -67, -66, -15, -28, -75,
            -88, -54, -13, -27, -5,  -18, -68, -71, -50, -56, -99, -99
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [300.5], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [{'input': 'reduceL2Input'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [122.375, 124.8125, 128.25, 128.125, 87.1875, 55.03125],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float16 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [114.4375, 110.3125, 133.5, 64.9375, 128, 101.6875],
          'descriptor': {shape: [2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float16 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [272],
          'descriptor': {shape: [1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'reduceL2 float16 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'}, {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output':
            {'data': [272], 'descriptor': {shape: [], dataType: 'float16'}}
      }
    }
  },
  {
    'name': 'reduceL2 float16 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments':
            [{'input': 'reduceL2Input'}, {'options': {'keepDimensions': true}}],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [272],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceL2 float16 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [138.625, 166.625, 149.875, 67.625],
          'descriptor': {shape: [2, 2], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name':
        'reduceL2 float16 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceL2Input': {
          'data': [
            4.859375,   88.25,  54.5,     64.75,         6.85546875,    91.375,
            41.875,     73.625, 35.3125,  48.34375,      82.375,        77.875,
            93.3125,    62.5,   60.28125, 13.234375,     20.53125,      53.4375,
            11.3203125, 64.75,  43.65625, 0.83740234375, 0.68505859375, 33.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'reduceL2',
        'arguments': [
          {'input': 'reduceL2Input'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceL2Output'
      }],
      'expectedOutputs': {
        'reduceL2Output': {
          'data': [138.625, 166.625, 149.875, 67.625],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float16'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceL2Tests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
