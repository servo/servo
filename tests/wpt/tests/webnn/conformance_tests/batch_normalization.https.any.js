// META: title=test WebNN API batchNormalization operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-batchnorm

runWebNNConformanceTests('batchNormalization', buildBatchNorm);
