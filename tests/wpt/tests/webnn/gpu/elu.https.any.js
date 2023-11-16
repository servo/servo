// META: title=test WebNN API elu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-elu

testWebNNOperation('elu', buildOperationWithSingleInput, 'gpu');