// META: title=test WebNN API tanh operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-hard-swish

testWebNNOperation('hardSwish', buildOperationWithSingleInput, 'gpu');