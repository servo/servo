// META: title=test WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-argminmax

runWebNNConformanceTests(['argMin', 'argMax'], buildOperationWithSingleInput);
