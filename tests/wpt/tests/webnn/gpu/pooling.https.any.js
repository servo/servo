// META: title=test WebNN API pooling operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-pool2d

testWebNNOperation(['averagePool2d', 'maxPool2d'], buildOperationWithSingleInput, 'gpu');