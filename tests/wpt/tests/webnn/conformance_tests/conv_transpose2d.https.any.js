// META: title=test WebNN API convTranspose2d operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-convtranspose2d

runWebNNConformanceTests('convTranspose2d', buildConvTranspose2d);
