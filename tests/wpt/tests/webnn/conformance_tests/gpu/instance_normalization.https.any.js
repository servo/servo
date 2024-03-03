// META: title=test WebNN API instanceNormalization operation
// META: global=window,dedicatedworker
// META: script=../../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-instancenorm

testWebNNOperation('instanceNormalization', buildLayerNorm, 'gpu');