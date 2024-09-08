// META: title=test WebNN API tile operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://github.com/webmachinelearning/webnn/issues/375
// Represents the tile operation that repeats a tensor the given number of
// times along each axis.
//
// MLOperand tile(
//     MLOperand input, sequence<unsigned long> repetitions, optional
//     MLOperatorOptions options = {});


const getTilePrecisionTolerance = (graphResources) => {
  return {metricType: 'ULP', value: 0};
};

const tileTests = [
  {
    'name': 'tile float32 1D constant tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 3, 4, 1, 2, 3, 4],
          'descriptor': {'dimensions': [8], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'tile uint32 2D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {'dimensions': [2, 2], 'dataType': 'uint32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [2, 3]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4,
            1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'uint32'}
        }
      }
    }
  },
  {
    'name': 'tile int32 4D tensor',
    'graph': {
      'inputs': {
        'tileInput': {
          'data': [1, 2, 3, 4],
          'descriptor': {'dimensions': [1, 1, 2, 2], 'dataType': 'int32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'tile',
        'arguments': [{'input': 'tileInput'}, {'repetitions': [1, 1, 2, 2]}],
        'outputs': 'tileOutput'
      }],
      'expectedOutputs': {
        'tileOutput': {
          'data': [1, 2, 1, 2, 3, 4, 3, 4, 1, 2, 1, 2, 3, 4, 3, 4],
          'descriptor': {'dimensions': [1, 1, 4, 4], 'dataType': 'int32'}
        }
      }
    }
  },
];

if (navigator.ml) {
  tileTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getTilePrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
