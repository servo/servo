// META: title=validation tests for WebNN API pad operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const beginningPadding = [1, 1];
  const endingPadding = [1, 1];
  assert_throws_js(
      TypeError,
      () =>
          builder.pad(inputFromOtherBuilder, beginningPadding, endingPadding));
}, '[pad] throw if input is from another builder');
