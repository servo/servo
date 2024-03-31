// META: title=validation tests for WebNN API reshape operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});

  const newShape = [3, 2, 1];
  assert_throws_js(
      TypeError, () => builder.reshape(inputFromOtherBuilder, newShape));
}, '[reshape] throw if input is from another builder');
