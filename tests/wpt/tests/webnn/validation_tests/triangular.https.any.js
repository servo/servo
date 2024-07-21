// META: title=validation tests for WebNN API triangular operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

promise_test(async t => {
  // The input tensor which is at least 2-D.
  for (let dimensions of allWebNNDimensionsArray.slice(0, 2)) {
    for (let dataType of allWebNNOperandDataTypes) {
      const input = builder.input(`input${++inputIndex}`, {dataType, dimensions});
      assert_throws_js(TypeError, () => builder.triangular(input));
    }
  }
}, "[triangular] TypeError is expected if input's rank is less than 2");

validateInputFromAnotherBuilder('triangular');
