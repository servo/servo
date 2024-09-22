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
// dictionary MLArgMinMaxOptions {
//   boolean keepDimensions = false;
// };
//
// MLOperand argMin(MLOperand input, [EnforceRange] unsigned long axis,
//                  optional MLArgMinMaxOptions options = {});
// MLOperand argMax(MLOperand input, [EnforceRange] unsigned long axis,
//                  optional MLArgMinMaxOptions options = {});


const getArgMinMaxPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {int64: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const argMinMaxTests = [
  // argMin tests
  {
    'name': 'argMin float32 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float32 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput':
            {'data': [7], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMin float32 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 0}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
        'arguments': [{'input': 'argminInput'}, {'axis': 2}],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
          {'input': 'argminInput'}, {'axis': 0},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
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
        'argminInput': {
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
          {'input': 'argminInput'}, {'axis': 0},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argminOutput'
      }],
      'expectedOutputs': {
        'argminOutput': {
          'data': [1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
          'descriptor': {shape: [1, 4, 3], dataType: 'int32'}
        }
      }
    }
  },

  // argMax tests
  {
    'name': 'argMax float32 1D constant tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float32 1D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput':
            {'data': [3], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'argMax float32 2D tensor, axis=0, default options',
    'graph': {
      'inputs': {
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 0}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
        'arguments': [{'input': 'argmaxInput'}, {'axis': 2}],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
          {'input': 'argmaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': true}}
        ],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
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
        'argmaxInput': {
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
          {'input': 'argmaxInput'}, {'axis': 3},
          {'options': {'keepDimensions': false}}
        ],
        'outputs': 'argmaxOutput'
      }],
      'expectedOutputs': {
        'argmaxOutput': {
          'data': [2, 0, 2, 1, 2, 0, 2, 1],
          'descriptor': {shape: [2, 1, 4], dataType: 'int32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  argMinMaxTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getArgMinMaxPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
