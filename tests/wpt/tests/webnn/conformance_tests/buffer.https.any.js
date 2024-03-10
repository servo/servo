// META: title=test WebNN API buffer operations
// META: global=window,dedicatedworker
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlbuffer

testCreateWebNNBuffer("create", 4);

testDestroyWebNNBuffer("destroyTwice");