// META: title=validation tests for WebNN API cast operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'int32', shape: [2, 2]});

  assert_throws_js(
      TypeError, () => builder.cast(inputFromOtherBuilder, 'int64'));
}, '[cast] throw if input is from another builder');
