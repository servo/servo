// META: title=validation tests for WebNN API instanceNormalization operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2, 2, 2]
};
// 1D tensor descriptor which may be used for `scale`, or `bias` inputs.
const kExample1DTensorDescriptor = {
  dataType: 'float32',
  dimensions: [2]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  assert_throws_js(
      TypeError, () => builder.instanceNormalization(inputFromOtherBuilder));
}, '[instanceNormalization] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExample1DTensorDescriptor);
  const options = {scale: scaleFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.instanceNormalization(input, options));
}, '[instanceNormalization] throw if scale option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExample1DTensorDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.instanceNormalization(input, options));
}, '[instanceNormalization] throw if bias option is from another builder');
