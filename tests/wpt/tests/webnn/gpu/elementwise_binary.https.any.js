// META: title=test WebNN API element-wise binary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-binary

testWebNNOperation(['add', 'sub', 'mul', 'div', 'max', 'min', 'pow'], buildOperationWithTwoInputs, 'gpu');