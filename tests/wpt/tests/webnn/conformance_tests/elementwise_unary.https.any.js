// META: title=test WebNN API element-wise unary operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-unary

runWebNNConformanceTests(
    [
      'abs', 'ceil', 'cos', 'erf', 'exp', 'floor', 'identity', 'log', 'neg',
      'reciprocal', 'sin', 'sqrt', 'tan'
    ],
    buildOperationWithSingleInput);
