// META: title=test WebNN API matmul operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-matmul

testWebNNOperation('matmul', buildOperationWithTwoInputs, 'gpu');