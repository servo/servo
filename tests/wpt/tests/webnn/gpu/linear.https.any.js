// META: title=test WebNN API linear operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-linear

testWebNNOperation('linear', buildOperationWithSingleInput, 'gpu');