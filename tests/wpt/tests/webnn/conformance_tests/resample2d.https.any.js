// META: title=test WebNN API resample2d operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-resample2d-method
// Resample the tensor values from the source to the destination spatial
// dimensions according to the scaling factors.
//
// enum MLInterpolationMode {
//   "nearest-neighbor",
//   "linear"
// };
//
// dictionary MLResample2dOptions {
//   MLInterpolationMode mode = "nearest-neighbor";
//   sequence<float> scales;
//   sequence<[EnforceRange] unsigned long> sizes;
//   sequence<[EnforceRange] unsigned long> axes;
// };
//
// MLOperand resample2d(
//     MLOperand input, optional MLResample2dOptions options = {});


const getResample2dPrecisionTolerance = (graphResources) => {
  const args = graphResources.operators[0].arguments;
  const options =
      args.length === 2 ? {...args[1][Object.keys(args[1])[0]]} : {};
  const expectedOutputs = graphResources.expectedOutputs;
  const dataType =
      expectedOutputs[Object.keys(expectedOutputs)[0]].descriptor.dataType;
  let tolerance;

  if (options.mode && options.mode === 'linear') {
    // interpolation mode is linear
    if (dataType === 'float32') {
      tolerance = 84;
    } else if (dataType === 'float16') {
      tolerance = 10;
    } else {
      tolerance = 1;
    }
  } else {
    // interpolation mode is nearest-neighbor
    tolerance = 0;
  }

  return {metricType: 'ULP', value: tolerance};
};

const resample2dTests = [
  {
    'name': 'resample2d float32 4D tensor default options',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            3.8600528240203857, 45.18463134765625,  87.67153930664062,
            98.7821044921875,   66.3741455078125,   3.411583423614502,
            86.14930725097656,  95.98133850097656,  76.87126159667969,
            16.52591323852539,  65.98783111572266,  25.470922470092773,
            22.56010627746582,  92.08479309082031,  85.80876922607422,
            92.63166046142578,  29.916208267211914, 75.40460968017578,
            62.06375503540039,  1.7712159156799316, 99.4723129272461,
            11.440549850463867, 25.396343231201172, 67.0217514038086
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [{'input': 'resample2dInput'}],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            3.8600528240203857, 45.18463134765625,  87.67153930664062,
            98.7821044921875,   66.3741455078125,   3.411583423614502,
            86.14930725097656,  95.98133850097656,  76.87126159667969,
            16.52591323852539,  65.98783111572266,  25.470922470092773,
            22.56010627746582,  92.08479309082031,  85.80876922607422,
            92.63166046142578,  29.916208267211914, 75.40460968017578,
            62.06375503540039,  1.7712159156799316, 99.4723129272461,
            11.440549850463867, 25.396343231201172, 67.0217514038086
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'resample2d(upsample) float32 4D tensor options.scales',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments':
            [{'input': 'resample2dInput'}, {'options': {'scales': [2, 2]}}],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'resample2d(upsample) float32 4D tensor options.sizes',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments':
            [{'input': 'resample2dInput'}, {'options': {'sizes': [4, 6]}}],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor options.sizes ignored options.scales',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'scales': [0.5, 0.5], 'sizes': [4, 6]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'resample2d(upsample) float32 4D tensor options.axes=[1, 2]',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 2, 3, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'sizes': [4, 6], 'axes': [1, 2]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 4, 6, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor explicit options.axes=[2, 3]',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'sizes': [4, 6], 'axes': [2, 3]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor explicit options.axes=[3, 2]',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'sizes': [6, 4], 'axes': [3, 2]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor explicit options.mode=\'nearest-neighbor\'',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'mode': 'nearest-neighbor', 'sizes': [4, 6]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            59.92947006225586, 59.92947006225586, 41.98918914794922,
            41.98918914794922, 66.39534759521484, 66.39534759521484,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281,
            90.7006607055664,  90.7006607055664,  86.95105743408203,
            86.95105743408203, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor options.scales options.mode=\'linear\'',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'mode': 'linear', 'scales': [2, 2]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586,  55.444400787353516, 46.47425842285156,
            48.090728759765625, 60.29380798339844,  66.39534759521484,
            67.62226867675781,  64.02411651611328,  56.82780838012695,
            57.31512451171875,  65.48605346679688,  69.57152557373047,
            83.00786590576172,  81.18354797363281,  77.534912109375,
            75.76390838623047,  75.87055206298828,  75.92387390136719,
            90.7006607055664,   89.76325988769531,  87.88845825195312,
            84.9883041381836,   81.06280517578125,  79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor options.sizes options.mode=\'linear\'',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'mode': 'linear', 'sizes': [4, 6]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586,  55.444400787353516, 46.47425842285156,
            48.090728759765625, 60.29380798339844,  66.39534759521484,
            67.62226867675781,  64.02411651611328,  56.82780838012695,
            57.31512451171875,  65.48605346679688,  69.57152557373047,
            83.00786590576172,  81.18354797363281,  77.534912109375,
            75.76390838623047,  75.87055206298828,  75.92387390136719,
            90.7006607055664,   89.76325988769531,  87.88845825195312,
            84.9883041381836,   81.06280517578125,  79.10005187988281
          ],
          'descriptor': {shape: [1, 1, 4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name':
        'resample2d(upsample) float32 4D tensor options.axes=[1, 2] options.mode=\'linear\'',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 41.98918914794922, 66.39534759521484,
            90.7006607055664, 86.95105743408203, 79.10005187988281
          ],
          'descriptor': {shape: [1, 2, 3, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'mode': 'linear', 'sizes': [4, 6], 'axes': [1, 2]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586,  55.444400787353516, 46.47425842285156,
            48.090728759765625, 60.29380798339844,  66.39534759521484,
            67.62226867675781,  64.02411651611328,  56.82780838012695,
            57.31512451171875,  65.48605346679688,  69.57152557373047,
            83.00786590576172,  81.18354797363281,  77.534912109375,
            75.76390838623047,  75.87055206298828,  75.92387390136719,
            90.7006607055664,   89.76325988769531,  87.88845825195312,
            84.9883041381836,   81.06280517578125,  79.10005187988281
          ],
          'descriptor': {shape: [1, 4, 6, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'resample2d(upsample) float32 4D tensor options.axes=[0, 1]',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 90.7006607055664, 41.98918914794922,
            86.95105743408203, 66.39534759521484, 79.10005187988281
          ],
          'descriptor': {shape: [3, 2, 1, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'sizes': [6, 4], 'axes': [0, 1]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 90.7006607055664,
            90.7006607055664,  59.92947006225586, 59.92947006225586,
            90.7006607055664,  90.7006607055664,  41.98918914794922,
            41.98918914794922, 86.95105743408203, 86.95105743408203,
            41.98918914794922, 41.98918914794922, 86.95105743408203,
            86.95105743408203, 66.39534759521484, 66.39534759521484,
            79.10005187988281, 79.10005187988281, 66.39534759521484,
            66.39534759521484, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [6, 4, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'resample2d(upsample) float32 4D tensor options.axes=[1, 0]',
    'graph': {
      'inputs': {
        'resample2dInput': {
          'data': [
            59.92947006225586, 90.7006607055664, 41.98918914794922,
            86.95105743408203, 66.39534759521484, 79.10005187988281
          ],
          'descriptor': {shape: [3, 2, 1, 1], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'resample2d',
        'arguments': [
          {'input': 'resample2dInput'},
          {'options': {'sizes': [4, 6], 'axes': [1, 0]}}
        ],
        'outputs': 'resample2dOutput'
      }],
      'expectedOutputs': {
        'resample2dOutput': {
          'data': [
            59.92947006225586, 59.92947006225586, 90.7006607055664,
            90.7006607055664,  59.92947006225586, 59.92947006225586,
            90.7006607055664,  90.7006607055664,  41.98918914794922,
            41.98918914794922, 86.95105743408203, 86.95105743408203,
            41.98918914794922, 41.98918914794922, 86.95105743408203,
            86.95105743408203, 66.39534759521484, 66.39534759521484,
            79.10005187988281, 79.10005187988281, 66.39534759521484,
            66.39534759521484, 79.10005187988281, 79.10005187988281
          ],
          'descriptor': {shape: [6, 4, 1, 1], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  resample2dTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getResample2dPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
