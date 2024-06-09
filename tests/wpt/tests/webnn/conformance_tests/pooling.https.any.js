// META: title=test WebNN API pooling operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-pool2d

runWebNNConformanceTests(
    ['averagePool2d', 'l2Pool2d', 'maxPool2d'], buildOperationWithSingleInput);
