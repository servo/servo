// META: title=validation tests for WebNN API cast operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'int32', dimensions: [2, 2]});

  assert_throws_js(
      TypeError, () => builder.cast(inputFromOtherBuilder, 'int64'));
}, '[cast] throw if input is from another builder');
