// META: title=test WebNN API transpose operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-transpose

testWebNNOperation('transpose', buildOperationWithSingleInput, 'gpu');