// META: title=test WebNN API cast operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-cast
// Cast each element in the input tensor to the target data type.
// MLOperand cast(MLOperand input, MLOperandDataType type);


const getCastPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {
    float32: 1,
    float16: 1,
    int32: 0,
    uint32: 0,
    int64: 0,
    int8: 0,
    uint8: 0
  };
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const castTests = [
  {
    'name': 'cast float32 0D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [84.77753448486328],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput':
            {'data': [84], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'cast float32 1D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,   -43.5,
            52.84621810913086,   -99.9583511352539,
            6.729493141174316,   92.66157531738281,
            -10.377813339233398, 106.65289306640625,
            -7.126272678375244,  91.51563262939453,
            -50.87134552001953,  83.38890075683594,
            72.9759750366211,    -31.015382766723633,
            79.94034576416016,   41.5,
            35.727149963378906,  -2.5,
            -96.05252838134766,  -86.76212310791016,
            -27.49382972717285,  -23.836687088012695,
            70.77123260498047,   83.5
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, -43, 52, -99, 6,  92, -10, 106, -7,  91,  -50, 83,
            72,  -31, 79, 41,  35, -2, -96, -86, -27, -23, 70,  83
          ],
          'descriptor': {shape: [24], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 2D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,   -43.5,
            52.84621810913086,   -99.9583511352539,
            6.729493141174316,   92.66157531738281,
            -10.377813339233398, 106.65289306640625,
            -7.126272678375244,  91.51563262939453,
            -50.87134552001953,  83.38890075683594,
            72.9759750366211,    -31.015382766723633,
            79.94034576416016,   41.5,
            35.727149963378906,  -2.5,
            -96.05252838134766,  -86.76212310791016,
            -27.49382972717285,  -23.836687088012695,
            70.77123260498047,   83.5
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, -43, 52, -99, 6,  92, -10, 106, -7,  91,  -50, 83,
            72,  -31, 79, 41,  35, -2, -96, -86, -27, -23, 70,  83
          ],
          'descriptor': {shape: [4, 6], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 3D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,   -43.5,
            52.84621810913086,   -99.9583511352539,
            6.729493141174316,   92.66157531738281,
            -10.377813339233398, 106.65289306640625,
            -7.126272678375244,  91.51563262939453,
            -50.87134552001953,  83.38890075683594,
            72.9759750366211,    -31.015382766723633,
            79.94034576416016,   41.5,
            35.727149963378906,  -2.5,
            -96.05252838134766,  -86.76212310791016,
            -27.49382972717285,  -23.836687088012695,
            70.77123260498047,   83.5
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, -43, 52, -99, 6,  92, -10, 106, -7,  91,  -50, 83,
            72,  -31, 79, 41,  35, -2, -96, -86, -27, -23, 70,  83
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,   -43.5,
            52.84621810913086,   -99.9583511352539,
            6.729493141174316,   92.66157531738281,
            -10.377813339233398, 106.65289306640625,
            -7.126272678375244,  91.51563262939453,
            -50.87134552001953,  83.38890075683594,
            72.9759750366211,    -31.015382766723633,
            79.94034576416016,   41.5,
            35.727149963378906,  -2.5,
            -96.05252838134766,  -86.76212310791016,
            -27.49382972717285,  -23.836687088012695,
            70.77123260498047,   83.5
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, -43, 52, -99, 6,  92, -10, 106, -7,  91,  -50, 83,
            72,  -31, 79, 41,  35, -2, -96, -86, -27, -23, 70,  83
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 5D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,   -43.5,
            52.84621810913086,   -99.9583511352539,
            6.729493141174316,   92.66157531738281,
            -10.377813339233398, 106.65289306640625,
            -7.126272678375244,  91.51563262939453,
            -50.87134552001953,  83.38890075683594,
            72.9759750366211,    -31.015382766723633,
            79.94034576416016,   41.5,
            35.727149963378906,  -2.5,
            -96.05252838134766,  -86.76212310791016,
            -27.49382972717285,  -23.836687088012695,
            70.77123260498047,   83.5
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, -43, 52, -99, 6,  92, -10, 106, -7,  91,  -50, 83,
            72,  -31, 79, 41,  35, -2, -96, -86, -27, -23, 70,  83
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,  43.60371780395508,  52.84621810913086,
            99.9583511352539,   6.729493141174316,  92.66157531738281,
            10.377813339233398, 106.65289306640625, 7.126272678375244,
            91.51563262939453,  50.87134552001953,  83.38890075683594,
            72.9759750366211,   31.015382766723633, 79.94034576416016,
            41.844703674316406, 35.727149963378906, 2.614182949066162,
            96.05252838134766,  86.76212310791016,  27.49382972717285,
            23.836687088012695, 70.77123260498047,  83.8347396850586
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102.1875, 43.59375,  52.84375, 99.9375,  6.73046875, 92.6875,
            10.375,   106.625,   7.125,    91.5,     50.875,     83.375,
            73,       31.015625, 79.9375,  41.84375, 35.71875,   2.61328125,
            96.0625,  86.75,     27.5,     23.84375, 70.75,      83.8125
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to uint32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,  43.60371780395508,  52.84621810913086,
            99.9583511352539,   6.729493141174316,  92.66157531738281,
            10.377813339233398, 106.65289306640625, 7.126272678375244,
            91.51563262939453,  50.87134552001953,  83.38890075683594,
            72.9759750366211,   31.015382766723633, 79.94034576416016,
            41.844703674316406, 35.727149963378906, 2.614182949066162,
            96.05252838134766,  86.76212310791016,  27.49382972717285,
            23.836687088012695, 70.77123260498047,  83.8347396850586
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, 43, 52, 99, 6,  92, 10, 106, 7,  91, 50, 83,
            72,  31, 79, 41, 35, 2,  96, 86,  27, 23, 70, 83
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,  43.60371780395508,  52.84621810913086,
            99.9583511352539,   6.729493141174316,  92.66157531738281,
            10.377813339233398, 106.65289306640625, 7.126272678375244,
            91.51563262939453,  50.87134552001953,  83.38890075683594,
            72.9759750366211,   31.015382766723633, 79.94034576416016,
            41.844703674316406, 35.727149963378906, 2.614182949066162,
            96.05252838134766,  86.76212310791016,  27.49382972717285,
            23.836687088012695, 70.77123260498047,  83.8347396850586
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '102', '43', '52', '99', '6',  '92', '10', '106',
            '7',   '91', '50', '83', '72', '31', '79', '41',
            '35',  '2',  '96', '86', '27', '23', '70', '83'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,  43.60371780395508,  52.84621810913086,
            99.9583511352539,   6.729493141174316,  92.66157531738281,
            10.377813339233398, 106.65289306640625, 7.126272678375244,
            91.51563262939453,  50.87134552001953,  83.38890075683594,
            72.9759750366211,   31.015382766723633, 79.94034576416016,
            41.844703674316406, 35.727149963378906, 2.614182949066162,
            96.05252838134766,  86.76212310791016,  27.49382972717285,
            23.836687088012695, 70.77123260498047,  83.8347396850586
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, 43, 52, 99, 6,  92, 10, 106, 7,  91, 50, 83,
            72,  31, 79, 41, 35, 2,  96, 86,  27, 23, 70, 83
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  },
  {
    'name': 'cast float32 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            102.1578369140625,  43.60371780395508,  52.84621810913086,
            99.9583511352539,   6.729493141174316,  92.66157531738281,
            10.377813339233398, 106.65289306640625, 7.126272678375244,
            91.51563262939453,  50.87134552001953,  83.38890075683594,
            72.9759750366211,   31.015382766723633, 79.94034576416016,
            41.844703674316406, 35.727149963378906, 2.614182949066162,
            96.05252838134766,  86.76212310791016,  27.49382972717285,
            23.836687088012695, 70.77123260498047,  83.8347396850586
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            102, 43, 52, 99, 6,  92, 10, 106, 7,  91, 50, 83,
            72,  31, 79, 41, 35, 2,  96, 86,  27, 23, 70, 83
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            3,  32, 62,  51, 87, 106, 125, 112, 70, 39, 10,  21,
            99, 16, 115, 66, 49, 115, 77,  57,  61, 12, 101, 123
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to uint32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            3,  32, 62,  51, 87, 106, 125, 112, 70, 39, 10,  21,
            99, 16, 115, 66, 49, 115, 77,  57,  61, 12, 101, 123
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '3',  '32',  '62', '51', '87', '106', '125', '112',
            '70', '39',  '10', '21', '99', '16',  '115', '66',
            '49', '115', '77', '57', '61', '12',  '101', '123'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            3,  32, 62,  51, 87, 106, 125, 112, 70, 39, 10,  21,
            99, 16, 115, 66, 49, 115, 77,  57,  61, 12, 101, 123
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  },
  {
    'name': 'cast float16 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            3.103515625, 32.40625, 62.15625, 51.75,      87.0625,    106.25,
            125.375,     112.9375, 70.8125,  39.1875,    10.3515625, 21.234375,
            99.75,       16.125,   115.625,  66,         49.375,     115.75,
            77,          57.15625, 61.6875,  12.9296875, 101.25,     123.9375
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            3,  32, 62,  51, 87, 106, 125, 112, 70, 39, 10,  21,
            99, 16, 115, 66, 49, 115, 77,  57,  61, 12, 101, 123
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D constant tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'},
          'constant': true,
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '45', '55',  '11',  '21', '78', '104', '102', '66',
            '41', '110', '92',  '69', '48', '23',  '58',  '12',
            '33', '24',  '101', '87', '49', '118', '1',   '77'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  },
  {
    'name': 'cast int32 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            45, 55, 11, 21, 78, 104, 102, 66, 41, 110, 92, 69,
            48, 23, 58, 12, 33, 24,  101, 87, 49, 118, 1,  77
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '34',  '83',  '113', '31', '62',  '80', '8',  '40',
            '104', '42',  '6',   '91', '93',  '21', '40', '21',
            '51',  '110', '115', '12', '122', '68', '57', '72'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  },
  {
    'name': 'cast uint32 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            34, 83, 113, 31, 62, 80,  8,   40, 104, 42, 6,  91,
            93, 21, 40,  21, 51, 110, 115, 12, 122, 68, 57, 72
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to uint32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  },
  {
    'name': 'cast int64 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            50, 1,  28, 20, 102, 86,  70, 38, 50,  19, 11, 4,
            56, 77, 40, 80, 45,  127, 4,  87, 125, 26, 63, 11
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast int8 0D constant tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [17],
          'descriptor': {shape: [], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput':
            {'data': [17], 'descriptor': {shape: [], dataType: 'int32'}}
      }
    }
  },
  {
    'name': 'cast int8 1D constant tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [24], dataType: 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [24], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to uint32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '123', '17', '31',  '77', '88', '44', '84', '40',
            '14',  '64', '109', '4',  '2',  '0',  '45', '47',
            '72',  '88', '82',  '4',  '73', '36', '65', '117'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast int8 4D tensor to uint8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            123, 17, 31, 77, 88, 44, 84, 40, 14, 64, 109, 4,
            2,   0,  45, 47, 72, 88, 82, 4,  73, 36, 65,  117
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to float32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to float16',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'float16'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float16'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to int32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to uint32',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'uint32'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint32'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to int64',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int64'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            '10', '112', '121', '120', '22',  '105', '41', '30',
            '75', '121', '55',  '47',  '121', '24',  '16', '33',
            '97', '24',  '3',   '37',  '45',  '6',   '56', '57'
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast uint8 4D tensor to int8',
    'graph': {
      'inputs': {
        'castInput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'cast',
        'arguments': [{'input': 'castInput'}, {'type': 'int8'}],
        'outputs': 'castOutput'
      }],
      'expectedOutputs': {
        'castOutput': {
          'data': [
            10,  112, 121, 120, 22, 105, 41, 30, 75, 121, 55, 47,
            121, 24,  16,  33,  97, 24,  3,  37, 45, 6,   56, 57
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'int8'}
        }
      }
    }
  }
];

if (navigator.ml) {
  castTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getCastPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
