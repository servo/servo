// META: title=test WebNN API convTranspose2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-convtranspose2d

testWebNNOperation('convTranspose2d', buildConvTranspose2d, 'gpu');