// META: title=validation tests for WebNN API layerNormalization operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2]
};

validateOptionsAxes('layerNormalization', 4);

validateInputFromAnotherBuilder('layerNormalization');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExampleInputDescriptor);
  const options = {scale: scaleFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.layerNormalization(input, options));
}, '[layerNormalization] throw if scale option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleInputDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.layerNormalization(input, options));
}, '[layerNormalization] throw if bias option is from another builder');
