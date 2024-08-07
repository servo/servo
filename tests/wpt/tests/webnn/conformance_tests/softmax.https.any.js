// META: title=test WebNN API softmax operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-softmax-method
// Compute the softmax values of the N-D input tensor along the given axis.
//
// MLOperand softmax(MLOperand input, unsigned long axis);


const getSoftmaxPrecisionTolerance = (graphResources) => {
  const args = graphResources.operators[0].arguments;
  const inputShape = graphResources.inputs[args[0][Object.keys(args[0])[0]]]
                         .descriptor.dimensions;
  const axis = args.length === 2 ? args[1][Object.keys(args[1])[0]] : 1;
  const tolerance = inputShape[axis] * 3 + 3;
  const toleranceValueDict = {float32: tolerance, float16: tolerance};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const softmaxTests = [
  {
    'name': 'softmax float32 2D constant tensor all positive',
    'graph': {
      'inputs': {
        'softmaxInput': {
          'data': [
            7.9037346839904785, 6.358251571655273,   4.833756923675537,
            9.5791654586792,    0.21071857213974,    4.554958820343018,
            7.150174140930176,  8.330297470092773,   1.5359858274459839,
            6.63361930847168,   1.4539369344711304,  0.213418647646904,
            5.257819652557373,  8.192137718200684,   8.16172981262207,
            2.874434232711792,  8.950733184814453,   6.111632823944092,
            1.6371468305587769, 0.27626121044158936, 5.02822732925415,
            3.8983259201049805, 2.8967113494873047,  6.88947057723999
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'softmax',
        'arguments': [{'input': 'softmaxInput'}],
        'outputs': 'softmaxOutput'
      }],
      'expectedOutputs': {
        'softmaxOutput': {
          'data': [
            0.15068615972995758,    0.03212761878967285,
            0.006995180621743202,   0.8048291206359863,
            0.00006871300138300285, 0.005293202120810747,
            0.2057899534702301,     0.6698001027107239,
            0.0007502624066546559,  0.1227685883641243,
            0.0006911618984304368,  0.00019990770670119673,
            0.012398251332342625,   0.23319464921951294,
            0.22621041536331177,    0.0011435872875154018,
            0.4979347288608551,     0.029118351638317108,
            0.004253828432410955,   0.001090824487619102,
            0.12633030116558075,    0.040812913328409195,
            0.014990009367465973,   0.8125221133232117
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'softmax float32 2D tensor all positive',
    'graph': {
      'inputs': {
        'softmaxInput': {
          'data': [
            7.9037346839904785, 6.358251571655273,   4.833756923675537,
            9.5791654586792,    0.21071857213974,    4.554958820343018,
            7.150174140930176,  8.330297470092773,   1.5359858274459839,
            6.63361930847168,   1.4539369344711304,  0.213418647646904,
            5.257819652557373,  8.192137718200684,   8.16172981262207,
            2.874434232711792,  8.950733184814453,   6.111632823944092,
            1.6371468305587769, 0.27626121044158936, 5.02822732925415,
            3.8983259201049805, 2.8967113494873047,  6.88947057723999
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'}
        }
      },
      'operators': [{
        'name': 'softmax',
        'arguments': [{'input': 'softmaxInput'}],
        'outputs': 'softmaxOutput'
      }],
      'expectedOutputs': {
        'softmaxOutput': {
          'data': [
            0.15068615972995758,    0.03212761878967285,
            0.006995180621743202,   0.8048291206359863,
            0.00006871300138300285, 0.005293202120810747,
            0.2057899534702301,     0.6698001027107239,
            0.0007502624066546559,  0.1227685883641243,
            0.0006911618984304368,  0.00019990770670119673,
            0.012398251332342625,   0.23319464921951294,
            0.22621041536331177,    0.0011435872875154018,
            0.4979347288608551,     0.029118351638317108,
            0.004253828432410955,   0.001090824487619102,
            0.12633030116558075,    0.040812913328409195,
            0.014990009367465973,   0.8125221133232117
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'softmax float32 2D tensor all negative',
    'graph': {
      'inputs': {
        'softmaxInput': {
          'data': [
            -3.3118433952331543, -3.3389549255371094, -3.4102790355682373,
            -6.697193145751953,  -7.896223545074463,  -3.308168888092041,
            -3.2309720516204834, -4.315771579742432,  -9.311088562011719,
            -3.9236626625061035, -3.780721426010132,  -6.034926891326904,
            -3.9196677207946777, -2.2234842777252197, -9.326531410217285,
            -1.4882491827011108, -6.302842617034912,  -5.53147554397583,
            -1.8421411514282227, -4.994808197021484,  -9.527292251586914,
            -4.985682964324951,  -8.421041488647461,  -6.235629558563232
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'}
        }
      },
      'operators': [{
        'name': 'softmax',
        'arguments': [{'input': 'softmaxInput'}],
        'outputs': 'softmaxOutput'
      }],
      'expectedOutputs': {
        'softmaxOutput': {
          'data': [
            0.2546302080154419,   0.24781952798366547,   0.2307596504688263,
            0.008623254485428333, 0.002599793951958418,  0.2555675804615021,
            0.40352678298950195,  0.13637976348400116,   0.0009232329903170466,
            0.20185552537441254,  0.23287305235862732,   0.024441635236144066,
            0.0551743283867836,   0.3008708655834198,    0.0002474947541486472,
            0.6276082992553711,   0.0050902292132377625, 0.011008745059370995,
            0.9090295433998108,   0.0388500951230526,    0.00041779119055718184,
            0.039206232875585556, 0.0012629841221496463, 0.011233373545110226
          ],
          'descriptor': {'dimensions': [4, 6], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'softmax float32 3D constant tensor',
    'graph': {
      'inputs': {
        'softmaxInput': {
          'data': [
            0.4301910996437073, 0.5471914410591125, -1.1637765169143677,
            0.18390046060085297, 0.583903968334198, 0.17356790602207184,
            0.5397239923477173, -0.9535139799118042, -0.5920282602310181,
            -0.17344485223293304, 0.14395014941692352, -0.37920907139778137
          ],
          'descriptor': {'dimensions': [1, 3, 4], 'dataType': 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'softmax',
        'arguments': [{'input': 'softmaxInput'}, {'axis': 1}],
        'outputs': 'softmaxOutput'
      }],
      'expectedOutputs': {
        'softmaxOutput': {
          'data': [
            0.39589041471481323, 0.45983806252479553, 0.09812675416469574,
            0.529077410697937, 0.4616699814796448, 0.31647709012031555,
            0.5390242338180542, 0.16964708268642426, 0.142439603805542,
            0.22368484735488892, 0.36284899711608887, 0.3012755215167999
          ],
          'descriptor': {'dimensions': [1, 3, 4], 'dataType': 'float32'}
        }
      }
    }
  },
  {
    'name': 'softmax float32 4D tensor',
    'graph': {
      'inputs': {
        'softmaxInput': {
          'data': [
            0.4301910996437073, 0.5471914410591125, -1.1637765169143677,
            0.18390046060085297, 0.583903968334198, 0.17356790602207184,
            0.5397239923477173, -0.9535139799118042, -0.5920282602310181,
            -0.17344485223293304, 0.14395014941692352, -0.37920907139778137
          ],
          'descriptor': {'dimensions': [3, 4, 1, 1], 'dataType': 'float32'}
        }
      },
      'operators': [{
        'name': 'softmax',
        'arguments': [{'input': 'softmaxInput'}, {'axis': 1}],
        'outputs': 'softmaxOutput'
      }],
      'expectedOutputs': {
        'softmaxOutput': {
          'data': [
            0.3216537833213806, 0.3615773916244507, 0.06533370912075043,
            0.25143513083457947, 0.35271573066711426, 0.23400123417377472,
            0.33747196197509766, 0.07581108063459396, 0.17110128700733185,
            0.26004093885421753, 0.3571779429912567, 0.2116798311471939
          ],
          'descriptor': {'dimensions': [3, 4, 1, 1], 'dataType': 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  softmaxTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getSoftmaxPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
