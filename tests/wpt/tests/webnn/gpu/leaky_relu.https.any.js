// META: title=test WebNN API leakyRelu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-leakyrelu

testWebNNOperation('leakyRelu', buildOperationWithSingleInput, 'gpu');