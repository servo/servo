// META: title=test WebNN API element-wise logical operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-logical

testWebNNOperation(
  [
    'equal',
    'greater',
    'greaterOrEqual',
    'lesser',
    'lesserOrEqual',
  ],
  buildOperationWithTwoInputs, 'gpu'
);
testWebNNOperation('logicalNot', buildOperationWithSingleInput, 'gpu');