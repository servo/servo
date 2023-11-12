// META: title=test WebNN API hardSigmoid operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-hard-sigmoid

testWebNNOperation('hardSigmoid', buildOperationWithSingleInput, 'gpu');