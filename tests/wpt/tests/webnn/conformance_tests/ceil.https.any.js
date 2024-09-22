// META: title=test WebNN API element-wise ceil operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-unary
// Compute the ceiling of the input tensor, element-wise.
//
// MLOperand ceil(MLOperand input);


const getCeilPrecisionTolerance = (graphResources) => {
  const toleranceValueDict = {float32: 0, float16: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const ceilTests = [
  {
    'name': 'ceil float32 0D scalar',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [67.38941955566406],
          'descriptor': {shape: [], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput':
            {'data': [68], 'descriptor': {shape: [], dataType: 'float32'}}
      }
    }
  },
  {
    'name': 'ceil float32 1D constant tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'ceil float32 1D tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [24], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'ceil float32 2D tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'ceil float32 3D tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [2, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'ceil float32 4D tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [2, 2, 2, 3], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'ceil float32 5D tensor',
    'graph': {
      'inputs': {
        'ceilInput': {
          'data': [
            67.38941955566406,   36.78218460083008,   99.10649108886719,
            -22.58710479736328,  32.70173645019531,   17.68880844116211,
            5.631034851074219,   12.965238571166992,  83.1319351196289,
            -29.292461395263672, 19.84463119506836,   65.2790298461914,
            26.31110954284668,   24.285673141479492,  -48.39767074584961,
            -5.617412567138672,  61.53380584716797,   -87.81197357177734,
            69.71428680419922,   5.0031023025512695,  84.36833953857422,
            -9.390542030334473,  -27.856616973876953, -34.895931243896484
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'ceil',
        'arguments': [{'input': 'ceilInput'}],
        'outputs': 'ceilOutput'
      }],
      'expectedOutputs': {
        'ceilOutput': {
          'data': [
            68, 37, 100, -22, 33, 18,  6,  13, 84, -29, 20,  66,
            27, 25, -48, -5,  62, -87, 70, 6,  85, -9,  -27, -34
          ],
          'descriptor': {shape: [2, 1, 4, 1, 3], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  ceilTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getCeilPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
