// META: title=test WebNN API scatterElements operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

const getScatterElementsPrecisionTolerance = () => {
  return {metricType: 'ULP', value: 0};
};

const scatterElementsTests = [
  {
    'name': 'Scatter elements along axis 0',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
        },
        'updates': {
          'data': [1.0, 1.1, 1.2, 2.0, 2.1, 2.2],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [2.0, 1.1, 0.0, 1.0, 0.0, 2.2, 0.0, 2.1, 1.2],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'Scatter elements along axis 0 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 0, 2, 0, 2, 1],
          'descriptor': {shape: [2, 3], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1.0, 1.1, 1.2, 2.0, 2.1, 2.2],
          'descriptor': {shape: [2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 0}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [2.0, 1.1, 0.0, 1.0, 0.0, 2.2, 0.0, 2.1, 1.2],
          'descriptor': {shape: [3, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'Scatter elements along axis 1',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 3],
          'descriptor': {shape: [1, 2], dataType: 'int32'},
        },
        'updates': {
          'data': [1.1, 2.1],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1.0, 1.1, 3.0, 2.1, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'Scatter elements along axis 1 and constant indices',
    'graph': {
      'inputs': {
        'input': {
          'data': [1.0, 2.0, 3.0, 4.0, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        },
        'indices': {
          'data': [1, 3],
          'descriptor': {shape: [1, 2], dataType: 'int32'},
          'constant': true
        },
        'updates': {
          'data': [1.1, 2.1],
          'descriptor': {shape: [1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'scatterElements',
        'arguments': [
          {'input': 'input'}, {'indices': 'indices'}, {'updates': 'updates'},
          {'options': {'axis': 1}}
        ],
        'outputs': 'output'
      }],
      'expectedOutputs': {
        'output': {
          'data': [1.0, 1.1, 3.0, 2.1, 5.0],
          'descriptor': {shape: [1, 5], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  scatterElementsTests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getScatterElementsPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
