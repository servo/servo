// META: title=validation tests for WebNN API cast operation
// META: global=window
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

promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    const input = builder.input('input', {
        dataType: 'int8',
        shape: [context.opSupportLimits().maxTensorByteLength / 2]});
    assert_throws_js(
        TypeError, () => builder.cast(input, 'int64'));
  }, '[cast] throw if the output tensor byte length exceeds limit');
