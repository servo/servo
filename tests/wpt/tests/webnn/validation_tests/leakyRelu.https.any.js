// META: title=validation tests for WebNN API leakyRelu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('leakyRelu');

validateUnaryOperation(
    'leakyRelu', floatingPointTypes, /*alsoBuildActivation=*/ true);

promise_test(async t => {
  const options = {alpha: 0.02};
  const input =
      builder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});
  const output = builder.leakyRelu(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[leakyRelu] Test building an operator with options');

promise_test(async t => {
  const options = {alpha: 0.03};
  builder.leakyRelu(options);
}, '[leakyRelu] Test building an activation with options');

promise_test(async t => {
  const options = {alpha: Infinity};
  const input = builder.input('input', {dataType: 'float16', dimensions: []});
  assert_throws_js(TypeError, () => builder.leakyRelu(input, options));
}, '[leakyRelu] Throw if options.alpha is Infinity when building an operator');

promise_test(async t => {
  const options = {alpha: -NaN};
  assert_throws_js(TypeError, () => builder.leakyRelu(options));
}, '[leakyRelu] Throw if options.alpha is -NaN when building an activation');
