// META: title=test WebNN API gemm operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-gemm

testWebNNOperation('gemm', buildGemm, 'gpu');