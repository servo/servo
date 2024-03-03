// META: title=test WebNN API constant
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-constant-range

testWebNNOperation('constant', buildConstantRange);