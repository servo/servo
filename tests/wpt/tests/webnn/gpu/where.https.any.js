// META: title=test WebNN API where operation
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-where

testWebNNOperation('where', buildWhere, 'gpu');