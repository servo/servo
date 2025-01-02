// META: title=ensure an MLOperand cannot be created with an invalid rank
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

promise_test(async t => {
  const builder = new MLGraphBuilder(context);

  const shapeWithLargeRank = Array(10).fill(2);
  assert_throws_js(
      TypeError,
      () =>
          builder.input('a', {dataType: 'float32', shape: shapeWithLargeRank}));
}, 'Throw if rank is too large');
