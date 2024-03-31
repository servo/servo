// META: title=validation tests for WebNN API conv2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

// Example input in NCHW layout.
const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [1, 1, 5, 5]
};
// Example filter in OIHW layout.
const kExampleFilterDescriptor = {
  dataType: 'float32',
  dimensions: [1, 1, 3, 3]
};
const kExampleBiasDescriptor = {
  dataType: 'float32',
  dimensions: [/* output channels */ 1]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(
      TypeError, () => builder.conv2d(inputFromOtherBuilder, filter));
}, '[conv2d] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const filterFromOtherBuilder =
      otherBuilder.input('filter', kExampleFilterDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.conv2d(input, filterFromOtherBuilder));
}, '[conv2d] throw if filter is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {inputLayout: 'nchw', bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(TypeError, () => builder.conv2d(input, filter, options));
}, '[conv2d] throw if bias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const activationFromOtherBuilder = otherBuilder.clamp();
  const options = {activation: activationFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(TypeError, () => builder.conv2d(input, filter, options));
}, '[conv2d] throw if activation option is from another builder');
