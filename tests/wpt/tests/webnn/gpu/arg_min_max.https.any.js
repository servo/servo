// META: title=test WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-argminmax

testWebNNOperation(['argMin', 'argMax'], buildOperationWithSingleInput, 'gpu');