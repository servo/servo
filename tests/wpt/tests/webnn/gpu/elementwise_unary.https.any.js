// META: title=test WebNN API element-wise unary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-unary

testWebNNOperation(
  ['abs', 'ceil', 'cos', 'erf', 'exp', 'floor', 'identity', 'log', 'neg', 'reciprocal', 'sin', 'sqrt', 'tan'],
  buildOperationWithSingleInput, 'gpu'
);