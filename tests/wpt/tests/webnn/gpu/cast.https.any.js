// META: title=test WebNN API cast operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-cast

testWebNNOperation('cast', buildCast, 'gpu');