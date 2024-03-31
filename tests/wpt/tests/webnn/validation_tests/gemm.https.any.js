// META: title=validation tests for WebNN API gemm operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2]
};

validateTwoInputsFromMultipleBuilders('gemm');

multi_builder_test(async (t, builder, otherBuilder) => {
  const cFromOtherBuilder = otherBuilder.input('c', kExampleInputDescriptor);
  const options = {c: cFromOtherBuilder};

  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.gemm(a, b, options));
}, '[gemm] throw if c option is from another builder');
