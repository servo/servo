// META: title=test parallel WebNN API compute operations
// META: global=window,dedicatedworker
// META: script=../../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlcontext-compute

testParallelCompute('gpu');
