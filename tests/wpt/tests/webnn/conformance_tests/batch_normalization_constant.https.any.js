// META: title=test WebNN API batchNormalization operation with constant inputs
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-batchnorm
// Normalize the values of the input tensor using Batch-Normalization.
//
// dictionary MLBatchNormalizationOptions {
//   MLOperand scale;
//   MLOperand bias;
//   [EnforceRange] unsigned long axis = 1;
//   double epsilon = 1e-5;
// };
//
// MLOperand batchNormalization(
//     MLOperand input, MLOperand mean, MLOperand, variance,
//     optional MLBatchNormalizationOptions options = {});

const batchNormTests = [
  {
    'name': 'batchNormalization float32 2D constant tensors default options',
    'graph': {
      'inputs': {
        'bnInput': {
          'data': [
            -41.30733108520508,  64.08863830566406,    -63.376670837402344,
            -46.790367126464844, 83.02227020263672,    -80.08049011230469,
            -62.144378662109375, -0.10012771934270859, -40.90216064453125,
            56.96306228637695,   37.37249755859375,    57.046478271484375,
            82.05680084228516,   -86.1164321899414,    76.8831787109375,
            97.03362274169922,   -21.35103988647461,   -96.93824005126953,
            -9.359310150146484,  80.20824432373047,    -85.36802673339844,
            62.35185241699219,   -68.4724349975586,    -12.10716724395752
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'},
          'constant': true
        },
        'bnMean': {
          'data': [
            -7.814267635345459, -95.64129638671875, 38.15440368652344,
            -55.95203399658203, -87.86500549316406, -41.63645553588867
          ],
          'descriptor': {shape: [6], dataType: 'float32'},
          'constant': true
        },
        'bnVariance': {
          'data': [
            60.31186294555664, 26.43260383605957, 53.275634765625,
            40.146121978759766, 59.41098403930664, 35.99981689453125
          ],
          'descriptor': {shape: [6], dataType: 'float32'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'batchNormalization',
        'arguments': [
          {'input': 'bnInput'}, {'mean': 'bnMean'}, {'variance': 'bnVariance'}
        ],
        'outputs': 'bnOutput'
      }],
      'expectedOutputs': {
        'bnOutput': {
          'data': [
            -4.312741756439209,  31.068212509155273, -13.910240173339844,
            1.4459478855133057,  22.170541763305664, -6.407354354858398,
            -6.995829105377197,  18.583200454711914, -10.831125259399414,
            17.820920944213867,  16.2480411529541,   16.447195053100586,
            11.57226848602295,   1.8526301383972168, 5.306026458740234,
            24.145092010498047,  8.629376411437988,  -9.216986656188965,
            -0.1989477425813675, 34.203548431396484, -16.923160552978516,
            18.671411514282227,  2.5159497261047363, 4.921559810638428
          ],
          'descriptor': {shape: [4, 6], dataType: 'float32'}
        }
      }
    }
  },
  {
    'name': 'batchNormalization float16 2D constant tensors default options',
    'graph': {
      'inputs': {
        'bnInput': {
          'data': [
            -41.3125, 64.0625,   -63.375,        -46.78125, 83,
            -80.0625, -62.15625, -0.10009765625, -40.90625, 56.96875,
            37.375,   57.03125,  82.0625,        -86.125,   76.875,
            97.0625,  -21.34375, -96.9375,       -9.359375, 80.1875,
            -85.375,  62.34375,  -68.5,          -12.109375
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'},
          'constant': true
        },
        'bnMean': {
          'data': [-7.8125, -95.625, 38.15625, -55.9375, -87.875, -41.625],
          'descriptor': {shape: [6], dataType: 'float16'},
          'constant': true
        },
        'bnVariance': {
          'data': [60.3125, 26.4375, 53.28125, 40.15625, 59.40625, 36],
          'descriptor': {shape: [6], dataType: 'float16'},
          'constant': true
        }
      },
      'operators': [{
        'name': 'batchNormalization',
        'arguments': [
          {'input': 'bnInput'}, {'mean': 'bnMean'}, {'variance': 'bnVariance'}
        ],
        'outputs': 'bnOutput'
      }],
      'expectedOutputs': {
        'bnOutput': {
          'data': [
            -4.3125,    31.0625,     -13.90625,   1.4453125,   22.171875,
            -6.40625,   -6.99609375, 18.578125,   -10.828125,  17.8125,
            16.25,      16.4375,     11.5703125,  1.84765625,  5.3046875,
            24.140625,  8.6328125,   -9.21875,    -0.19921875, 34.1875,
            -16.921875, 18.671875,   2.513671875, 4.91796875
          ],
          'descriptor': {shape: [4, 6], dataType: 'float16'}
        }
      }
    }
  }
];

webnn_conformance_test(
    batchNormTests, buildAndExecuteGraph, getPrecisionTolerance);
