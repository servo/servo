// META: title=test WebNN API prelu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-prelu

testWebNNOperation('prelu', buildOperationWithTwoInputs, 'gpu');