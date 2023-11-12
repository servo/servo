// META: title=test WebNN API conv2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-conv2d

testWebNNOperation('conv2d', buildConv2d, 'gpu');