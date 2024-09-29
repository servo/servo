// META: title=test WebNN API cumulativeSum operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://github.com/webmachinelearning/webnn/issues/375#issuecomment-2292466613
// Sums the elements of a tensor along an axis.
//
// dictionary MLCumulativeSumOptions {
//   bool exclusive = false; // Post-sum addition rather than inclusive pre-sum.
//   bool reversed = false; // Reverse the summation direction.
// };
//
// MLOperand cumulativeSum(MLOperand input, unsigned long axis, optional
// MLCumulativeSumOptions options = {});

const getCumulativeSumPrecisionTolerance = (graphResources) => {
  const args = graphResources.operators[0].arguments;
  const inputShape =
      graphResources.inputs[args[0][Object.keys(args[0])[0]]].descriptor.shape;
  const axis = args[1][Object.keys(args[1])[0]];
  let tolerance = inputShape[axis] - 1;

  const toleranceValueDict = {float32: tolerance, int32: 0};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const cumulativeSumTests = [
  {
    'name': 'cumulativeSum with float32 input and default options.',
    'graph': {
      'inputs': {
        'cumulativeSumInput': {
          'data': [
            60.42374038696289, -86.92247772216797, -19.496112823486328,
            -15.150615692138672, 13.455190658569336, 45.433597564697266,
            61.082862854003906, 70.71882629394531, -31.278579711914062,
            56.08354187011719, 38.992767333984375, -3.27536940574646
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cumulativeSum',
        'arguments': [
          {'input': 'cumulativeSumInput'},
          {'axis': 3},
        ],
        'outputs': 'cumulativeSumOutput'
      }],
      'expectedOutputs': {
        'cumulativeSumOutput': {
          'data': [
            60.4237404, -26.4987373, -45.994854, -61.1454659, 13.4551907,
            58.8887863, 119.9716568, 190.6904907, -31.2785797, 24.8049622,
            63.7977295, 60.5223611
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cumulativeSum with int32 input and axis = 2.',
    'graph': {
      'inputs': {
        'cumulativeSumInput': {
          'data': [2, 1, 3, 5, 3, 8, 7, 3, 9, 6, 2, 4],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'int32'}
        }
      },
      'operators': [{
        'name': 'cumulativeSum',
        'arguments': [
          {'input': 'cumulativeSumInput'},
          {'axis': 2},
        ],
        'outputs': 'cumulativeSumOutput'
      }],
      'expectedOutputs': {
        'cumulativeSumOutput': {
          'data': [2, 1, 3, 5, 5, 9, 10, 8, 14, 15, 12, 12],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'int32'}
        }
      }
    }
  },
  {
    'name': 'cumulativeSum with float32 input and set exclusive to true.',
    'graph': {
      'inputs': {
        'cumulativeSumInput': {
          'data': [
            60.42374038696289, -86.92247772216797, -19.496112823486328,
            -15.150615692138672, 13.455190658569336, 45.433597564697266,
            61.082862854003906, 70.71882629394531, -31.278579711914062,
            56.08354187011719, 38.992767333984375, -3.27536940574646
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cumulativeSum',
        'arguments': [
          {'input': 'cumulativeSumInput'},
          {'axis': 3},
          {'options': {'exclusive': true}},
        ],
        'outputs': 'cumulativeSumOutput'
      }],
      'expectedOutputs': {
        'cumulativeSumOutput': {
          'data': [
            0.0, 60.4237404, -26.4987373, -45.994854, 0.0, 13.4551907,
            58.8887863, 119.9716568, 0.0, -31.2785797, 24.8049622, 63.7977295
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'cumulativeSum with float32 input and set reversed to true.',
    'graph': {
      'inputs': {
        'cumulativeSumInput': {
          'data': [
            60.42374038696289, -86.92247772216797, -19.496112823486328,
            -15.150615692138672, 13.455190658569336, 45.433597564697266,
            61.082862854003906, 70.71882629394531, -31.278579711914062,
            56.08354187011719, 38.992767333984375, -3.27536940574646
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'cumulativeSum',
        'arguments': [
          {'input': 'cumulativeSumInput'},
          {'axis': 3},
          {'options': {'reversed': true}},
        ],
        'outputs': 'cumulativeSumOutput'
      }],
      'expectedOutputs': {
        'cumulativeSumOutput': {
          'data': [
            -61.1454659, -121.5692139, -34.6467285, -15.1506157, 190.6904907,
            177.2352905, 131.8016968, 70.7188263, 60.5223618, 91.8009415,
            35.7173996, -3.2753694
          ],
          'descriptor': {shape: [1, 1, 3, 4], dataType: 'float32'}
        }
      }
    }
  },
];

if (navigator.ml) {
  cumulativeSumTests.forEach((test) => {
    webnn_conformance_test(
        buildGraphAndCompute, getCumulativeSumPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
