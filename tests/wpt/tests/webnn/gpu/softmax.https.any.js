// META: title=test WebNN API softmax operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-softmax

testWebNNOperation('softmax', buildOperationWithSingleInput, 'gpu');