// META: title=test WebNN API softplus operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-softplus

testWebNNOperation('softplus', buildOperationWithSingleInput, 'gpu');