// META: title=test WebNN MLNumber
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlnumber-typedef

const getClampPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const mlNumberTests = [
  {
    'name': 'cast BigInt to int64',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [-21474836470, 21474836470, -2, 1, 0],
          'descriptor': {shape: [5], dataType: 'int64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [{'input': 'clampInput'}, {'options': {'minValue': -2n}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [-2, 21474836470, -2, 1, 0],
          'descriptor': {shape: [5], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast BigInt to int64 overflow',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [-21474836470, 21474836470, -2, 1, 0],
          'descriptor': {shape: [5], dataType: 'int64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [
          {'input': 'clampInput'},
          {'options': {'minValue': 9223372036854775820n}}
        ],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [
            9223372036854775807n, 9223372036854775807n, 9223372036854775807n,
            9223372036854775807n, 9223372036854775807n
          ],
          'descriptor': {shape: [5], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast BigInt to int64 underflow',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [-21474836470, 21474836470, -2, 1, 0],
          'descriptor': {shape: [5], dataType: 'int64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [
          {'input': 'clampInput'},
          {'options': {'maxValue': -9223372036854775820n}}
        ],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [
            -9223372036854775808, -9223372036854775808, -9223372036854775808,
            -9223372036854775808, -9223372036854775808
          ],
          'descriptor': {shape: [5], dataType: 'int64'}
        }
      }
    }
  },
  {
    'name': 'cast BigInt to uint64',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [42949672950, 127, 5, 0],
          'descriptor': {shape: [4], dataType: 'uint64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [{'input': 'clampInput'}, {'options': {'minValue': 5n}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [42949672950, 127, 5, 5],
          'descriptor': {shape: [4], dataType: 'uint64'}
        }
      }
    }
  },
  {
    'name': 'cast BigInt to uint64 overflow',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [42949672950, 127, 5, 0],
          'descriptor': {shape: [4], dataType: 'uint64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [
          {'input': 'clampInput'},
          {'options': {'minValue': 184467440737095511615n}}
        ],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [
            18446744073709551615n, 18446744073709551615n, 18446744073709551615n,
            18446744073709551615n
          ],
          'descriptor': {shape: [4], dataType: 'uint64'}
        }
      }
    }
  },
  {
    'name': 'cast BigInt to uint64 underflow',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [42949672950, 127, 5, 0],
          'descriptor': {shape: [4], dataType: 'uint64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [{'input': 'clampInput'}, {'options': {'maxValue': -1n}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [4], dataType: 'uint64'}
        }
      }
    }
  },
  {
    'name': 'cast float to integer',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [129, 4294967295, 127, 1, 0],
          'descriptor': {shape: [5], dataType: 'uint64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments':
            [{'input': 'clampInput'}, {'options': {'maxValue': 128.0}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [128, 128, 127, 1, 0],
          'descriptor': {shape: [5], dataType: 'uint64'}
        }
      }
    }
  },
  {
    'name': 'cast float to integer overflows',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [255, 127, 5, 0],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments':
            [{'input': 'clampInput'}, {'options': {'minValue': 1000.0}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [255, 255, 255, 255],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast float to integer underflows',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [255, 127, 5, 0],
          'descriptor': {shape: [4], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [{'input': 'clampInput'}, {'options': {'maxValue': -1.0}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [0, 0, 0, 0],
          'descriptor': {shape: [4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'cast fractional float to integer',
    'graph': {
      'inputs': {
        'clampInput': {
          'data': [3, 4, 5, -1, 0],
          'descriptor': {shape: [5], dataType: 'int64'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'clamp',
        'arguments': [{'input': 'clampInput'}, {'options': {'minValue': 3.9}}],
        'outputs': 'clampOutput'
      }],
      'expectedOutputs': {
        'clampOutput': {
          'data': [3, 4, 5, 3, 3],
          'descriptor': {shape: [5], dataType: 'int64'}
        }
      }
    }
  },
];

webnn_conformance_test(
    mlNumberTests, buildAndExecuteGraph, getClampPrecisionTolerance);
