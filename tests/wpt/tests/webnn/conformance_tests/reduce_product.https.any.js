// META: title=test WebNN API reduction operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-reduceproduct
// Reduce the input tensor along all dimensions, or along the axes specified in
// the axes array parameter.
//
// dictionary MLReduceOptions {
//   sequence<[EnforceRange] unsigned long> axes;
//   boolean keepDimensions = false;
// };
//
// MLOperand reduceProduct(MLOperand input, optional MLReduceOptions options
// = {});

const getReductionOperatorsPrecisionTolerance = (graphResources) => {
  return {
    metricType: 'ULP',
    value: getReducedElementCount(graphResources),
  };
};

const reduceProductTests = [
  {
    'name': 'reduceProduct float32 0D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [-68.75911712646484],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': -68.75911712646484,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 0D constant tensor empty axes',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [-68.75911712646484],
          'descriptor': {shape: [], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments':
            [{'input': 'reduceProductInput'}, {'options': {'axes': []}}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': -68.75911712646484,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 1D constant tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 1D tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 2D tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 3D tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 4D tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 5D tensor default options',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [{'input': 'reduceProductInput'}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 3D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments':
            [{'input': 'reduceProductInput'}, {'options': {'axes': [2]}}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [
            7519895, -1292816.375, 2441721.75, -110637.7734375, -7380313.5,
            -818030.5
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 4D tensor options.axes',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments':
            [{'input': 'reduceProductInput'}, {'options': {'axes': [0, 2]}}],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [
            4227263.5, -446960.5625, 3811296.75, 1280298.5, -1343475.375,
            1280118.75
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 3D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 3D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [1.5855958784642327e+37],
          'descriptor': {shape: [1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 4D tensor options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': 1.5855958784642327e+37,
          'descriptor': {shape: [], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reduceProduct float32 4D tensor options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'}, {'options': {'keepDimensions': true}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [1.5855958784642327e+37],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceProduct float32 4D tensor options.axes with options.keepDimensions=false',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': false}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [-3638925568, 6523364352, -414643360, 1610916352],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'reduceProduct float32 4D tensor options.axes with options.keepDimensions=true',
    'graph': {
      'inputs': {
        'reduceProductInput': {
          'data': [
            -68.75911712646484, 99.44961547851562,   24.86055564880371,
            -44.23515319824219, -22.699743270874023, 79.97555541992188,
            14.4650239944458,   49.23109436035156,   30.058706283569336,
            69.45106506347656,  -20.15709686279297,  -58.0255126953125,
            51.896610260009766, -2.020799160003662,  39.392974853515625,
            26.78073501586914,  -97.97651672363281,  48.66154479980469,
            -85.19523620605469, -18.16986083984375,  64.83759307861328,
            -14.95883846282959, -74.50932312011719,  -11.319679260253906
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reduceProduct',
        'arguments': [
          {'input': 'reduceProductInput'},
          {'options': {'axes': [1, 3], 'keepDimensions': true}}
        ],
        'outputs': 'reduceProductOutput'
      }],
      'expectedOutputs': {
        'reduceProductOutput': {
          'data': [-3638925568, 6523364352, -414643360, 1610916352],
          'descriptor': {shape: [2, 1, 2, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reduceProductTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getReductionOperatorsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
