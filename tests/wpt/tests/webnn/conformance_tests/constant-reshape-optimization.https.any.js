// META: title=test constant reshape optimization
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

const tests = [{
  'name': 'reshape + reshape + reshape + instanceNormalization float32',
  'graph': {
    'inputs': {
      'originalInput': {
        'data': [
          -97.949951171875,    29.44037628173828,  -73.92131042480469,
          -38.11185836791992,  41.33772659301758,  -59.77853012084961,
          -74.66901397705078,  -68.16508483886719, 35.82481384277344,
          -6.948329448699951,  54.42462158203125,  47.53074645996094,
          66.93562316894531,   76.74034881591797,  5.6758809089660645,
          25.68659210205078,   37.37651062011719,  56.252689361572266,
          -16.574905395507812, 42.949893951416016, 73.8739242553711,
          -99.00035095214844,  -33.11322784423828, -17.380685806274414
        ],
        'descriptor': {shape: [3, 8], dataType: 'float32'},
        'constant': true
      },
      'originalScale': {
        'data': [-94.42772674560547, 66.69620513916016, -98.56572723388672],
        'descriptor': {shape: [1, 3, 1, 1], dataType: 'float32'},
        'constant': true
      },
      'originalBias': {
        'data': [-33.048641204833984, 4.511423587799072, -37.93617248535156],
        'descriptor': {shape: [1, 3, 1, 1], dataType: 'float32'},
        'constant': true
      },
    },
    'operators': [
      {
        'name': 'reshape',
        'arguments': [{'input': 'originalInput'}, {'newShape': [2, 3, 2, 2]}],
        'outputs': 'reshapedInput'
      },
      {
        'name': 'reshape',
        'arguments': [{'input': 'originalScale'}, {'newShape': [3]}],
        'outputs': 'reshapedScale'
      },
      {
        'name': 'reshape',
        'arguments': [{'input': 'originalBias'}, {'newShape': [3]}],
        'outputs': 'reshapedBias'
      },
      {
        'name': 'instanceNormalization',
        'arguments': [
          {'input': 'reshapedInput'}, {
            'options': {
              'scale': 'reshapedScale',
              'bias': 'reshapedBias',
              'epsilon': 0.000001,
              'layout': 'nchw'
            }
          }
        ],
        'outputs': 'instanceNormOutput'
      }
    ],
    'expectedOutputs': {
      'instanceNormOutput': {
        'data': [
          70.77738189697266,   -179.65554809570312, 23.540178298950195,
          -46.8565788269043,   119.31526184082031,  -22.847837448120117,
          -43.782920837402344, -34.6388053894043,   -50.821895599365234,
          126.01134490966797,  -127.71744537353516, -99.2166976928711,
          -108.09159851074219, -139.83889770507812, 90.26488494873047,
          25.471038818359375,  22.237276077270508,  67.60342407226562,
          -107.4271011352539,  35.6320915222168,    -186.15142822265625,
          90.01669311523438,   -15.238543510437012, -40.37141418457031
        ],
        'descriptor': {shape: [2, 3, 2, 2], dataType: 'float32'}
      }
    }
  }
}];

if (navigator.ml) {
  tests.forEach((test) => {
    webnn_conformance_test(
        buildAndExecuteGraph, getInstanceNormPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
