// META: title=test WebNN API sign operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://github.com/webmachinelearning/webnn/issues/375#issuecomment-2292466613
// Represents the sign operation that return elementwise -1/0/1 depending on
// element sign.
//
// MLOperand sign(MLOperand input, optional MLOperatorOptions options = {});


const getSignPrecisionTolerance = (graphResources) => {
  return {metricType: 'ULP', value: 0};
};

const signTests = [
  {
    'name': 'sign float32 1D constant tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [
            -0.946033775806427, 0.9996118545532227, 0.21998752653598785,
            -0.22639396786689758
          ],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 1, 1, -1],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'sign float16 1D tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [
            -0.946033775806427, 0.9996118545532227, 0.21998752653598785,
            -0.22639396786689758
          ],
          'descriptor': {'dimensions': [4], 'dataType': 'float16'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 1, 1, -1],
          'descriptor': {'dimensions': [4], 'dataType': 'float16'}
        }
      }
    }
  },
  {
    'name': 'sign float32 1D tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [
            -0.946033775806427, 0.9996118545532227, 0.21998752653598785, 0.0
          ],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 1, 1, 0],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'sign float32 1D tensor with -infinity and +infinity',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [-0.946033775806427, 0.9996118545532227, -Infinity, Infinity],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 1, -1, 1],
          'descriptor': {'dimensions': [4], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'sign int32 2D tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [-1, 0, 1, 2],
          'descriptor': {'dimensions': [2, 2], 'dataType': 'int32'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 0, 1, 1],
          'descriptor': {'dimensions': [2, 2], 'dataType': 'int32'}
        }
      }
    }
  },
  {
    'name': 'sign int64 3D tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [-1, 0, 1, 2, -2, -1, 0, 1],
          'descriptor': {'dimensions': [2, 2, 2], 'dataType': 'int64'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 0, 1, 1, -1, -1, 0, 1],
          'descriptor': {'dimensions': [2, 2, 2], 'dataType': 'int64'}
        }
      }
    }
  },
  {
    'name': 'sign int8 4D tensor',
    'graph': {
      'inputs': {
        'signInput': {
          'data': [-1, 0, 1, 2, -2, -1, 0, 1],
          'descriptor': {'dimensions': [1, 2, 2, 2], 'dataType': 'int8'}
        }
      },
      'operators': [{
        'name': 'sign',
        'arguments': [{'input': 'signInput'}],
        'outputs': 'signOutput'
      }],
      'expectedOutputs': {
        'signOutput': {
          'data': [-1, 0, 1, 1, -1, -1, 0, 1],
          'descriptor': {'dimensions': [1, 2, 2, 2], 'dataType': 'int8'}
        }
      }
    }
  },
];

if (navigator.ml) {
  signTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getSignPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
