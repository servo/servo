// META: title=test WebNN API squeeze operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-squeeze

testWebNNOperation('squeeze', buildOperationWithSingleInput);