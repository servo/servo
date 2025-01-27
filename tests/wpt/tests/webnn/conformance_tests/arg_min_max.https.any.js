// META: title=test WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-argminmax
// Return the index location of the minimum or maximum values of all the input
// values along the axis.
//
// dictionary MLArgMinMaxOptions : MLOperatorOptions {
//   boolean keepDimensions = false;
//   MLOperandDataType outputDataType = "int32";
// };
//
// MLOperand argMin(MLOperand input, [EnforceRange] unsigned long axis,
//                  optional MLArgMinMaxOptions options = {});
// MLOperand argMax(MLOperand input, [EnforceRange] unsigned long axis,
//                  optional MLArgMinMaxOptions options = {});

const getArgMinMaxPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const argMinMaxTests = [
  // argMin tests
  {
    'name': 'argMin float32 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float32 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float32 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 2, 0, 1, 1],
          'descriptor': {shape: [6], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 3D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 4D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 5D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 4D tensor, axis=2',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 2}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [2, 2, 0, 0, 0, 0],
          'descriptor': {shape: [2, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 4D tensor, axis=0, options.keepDimensions=true',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 4D tensor, axis=0, options.keepDimensions=false',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMin float32 4D tensor, axis=0, explicit options.outputDataType=\'int32\'',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'outputDataType': 'int32'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMin float32 4D tensor, axis=0, options.outputDataType=\'int64\'',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'outputDataType': 'int64'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1n, 1n, 1n, 0n, 0n, 0n, 0n, 0n, 1n, 0n, 0n, 0n],
          'descriptor': {shape: [1, 4, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'argMin float32 4D tensor, axis=0, all options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.8301241397857666, -24.986488342285156, 5.29998254776001,
            -48.54866027832031, 40.308868408203125,  60.184295654296875,
            -82.78385925292969, -96.50904083251953,  71.87028503417969,
            38.86639404296875,  -39.14372634887695,  31.444366455078125,
            -82.78385925292969, -96.50904083251953,  -25.533889770507812,
            -16.14226531982422, 66.63677215576172,   82.51197814941406,
            -82.78385925292969, -96.50904083251953,  39.76872634887695,
            42.1504020690918,   82.66864013671875,   85.45269012451172
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': true, 'outputDataType': 'int64'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1n, 1n, 1n, 0n, 0n, 0n, 0n, 0n, 1n, 0n, 0n, 0n],
          'descriptor': {shape: [1, 1, 4, 3], dataType: 'int64'}
        }
      }
    }
  },

  // float16 argMin tests
  {
    'name': 'argMin float16 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float16 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float16 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 2, 0, 1, 1],
          'descriptor': {shape: [6], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 3D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 4D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 5D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 0}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 4D tensor, axis=2',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [{'input': 'argMinInput'}, {'axis': 2}],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [2, 2, 0, 0, 0, 0],
          'descriptor': {shape: [2, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 4D tensor, axis=0, options.keepDimensions=true',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 4D tensor, axis=0, options.keepDimensions=false',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMin float16 4D tensor, axis=0, explicit options.outputDataType=\'int32\'',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'outputDataType': 'int32'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMin float16 4D tensor, axis=0, options.outputDataType=\'int64\'',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'outputDataType': 'int64'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1n, 1n, 1n, 0n, 0n, 0n, 0n, 0n, 1n, 0n, 0n, 0n],
          'descriptor': {shape: [1, 4, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'argMin float16 4D tensor, axis=0, all options',
    'graph': {
      'inputs': {
        'argMinInput': {
          'data': [
            3.830078125, -24.984375, 5.30078125, -48.5625, 40.3125,
            60.1875,     -82.8125,   -96.5,      71.875,   38.875,
            -39.15625,   31.4375,    -82.8125,   -96.5,    -25.53125,
            -16.140625,  66.625,     82.5,       -82.8125, -96.5,
            39.78125,    42.15625,   82.6875,    85.4375
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMin',
        'arguments': [
          {'input': 'argMinInput'}, {'axis': 0},
          {'options': {'keepDimensions': true, 'outputDataType': 'int64'}}
        ],
        'outputs': 'argMinOutput'
      }],
      'expectedOutputs': {
        'argMinOutput': {
          'data': [1n, 1n, 1n, 0n, 0n, 0n, 0n, 0n, 1n, 0n, 0n, 0n],
          'descriptor': {shape: [1, 1, 4, 3], dataType: 'int64'}
        }
      }
    }
  },

  // argMax tests
  {
    'name': 'argMax float32 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [24], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float32 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float32 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [1, 1, 1, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 3D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 4D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 5D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 4, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 4D tensor, axis=2',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 2}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [1, 2, 2, 1, 2, 2],
          'descriptor': {shape: [2, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 4D tensor, axis=3, options.keepDimensions=true',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4, 1], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 4D tensor, axis=3, options.keepDimensions=false',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMax float32 4D tensor, axis=3, explicit options.outputDataType=\'int32\'',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'outputDataType': 'int32'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMax float32 4D tensor, axis=3, options.outputDataType=\'int64\'',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'outputDataType': 'int64'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2n, 0n, 2n, 1n, 2n, 0n, 2n, 1n],
          'descriptor': {shape: [2, 1, 4], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'argMax float32 4D tensor, axis=3, all options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875,
            -51.09362030029297, -6.53970193862915,  73.8133773803711,
            88.46114349365234,  -5.294266700744629, -79.20668029785156,
            -41.70176696777344, 73.8133773803711,   88.46114349365234,
            -84.94000244140625, -61.48894119262695, -98.3387451171875
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': true, 'outputDataType': 'int64'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2n, 0n, 2n, 1n, 2n, 0n, 2n, 1n],
          'descriptor': {shape: [2, 1, 4, 1], dataType: 'int64'}
        }
      }
    }
  },

  // float16 argMax tests
  {
    'name': 'argMax float16 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [24], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float16 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [24], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float16 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [1, 1, 1, 0, 0, 0],
          'descriptor': {shape: [6], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 3D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 4D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 5D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 0}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
          'descriptor': {shape: [1, 4, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 4D tensor, axis=2',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [{'input': 'argMaxInput'}, {'axis': 2}],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [1, 2, 2, 1, 2, 2],
          'descriptor': {shape: [2, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 4D tensor, axis=3, options.keepDimensions=true',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4, 1], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 4D tensor, axis=3, options.keepDimensions=false',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMax float16 4D tensor, axis=3, explicit options.outputDataType=\'int32\'',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'outputDataType': 'int32'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name':
        'argMax float16 4D tensor, axis=3, options.outputDataType=\'int64\'',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'outputDataType': 'int64'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2n, 0n, 2n, 1n, 2n, 0n, 2n, 1n],
          'descriptor': {shape: [2, 1, 4], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'argMax float16 4D tensor, axis=3, all options',
    'graph': {
      'inputs': {
        'argMaxInput': {
          'data': [
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125,
            -51.09375, -6.5390625, 73.8125, 88.4375,  -5.29296875, -79.1875,
            -41.6875,  73.8125,    88.4375, -84.9375, -61.5,       -98.3125
          ],
          'descriptor': {shape: [2, 1, 4, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'argMax',
        'arguments': [
          {'input': 'argMaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': true, 'outputDataType': 'int64'}}
        ],
        'outputs': 'argMaxOutput'
      }],
      'expectedOutputs': {
        'argMaxOutput': {
          'data': [2n, 0n, 2n, 1n, 2n, 0n, 2n, 1n],
          'descriptor': {shape: [2, 1, 4, 1], dataType: 'int64'}
        }
      }
    }
  },
];

if (navigator.ml) {
  argMinMaxTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getArgMinMaxPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
