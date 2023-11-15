// META: title=test WebNN API reduction  operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-reduce

testWebNNOperation(
  [
    'reduceL1',
    'reduceL2',
    'reduceLogSum',
    'reduceLogSumExp',
    'reduceMax',
    'reduceMean',
    'reduceMin',
    'reduceProduct',
    'reduceSum',
    'reduceSumSquare',
  ],
  buildOperationWithSingleInput, 'gpu'
);