// META: title=test WebNN API reshape operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-reshape

testWebNNOperation('reshape', buildReshape, 'gpu');

