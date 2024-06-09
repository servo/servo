// META: title=test parallel WebNN API compute operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlcontext-compute

if (navigator.ml) {
  testParallelCompute();
} else {
  // Show indication to users why the test failed
  test(
      () => assert_not_equals(
          navigator.ml, undefined, 'ml property is defined on navigator'));
}
