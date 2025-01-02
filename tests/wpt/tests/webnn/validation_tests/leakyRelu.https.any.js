// META: title=validation tests for WebNN API leakyRelu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('leakyRelu');

const label = 'leaky_relu';
validateSingleInputOperation('leakyRelu', label);

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: 0.02};
  const input = builder.input('input', {dataType: 'float32', shape: [1, 2, 3]});
  const output = builder.leakyRelu(input, options);
  assert_equals(output.dataType, 'float32');
  assert_array_equals(output.shape, [1, 2, 3]);
}, '[leakyRelu] Build with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: Infinity};
  const input = builder.input('input', {dataType: 'float16', shape: []});
  assert_throws_js(TypeError, () => builder.leakyRelu(input, options));
}, '[leakyRelu] Throw if options.alpha is Infinity');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: -NaN};
  const input = builder.input('input', {dataType: 'float32', shape: [1]});
  assert_throws_js(TypeError, () => builder.leakyRelu(input, options));
}, '[leakyRelu] Throw if options.alpha is -NaN');
