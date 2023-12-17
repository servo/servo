// META: title=test WebNN API expand operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-expand

// reuse buildReshape method
testWebNNOperation('expand', buildReshape);