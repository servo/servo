// META: title=test WebNN API clamp operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-clamp

testWebNNOperation('clamp', buildOperationWithSingleInput, 'gpu');