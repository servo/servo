// META: title=test WebNN API resample2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-resample2d-method

testWebNNOperation('resample2d', buildOperationWithSingleInput);