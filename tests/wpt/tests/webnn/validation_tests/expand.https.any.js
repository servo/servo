// META: title=validation tests for WebNN API expand operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 1, 2]});

  const newShape = [2, 2, 2];
  assert_throws_js(
      TypeError, () => builder.expand(inputFromOtherBuilder, newShape));
}, '[expand] throw if input is from another builder');
