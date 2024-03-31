// META: title=validation tests for WebNN API split operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [4, 4]});

  const splits = 2;
  assert_throws_js(
      TypeError, () => builder.split(inputFromOtherBuilder, splits));
}, '[split] throw if input is from another builder');
