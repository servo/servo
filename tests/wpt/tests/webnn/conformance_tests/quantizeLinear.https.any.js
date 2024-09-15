// META: title=test WebNN API quantizeLinear operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Calculate a floating-point input down to a low-precision integer
// (typically uint8 with a zero-point bias) following the expression:
//    output = clamp(roundToNearestEvens(input / scale) + zeroPoint, 0, 255).
//
// MLOperand quantizeLinear(
//     MLOperand input, MLOperand scale, MLOperand zeroPoint,
//     optional MLOperatorOptions options = {});


const getQuantizeLinearPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {int8: 1, uint8: 1};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const quantizeLinearTests = [
  {
    'name':
        'quantizeLinear float32 0D scalar tensor with int8 scalar zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [10.794857501983643],
          'descriptor': {'dimensions': [], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [1.1202747821807861],
          'descriptor': {'dimensions': [], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [1],
          'descriptor': {'dimensions': [], 'dataType': 'int8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput':
            {'data': [11], 'descriptor': {'dimensions': [], 'dataType': 'int8'}}
      }
    }
  },
  {
    'name': 'quantizeLinear float32 1D constant tensor broadcasting zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [
            9.343092918395996,
            0.2800687253475189,
            -4.617084980010986,
            1.1202747821807861,
          ],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128],
          'descriptor': {'dimensions': [], 'dataType': 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [128, 111, 126, 133],
          'descriptor': {'dimensions': [4], 'dataType': 'uint8'}
        }
      }
    }
  },
  {
    'name':
        'quantizeLinear float32 4D constant tensor broadcasting scale and zeroPoint',
    'graph': {
      'inputs': {
        'quantizeLinearInput': {
          'data': [
            -2.549168109893799, -4.794857501983643, 8.413617134094238,
            6.108623504638672
          ],
          'descriptor': {'dimensions': [1, 1, 2, 2], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearScale': {
          'data': [0.2800687253475189, -4.617084980010986],
          'descriptor': {'dimensions': [2, 1], 'dataType': 'float32'},
          'constant': true
        },
        'quantizeLinearZeroPoint': {
          'data': [128],
          'descriptor': {'dimensions': [], 'dataType': 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'quantizeLinear',
        'arguments': [
          {'input': 'quantizeLinearInput'}, {'scale': 'quantizeLinearScale'},
          {'zeroPoint': 'quantizeLinearZeroPoint'}
        ],
        'outputs': 'quantizeLinearOutput'
      }],
      'expectedOutputs': {
        'quantizeLinearOutput': {
          'data': [119, 111, 126, 127],
          'descriptor': {'dimensions': [1, 1, 2, 2], 'dataType': 'uint8'}
        }
      }
    }
  }
];

if (navigator.ml) {
  quantizeLinearTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getQuantizeLinearPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
