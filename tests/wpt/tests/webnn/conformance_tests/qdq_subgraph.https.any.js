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
            49.1112174987793,    11.907459259033203,   21.115795135498047,
            70.7490005493164,    94.51628112792969,   93.78905487060547,
            11.178888320922852,  32.80592346191406,   83.31897735595703,
            91.1207275390625,    0.11235756427049637, 15.397955894470215,
          ],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'},
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
            2,  17, 38, 41, 5, 3, 2, 17, 38, 41, 5, 3,
          ],
          'descriptor': {shape: [2, 3, 2], dataType: 'int8'},
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
            -0.4901961088180542, -0.43137258291244507, -0.3490196168422699,
            -0.33725491166114807, -0.4784314036369324, -0.4862745404243469,
            -0.4901961088180542, -0.43137258291244507, -0.3490196168422699,
            -0.33725491166114807, -0.4784314036369324, -0.4862745404243469,
          ],
          'descriptor': {shape: [2, 3, 2], dataType: 'float32'}
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
          'outputs': 'quantizedtransposeOutput'
        },
        {
          'name': 'dequantizeLinear',
          'arguments': [
            {'input': 'quantizedtransposeOutput'},
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
];

if (navigator.ml) {
  subgraphTests.forEach((test) => {
    webnn_conformance_test(buildAndExecuteGraph, getPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
