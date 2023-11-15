// META: title=test WebNN API sigmoid operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-sigmoid

testWebNNOperation('sigmoid', buildOperationWithSingleInput, 'gpu');