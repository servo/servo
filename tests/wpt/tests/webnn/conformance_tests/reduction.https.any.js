// META: title=test WebNN API reduction  operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-reduce

runWebNNConformanceTests(
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
    buildOperationWithSingleInput);
