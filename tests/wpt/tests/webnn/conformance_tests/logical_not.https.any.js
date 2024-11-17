// META: title=test WebNN API element-wise logicalNot operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-logical
// Invert the values of the input tensor to values 0 or 1, element-wise.
//
// MLOperand logicalNot(MLOperand a);


const getLogicalNotPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {uint8: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const logicalNotTests = [
  {
    'name': 'logicalNot uint8 0D scalar',
    'graph': {
      'inputs': {
        'logicalNotInput':
            {'data': [1], 'descriptor': {shape: [], dataType: 'uint8'}}
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput':
            {'data': [0], 'descriptor': {shape: [], dataType: 'uint8'}}
      }
    }
  },
  {
    'name': 'logicalNot uint8 1D constant tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [24], dataType: 'uint8'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalNot uint8 1D tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [24], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalNot uint8 2D tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [4, 6], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalNot uint8 3D tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalNot uint8 4D tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'uint8'}
        }
      }
    }
  },
  {
    'name': 'logicalNot uint8 5D tensor',
    'graph': {
      'inputs': {
        'logicalNotInput': {
          'data': [
            204, 130, 90, 0,   147, 42, 10,  18,  13,  235, 0,   233,
            53,  83,  9,  254, 69,  56, 219, 109, 171, 0,   228, 135
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'uint8'}
        }
      },
      'operators': [{
        'name': 'logicalNot',
        'arguments': [{'input': 'logicalNotInput'}],
        'outputs': 'logicalNotOutput'
      }],
      'expectedOutputs': {
        'logicalNotOutput': {
          'data': [
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'uint8'}
        }
      }
    }
  }
];

if (navigator.ml) {
  logicalNotTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getLogicalNotPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
