// META: title=validation tests for WebNN API slice operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', dimensions: [2, 2]});

  const starts = [1, 1];
  const sizes = [1, 1];
  assert_throws_js(
      TypeError, () => builder.slice(inputFromOtherBuilder, starts, sizes));
}, '[slice] throw if input is from another builder');
