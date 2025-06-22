// META: title=test WebNN `dequantization -> operators -> quantization` subgraph
// META: global=window,worker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

const subgraphTests = [
  {
    'name': 'quantized conv2d',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.05605664849281311, 0.7114229798316956, 0.6529743671417236],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'filter': {
          'data': [2, 3, 4],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'int8'},
          'constant': true
        },
        'filterScale': {
          'data': [0.023458752938762234],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'filterZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'bias': {
          'data': [1],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'biasScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'biasZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'filter'},
            {'scale': 'filterScale', 'zeroPoint': 'filterZeroPoint'}
          ],
          'outputs': 'dequantizedFilter'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'bias'},
            {'scale': 'biasScale', 'zeroPoint': 'biasZeroPoint'}
          ],
          'outputs': 'dequantizedBias'
        },
        {
          'name': 'conv2d',
          'arguments': [
            {'input': 'dequantizedInput'}, {'filter': 'dequantizedFilter'}, {
              'options': {
                'inputLayout': 'nhwc',
                'bias': 'dequantizedBias',
                'filterLayout': 'ohwi'
              }
            }
          ],
          'outputs': 'conv2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'conv2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedConv2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedConv2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [0.11372549831867218],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear -> conv2d -> clamp -> quantizeLinear',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.05605664849281311, 0.7114229798316956, 0.6529743671417236],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'filter': {
          'data': [2, 3, 4],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'int8'},
          'constant': true
        },
        'filterScale': {
          'data': [0.023458752938762234],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'filterZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'bias': {
          'data': [1],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'biasScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'biasZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'filter'},
            {'scale': 'filterScale', 'zeroPoint': 'filterZeroPoint'}
          ],
          'outputs': 'dequantizedFilter'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'bias'},
            {'scale': 'biasScale', 'zeroPoint': 'biasZeroPoint'}
          ],
          'outputs': 'dequantizedBias'
        },
        {
          'name': 'conv2d',
          'arguments': [
            {'input': 'dequantizedInput'}, {'filter': 'dequantizedFilter'}, {
              'options': {
                'inputLayout': 'nhwc',
                'bias': 'dequantizedBias',
                'filterLayout': 'ohwi'
              }
            }
          ],
          'outputs': 'conv2dOutput'
        },
        {
          'name': 'clamp',
          'arguments': [
            {'input': 'conv2dOutput'},
            {'options': {'minValue': 0, 'maxValue': 6}}
          ],
          'outputs': 'clampOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'clampOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedClampOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedClampOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [0.11372549831867218],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'dequantizeLinear -> conv2d -> relu -> quantizeLinear',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.05605664849281311, 0.7114229798316956, 0.6529743671417236],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'filter': {
          'data': [2, 3, 4],
          'descriptor': {shape: [1, 1, 1, 3], dataType: 'int8'},
          'constant': true
        },
        'filterScale': {
          'data': [0.7114229798316956],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'filterZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'bias': {
          'data': [1],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'biasScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'biasZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'filter'},
            {'scale': 'filterScale', 'zeroPoint': 'filterZeroPoint'}
          ],
          'outputs': 'dequantizedFilter'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'bias'},
            {'scale': 'biasScale', 'zeroPoint': 'biasZeroPoint'}
          ],
          'outputs': 'dequantizedBias'
        },
        {
          'name': 'conv2d',
          'arguments': [
            {'input': 'dequantizedInput'}, {'filter': 'dequantizedFilter'}, {
              'options': {
                'inputLayout': 'nhwc',
                'bias': 'dequantizedBias',
                'filterLayout': 'ohwi'
              }
            }
          ],
          'outputs': 'conv2dOutput'
        },
        {
          'name': 'relu',
          'arguments': [
            {'input': 'conv2dOutput'}
          ],
          'outputs': 'reluOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReluOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [0.49803924560546875],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized convTranspose2d',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.05605664849281311, 0.7114229798316956,
            0.6529743671417236, 0.7114229798316956,
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'filter': {
          'data': [
            2, 3, 4, 5,
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'int8'},
          'constant': true
        },
        'filterScale': {
          'data': [0.023458752938762234],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'filterZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'bias': {
          'data': [2],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'biasScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'biasZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'filter'},
            {'scale': 'filterScale', 'zeroPoint': 'filterZeroPoint'}
          ],
          'outputs': 'dequantizedFilter'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'bias'},
            {'scale': 'biasScale', 'zeroPoint': 'biasZeroPoint'}
          ],
          'outputs': 'dequantizedBias'
        },
        {
          'name': 'convTranspose2d',
          'arguments': [
            {'input': 'dequantizedInput'}, {'filter': 'dequantizedFilter'}, {
              'options': {
                'inputLayout': 'nhwc',
                'bias': 'dequantizedBias',
                'filterLayout': 'ohwi'
              }
            }
          ],
          'outputs': 'convTranspose2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'convTranspose2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedConvTranspose2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedConvTranspose2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.003921568859368563, 0.03921568766236305, 0.05098039656877518,
            0.03529411926865578,  0.15294118225574493, 0.13333334028720856,
            0.062745101749897,    0.14509804546833038, 0.08235294371843338,
          ],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise binary add',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            0.41167140007019043,  0.0479511022567749,  0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            2,  7,  8, 11, 5, 23,
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'add',
          'arguments': [
            {'inputA': 'dequantizedInputA'}, {'inputB': 'dequantizedInputB'}
          ],
          'outputs': 'addOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'addOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedAddOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedAddOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.9215686917304993, 0.5764706134796143, 0.8666667342185974,
            0.7450980544090271, 0.9333333969116211, 0.6705882549285889,
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized conv2d with padding',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.6124474406242371,  0.8857858777046204,  0.13667134940624237,
            0.5645291209220886,  0.8965172171592712,  0.36792829632759094,
            0.6811466217041016,  0.0479511022567749,  0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
            0.4272463321685791,  0.535800576210022,   0.5910806059837341,
            0.28415432572364807, 0.4147258698940277,  0.026906268671154976,
            0.3621256649494171,  0.9945681691169739,  0.07184549421072006,
            0.12204372137784958, 0.8422137498855591,  0.4537501037120819,
            0.21529443562030792
          ],
          'descriptor': {shape: [1, 5, 5, 1], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'filter': {
          'data': [2, 3, 4, 5, 6, 7, 8, 9, 3],
          'descriptor': {shape: [1, 3, 3, 1], dataType: 'int8'},
          'constant': true
        },
        'filterScale': {
          'data': [0.023458752938762234],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'filterZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'bias': {
          'data': [1],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'biasScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'biasZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'filter'},
            {'scale': 'filterScale', 'zeroPoint': 'filterZeroPoint'}
          ],
          'outputs': 'dequantizedFilter'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'bias'},
            {'scale': 'biasScale', 'zeroPoint': 'biasZeroPoint'}
          ],
          'outputs': 'dequantizedBias'
        },
        {
          'name': 'conv2d',
          'arguments': [
            {'input': 'dequantizedInput'}, {'filter': 'dequantizedFilter'}, {
              'options': {
                'inputLayout': 'nhwc',
                'bias': 'dequantizedBias',
                'filterLayout': 'ohwi',
                'padding': [2, 1, 2, 1]
              }
            }
          ],
          'outputs': 'conv2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'conv2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedConv2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedConv2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.04313725605607033, 0.19215688109397888, 0.30980393290519714,
            0.2352941334247589, 0.20784315466880798, 0.29411765933036804,
            0.125490203499794, 0.35686275362968445, 0.43529415130615234,
            0.3764706254005432, 0.33725491166114807, 0.2980392277240753,
            0.14509804546833038,  0.38431376218795776, 0.3764706254005432,
            0.38823533058166504, 0.45098042488098145, 0.38431376218795776,
            0.12156863510608673, 0.250980406999588, 0.34117648005485535,
            0.3333333432674408,  0.41960787773132324, 0.4549019932746887,
            0.09019608050584793,  0.16862745583057404, 0.25882354378700256,
            0.4274510145187378,  0.49803924560546875, 0.3803921937942505,
            0.03921568766236305, 0.09019608050584793, 0.20784315466880798,
            0.26274511218070984, 0.3176470696926117, 0.1725490242242813
          ],
          'descriptor': {shape: [1, 6, 6, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise binary sub',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            15.57776927947998, -62.7008056640625,
            82.55709075927734, -74.90638732910156,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.617084980010986],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [120],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            12, 9, 2, 43,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.617084980010986],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [120],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.617084980010986],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [120],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'sub',
          'arguments': [
            {'inputA': 'dequantizedInputA'}, {'inputB': 'dequantizedInputB'}
          ],
          'outputs': 'subOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'subOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSubOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSubOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            4.319594860076904, 4.319594860076904, 4.319594860076904,
            -27.1517391204834,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise binary mul',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            49.1112174987793, 11.907459259033203,
            21.115795135498047, 70.7490005493164,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.3921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            21, 24,
            8, 13
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.3921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.3921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'mul',
          'arguments': [
            {'inputA': 'dequantizedInputA'}, {'inputB': 'dequantizedInputB'}
          ],
          'outputs': 'mulOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'mulOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedMulOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedMulOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            43.529415130615234, 36.86274719238281,
            -56.4705924987793, -51.372554779052734,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise binary max',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, -4.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            12, 24, 35, 11,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'max',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'maxOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'maxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedMaxOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedMaxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            48.03300857543945, 52.150123596191406,
            55.92414474487305, 47.68991470336914,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise binary min',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            3.549168109893799, 4.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            12, 24, 35, 11,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'min',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'minOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'minOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedMinOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedMinOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            3.430929183959961, 4.803300857543945,
            8.577322959899902, 6.17567253112793,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical equal',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'equal',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'equalOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'equalOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1,
            0, 0,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical notEqual',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'notEqual',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'notEqualOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'notEqualOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 0,
            1, 1,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical greater',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'greater',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'greaterOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'greaterOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 0,
            1, 0,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical greaterOrEqual',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'greaterOrEqual',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'greaterOrEqualOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'greaterOrEqualOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1,
            1, 0,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical lesser',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'lesser',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'lesserOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'lesserOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0, 0,
            0, 1,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized element-wise logical lesserOrEqual',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -2.549168109893799, 0.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'},
          'constant': false
        },
        'inputB': {
          'data': [
            -7, 2,
            2, 30,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int8'},
          'constant': true
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'lesserOrEqual',
          'arguments': [{'inputA': 'dequantizedInputA'},  {'inputB': 'dequantizedInputB'}],
          'outputs': 'lesserOrEqualOutput'
        },
        {
          'name': 'cast',
          'arguments': [{'input': 'lesserOrEqualOutput'}, {'type': 'int32'}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1,
            0, 1,
          ],
          'descriptor': {shape: [2, 2], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized gather',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            2.549168109893799, 4.794857501983643, 7.413617134094238,
            8.413617134094238, 6.108623504638672, 3.549168109893799,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'gatherIndices': {
          'data': [1],
          'descriptor': {shape: [], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'gather',
          'arguments': [{'input': 'dequantizedInput'}, {'indices': 'gatherIndices'}],
          'outputs': 'gatherOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'gatherOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedGatherOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedGatherOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            8.577322959899902, 6.17567253112793, 3.430929183959961,
          ],
          'descriptor': {shape: [3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized gemm with bias',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            49.1112174987793, 11.907459259033203, 11.115795135498047,
            21.115795135498047, 70.7490005493164, 31.115795135498047
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            21, 24, 8, 15, 6, 7
          ],
          'descriptor': {shape: [3, 2], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.023458752938762234],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputC': {
          'data': [
            8, 15
          ],
          'descriptor': {shape: [2], dataType: 'int32'},
          'constant': true
        },
        'inputCScale': {
          'data': [0.000091995115004270],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputCZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int32'},
          'constant': true
        },
        'outputScale': {
          'data': [0.3921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputC'},
            {'scale': 'inputCScale', 'zeroPoint': 'inputCZeroPoint'}
          ],
          'outputs': 'dequantizedInputC'
        },
        {
          'name': 'gemm',
          'arguments': [
            {'a': 'dequantizedInputA'}, {'b': 'dequantizedInputB'},
            {'options': {'c': 'dequantizedInputC'}}
          ],
          'outputs': 'gemmOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'gemmOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedGemmOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedGemmOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.7843137979507446, 1.1764707565307617,
            0.7843137979507446, 1.1764707565307617,
          ],
          'descriptor': {shape: [2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized transpose',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
          ],
          'descriptor': {shape: [1, 1, 2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'transpose',
          'arguments': [{'input': 'dequantizedInput'}],
          'outputs': 'transposeOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'transposeOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedTransposeOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedTransposeOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.6823529601097107, 0.20000001788139343,
            0.0470588281750679, 0.4117647409439087,
            0.3333333432674408, 0.0784313753247261,
          ],
          'descriptor': {shape: [3, 2, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized tanh',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'tanh',
          'arguments': [{'input': 'dequantizedInput'}],
          'outputs': 'tanhOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'tanhOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedTanhOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedTanhOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.5921568870544434, 0.0470588281750679, 0.32156863808631897,
            0.19607844948768616, 0.38823533058166504, 0.0784313753247261,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized sigmoid',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.00390625],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.00390625],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'sigmoid',
          'arguments': [{'input': 'dequantizedInput'}],
          'outputs': 'sigmoidOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'sigmoidOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSigmoidOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSigmoidOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.6640625, 0.51171875, 0.58203125,
            0.55078125, 0.6015625, 0.51953125,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized leaky relu',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'leakyRelu',
          'arguments': [
            {'input': 'dequantizedInput'},
            {'options': {'alpha': 5.799162942273234}}
          ],
          'outputs': 'leakyReluOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'leakyReluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedLeakyReluOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedLeakyReluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.49803924560546875, 0.0470588281750679, 0.3333333432674408,
            -0.501960813999176, -0.02352941408753395, -0.364705890417099,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized concat',
    'graph': {
      'inputs': {
        'inputA': {
          'data': [
            -0.990639865398407, -0.576785683631897, -0.32276400923728943,
            -0.44735023379325867, -0.11028251051902771, -0.5945112705230713,
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'},
          'constant': false
        },
        'inputAScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputAZeroPoint': {
          'data': [127],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'inputB': {
          'data': [
            2, 27, 38,
          ],
          'descriptor': {shape: [3, 1], dataType: 'int8'},
          'constant': true
        },
        'inputBScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputBZeroPoint': {
          'data': [127],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [127],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'inputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'quantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInputA'},
            {'scale': 'inputAScale', 'zeroPoint': 'inputAZeroPoint'}
          ],
          'outputs': 'dequantizedInputA'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'inputB'},
            {'scale': 'inputBScale', 'zeroPoint': 'inputBZeroPoint'}
          ],
          'outputs': 'dequantizedInputB'
        },
        {
          'name': 'concat',
          'arguments': [
            {
              'inputs': ['dequantizedInputA', 'dequantizedInputB']
            },
            {'axis': 1}
          ],
          'outputs': 'concatOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'concatOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedConcatOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedConcatOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            -0.9921569228172302, -0.5764706134796143, -0.4901961088180542,
            -0.32156863808631897, -0.44705885648727417, -0.3921568989753723,
            -0.1098039299249649, -0.5960784554481506, -0.3490196168422699,
          ],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized elu',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'elu',
          'arguments': [{'input': 'dequantizedInput'}],
          'outputs': 'eluOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'eluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedEluOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedEluOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.49803924560546875, 0.0470588281750679, 0.3333333432674408,
            -0.18039216101169586, -0.003921568859368563, -0.062745101749897,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized averagePool2d',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            -2.549168109893799, -4.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'averagePool2d',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'layout': 'nhwc'}}],
          'outputs': 'averagePool2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'averagePool2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedAveragePool2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedAveragePool2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            3.774022102355957,
          ],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized maxPool2d',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            -2.549168109893799, -4.794857501983643,
            8.413617134094238, 6.108623504638672
          ],
          'descriptor': {shape: [1, 2, 2, 1], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'maxPool2d',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'layout': 'nhwc'}}],
          'outputs': 'maxPool2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'maxPool2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedMaxPool2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedMaxPool2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            8.577322959899902,
          ],
          'descriptor': {shape: [1, 1, 1, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized reshape',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'reshape',
          'arguments': [{'input': 'dequantizedInput'}, {'newShape': [3, 2]}],
          'outputs': 'reshapeOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reshapeOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReshapeOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReshapeOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.43529415130615234, 0.0470588281750679,
            0.3333333432674408, -0.20000001788139343,
            -0.003921568859368563, -0.062745101749897,
          ],
          'descriptor': {shape: [3, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized split',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'split',
          'arguments': [{'input': 'dequantizedInput'}, {'splits': 3}, {'options': {'axis': 1}}],
          'outputs': ['splitOutput 1', 'splitOutput 2', 'splitOutput 3'],
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'splitOutput 1'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSplitOutput 1'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSplitOutput 1'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output 1'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'splitOutput 2'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSplitOutput 2'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSplitOutput 2'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output 2'
        },
                {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'splitOutput 3'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSplitOutput 3'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSplitOutput 3'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output 3'
        }
      ],
      'expectedOutputs': {
        'output 1': {
          'data': [
            0.43529415130615234, -0.20000001788139343,
          ],
          'descriptor': {shape: [2, 1], dataType: 'float32'}
        },
        'output 2': {
          'data': [
            0.0470588281750679, -0.003921568859368563,
          ],
          'descriptor': {shape: [2, 1], dataType: 'float32'}
        },
        'output 3': {
          'data': [
            0.3333333432674408, -0.062745101749897,
          ],
          'descriptor': {shape: [2, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized slice',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [16],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'slice',
          'arguments': [{'input': 'dequantizedInput'}, {'starts': [0, 1]}, {'sizes': [1, 2]}],
          'outputs': 'sliceOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'sliceOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSliceOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSliceOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.0470588281750679, 0.3333333432674408,
          ],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized argMax',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            2.549168109893799, 4.794857501983643, 7.413617134094238,
            8.413617134094238, 6.108623504638672, 3.549168109893799,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'argMax',
          'arguments': [{'input': 'dequantizedInput'},  {'axis': 0}],
          'outputs': 'output'
        },
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            1, 1, 0,
          ],
          'descriptor': {shape: [3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'quantized softmax',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            0.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            0.19882695376873016, 0.41167140007019043, 0.07934240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.00390625],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.00390625],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [-128],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'softmax',
          'arguments': [{'input': 'dequantizedInput'}, {'axis': 0}],
          'outputs': 'softmaxOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'softmaxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedSoftmaxOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedSoftmaxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.6171875, 0.41015625, 0.5625,
            0.3828125, 0.58984375, 0.4375,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized reduceMax',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'reduceMax',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'axes': [1]}}],
          'outputs': 'reduceMaxOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reduceMaxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReduceMaxOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReduceMaxOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.49803924560546875, -0.003921568859368563,
          ],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized reduceMin',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'reduceMin',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'axes': [1]}}],
          'outputs': 'reduceMinOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reduceMinOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReduceMinOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReduceMinOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.0470588281750679, -0.20000001788139343,
          ],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized reduceMean',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'reduceMean',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'axes': [1]}}],
          'outputs': 'reduceMeanOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reduceMeanOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReduceMeanOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReduceMeanOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.29411765933036804, -0.09019608050584793,
          ],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized reduceSum',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'reduceSum',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'axes': [1]}}],
          'outputs': 'reduceSumOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'reduceSumOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedReduceSumOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedReduceSumOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.49803924560546875, -0.2666666805744171,
          ],
          'descriptor': {shape: [2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized resample2d',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [1, 2, 3, 1], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'resample2d',
          'arguments': [{'input': 'dequantizedInput'}, {'options': {'sizes': [4, 6], 'axes': [1, 2]}}],
          'outputs': 'resample2dOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'resample2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedResample2dOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedResample2dOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            0.49803924560546875, 0.49803924560546875, 0.0470588281750679,
            0.0470588281750679, 0.3333333432674408, 0.3333333432674408,
            0.49803924560546875, 0.49803924560546875, 0.0470588281750679,
            0.0470588281750679, 0.3333333432674408, 0.3333333432674408,
            -0.20000001788139343, -0.20000001788139343, -0.003921568859368563,
            -0.003921568859368563, -0.062745101749897, -0.062745101749897,
            -0.20000001788139343, -0.20000001788139343, -0.003921568859368563,
            -0.003921568859368563, -0.062745101749897, -0.062745101749897,
          ],
          'descriptor': {shape: [1, 4, 6, 1], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized pad with reflection mode',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            1.6811466217041016, 0.0479511022567749, 0.33355462551116943,
            -0.1988269537687301, -0.0041167140007019, -0.0634240251779556,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'inputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'inputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
        'outputScale': {
          'data': [0.003921568859368563],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'outputZeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'inputScale', 'zeroPoint': 'inputZeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'pad',
          'arguments': [
            {'input': 'dequantizedInput'}, {'beginningPadding': [1, 2]},
            {'endingPadding': [1, 2]}, {'options': {'mode': 'reflection'}}
          ],
          'outputs': 'padOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'padOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'quantizedPadOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedPadOutput'},
            {'scale': 'outputScale', 'zeroPoint': 'outputZeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            -0.062745101749897, -0.003921568859368563, -0.20000001788139343,
            -0.003921568859368563, -0.062745101749897, -0.003921568859368563,
            -0.20000001788139343, 0.3333333432674408, 0.0470588281750679,
            0.49803924560546875, 0.0470588281750679, 0.3333333432674408,
            0.0470588281750679, 0.49803924560546875, -0.062745101749897,
            -0.003921568859368563, -0.20000001788139343, -0.003921568859368563,
            -0.062745101749897, -0.003921568859368563, -0.20000001788139343,
            0.3333333432674408, 0.0470588281750679, 0.49803924560546875,
            0.0470588281750679, 0.3333333432674408, 0.0470588281750679,
            0.49803924560546875,
          ],
          'descriptor': {shape: [4, 7], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized clamp',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            8.413617134094238, 6.108623504638672, 3.549168109893799,
            1.6811466217041016, -0.1988269537687301, -8.413617134094238,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'clamp',
          'arguments': [
            {'input': 'dequantizedInput'},
            {'options': {'minValue': 0, 'maxValue': 6}}
          ],
          'outputs': 'clampOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'clampOutput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedClampOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedClampOutput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            5.832579612731934, 5.832579612731934, 3.430929183959961,
            1.7154645919799805, 0, 0,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'quantized clamp with emulation',
    'graph': {
      'inputs': {
        'input': {
          'data': [
            8.413617134094238, 6.108623504638672, 3.549168109893799,
            1.6811466217041016, -0.1988269537687301, -8.413617134094238,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'},
          'constant': false
        },
        'scale': {
          'data': [0.343092918395996],
          'descriptor': {shape: [1], dataType: 'float32'},
          'constant': true
        },
        'zeroPoint': {
          'data': [0],
          'descriptor': {shape: [1], dataType: 'int8'},
          'constant': true
        },
      },
      'operators': [
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'input'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedInput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedInput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'dequantizedInput'
        },
        {
          'name': 'clamp',
          'arguments': [
            {'input': 'dequantizedInput'},
            {'options': {'minValue': -8, 'maxValue': 8}}
          ],
          'outputs': 'clampOutput'
        },
        {
          'name': 'quantizeLinear',
          'arguments': [
            {'input': 'clampOutput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'quantizedClampOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedClampOutput'},
            {'scale': 'scale', 'zeroPoint': 'zeroPoint'}
          ],
          'outputs': 'output'
        }
      ],
      'expectedOutputs': {
        'output': {
          'data': [
            7.89113712310791, 6.17567253112793, 3.430929183959961,
            1.7154645919799805, -0.3430929183959961, -7.89113712310791,
          ],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      }
    }
  },
];

if (navigator.ml) {
  subgraphTests.forEach((test) => {
    webnn_conformance_test(buildAndExecuteGraph, getPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
