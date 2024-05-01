// META: title=validation tests for WebNN API elu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('elu');

validateUnaryOperation(
    'elu', floatingPointTypes, /*alsoBuildActivation=*/ true);

promise_test(async t => {
  const options = {alpha: 1.0};
  const input =
      builder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});
  const output = builder.elu(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[elu] Test building an operator with options');

promise_test(async t => {
  const options = {alpha: 1.5};
  builder.elu(options);
}, '[elu] Test building an activation with options');

promise_test(async t => {
  const options = {alpha: -1.0};
  const input =
      builder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});
  assert_throws_js(TypeError, () => builder.elu(input, options));
}, '[elu] Throw if options.alpha <= 0 when building an operator');

promise_test(async t => {
  const options = {alpha: NaN};
  const input = builder.input('input', {dataType: 'float16', dimensions: []});
  assert_throws_js(TypeError, () => builder.elu(input, options));
}, '[elu] Throw if options.alpha is NaN when building an operator');

promise_test(async t => {
  const options = {alpha: 0};
  assert_throws_js(TypeError, () => builder.elu(options));
}, '[elu] Throw if options.alpha <= 0 when building an activation');

promise_test(async t => {
  const options = {alpha: Infinity};
  assert_throws_js(TypeError, () => builder.elu(options));
}, '[elu] Throw if options.alpha is Infinity when building an activation');
