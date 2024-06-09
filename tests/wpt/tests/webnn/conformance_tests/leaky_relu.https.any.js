// META: title=test WebNN API leakyRelu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-leakyrelu

runWebNNConformanceTests('leakyRelu', buildOperationWithSingleInput);
