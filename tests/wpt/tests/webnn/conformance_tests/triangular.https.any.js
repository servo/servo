// META: title=test WebNN API triangular operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-triangular

testWebNNOperation('triangular', buildOperationWithSingleInput);