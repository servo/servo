// META: title=test WebNN API reverse operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-reverse-method
// Reverse the order of the input tensor along specified axes.
//
// dictionary MLReverseOptions : MLOperatorOptions {
//   sequence<[EnforceRange] unsigned long> axes;
// };
//
// MLOperand reverse(MLOperand input, optional MLReverseOptions options = {});


const reverseTests = [
  {
    'name': 'reverse float32 2D input with default options',
    'graph': {
      'inputs': {
        'reverseInput': {
          'data': [
            -30.0561466217041, 99.56941986083984, 88.04620361328125,
            -91.87507629394531, -23.7972354888916, -91.28665161132812,
            -63.15204620361328, 12.0669527053833, -96.1172866821289,
            -44.77365493774414, -80.08650970458984, -64.43756866455078
          ],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reverse',
        'arguments': [{'input': 'reverseInput'}],
        'outputs': 'reverseOutput'
      }],
      'expectedOutputs': {
        'reverseOutput': {
          'data': [
            -64.43756866455078, -80.08650970458984, -44.77365493774414,
            -96.1172866821289, 12.0669527053833, -63.15204620361328,
            -91.28665161132812, -23.7972354888916, -91.87507629394531,
            88.04620361328125, 99.56941986083984, -30.0561466217041
          ],
          'descriptor': {shape: [3, 4], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reverse float32 3D input options.axes=[1, 2]',
    'graph': {
      'inputs': {
        'reverseInput': {
          'data': [
            -30.0561466217041, 99.56941986083984, 88.04620361328125,
            -91.87507629394531, -23.7972354888916, -91.28665161132812,
            -63.15204620361328, 12.0669527053833, -96.1172866821289,
            -44.77365493774414, -80.08650970458984, -64.43756866455078
          ],
          'descriptor': {shape: [3, 2, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reverse',
        'arguments': [{'input': 'reverseInput'}, {'options': {'axes': [1, 2]}}],
        'outputs': 'reverseOutput'
      }],
      'expectedOutputs': {
        'reverseOutput': {
          'data': [
            -91.87507629394531, 88.04620361328125, 99.56941986083984,
            -30.0561466217041, 12.0669527053833, -63.15204620361328,
            -91.28665161132812, -23.7972354888916, -64.43756866455078,
            -80.08650970458984, -44.77365493774414, -96.1172866821289
          ],
          'descriptor': {shape: [3, 2, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reverse float32 4D input options.axes=[3, 1]',
    'graph': {
      'inputs': {
        'reverseInput': {
          'data': [
            -30.0561466217041, 99.56941986083984, 88.04620361328125,
            -91.87507629394531, -23.7972354888916, -91.28665161132812,
            -63.15204620361328, 12.0669527053833, -96.1172866821289,
            -44.77365493774414, -80.08650970458984, -64.43756866455078
          ],
          'descriptor': {shape: [3, 2, 1, 2], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reverse',
        'arguments': [{'input': 'reverseInput'}, {'options': {'axes': [3, 1]}}],
        'outputs': 'reverseOutput'
      }],
      'expectedOutputs': {
        'reverseOutput': {
          'data': [
            -91.87507629394531, 88.04620361328125, 99.56941986083984,
            -30.0561466217041, 12.0669527053833, -63.15204620361328,
            -91.28665161132812, -23.7972354888916, -64.43756866455078,
            -80.08650970458984, -44.77365493774414, -96.1172866821289
          ],
          'descriptor': {shape: [3, 2, 1, 2], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'reverse float32 4D input options.axes=[]',
    'graph': {
      'inputs': {
        'reverseInput': {
          'data': [
            -30.0561466217041, 99.56941986083984, 88.04620361328125,
            -91.87507629394531, -23.7972354888916, -91.28665161132812,
            -63.15204620361328, 12.0669527053833, -96.1172866821289,
            -44.77365493774414, -80.08650970458984, -64.43756866455078
          ],
          'descriptor': {shape: [2, 1, 2, 3], dataType: 'float32'}
        }
      },
      'operators': [{
        'name': 'reverse',
        'arguments': [{'input': 'reverseInput'}, {'options': {'axes': []}}],
        'outputs': 'reverseOutput'
      }],
      'expectedOutputs': {
        'reverseOutput': {
          'data': [
            -30.0561466217041, 99.56941986083984, 88.04620361328125,
            -91.87507629394531, -23.7972354888916, -91.28665161132812,
            -63.15204620361328, 12.0669527053833, -96.1172866821289,
            -44.77365493774414, -80.08650970458984, -64.43756866455078
          ],
          'descriptor': {shape: [2, 1, 2, 3], dataType: 'float32'}
        }
      }
    }
  }
];

if (navigator.ml) {
  reverseTests.forEach((test) => {
    webnn_conformance_test(buildAndExecuteGraph, getPrecisionTolerance, test);
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
