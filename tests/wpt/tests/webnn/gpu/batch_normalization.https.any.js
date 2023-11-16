// META: title=test WebNN API batchNormalization operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-batchnorm

testWebNNOperation('batchNormalization', buildBatchNorm, 'gpu');