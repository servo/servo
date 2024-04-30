// META: title=validation tests for WebNN API linear operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('linear');

validateUnaryOperation(
    'linear', floatingPointTypes, /*alsoBuildActivation=*/ true);

promise_test(async t => {
  const options = {alpha: 1.5, beta: 0.3};
  const input =
      builder.input('input', {dataType: 'float32', dimensions: [1, 2, 3]});
  const output = builder.linear(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[linear] Test building an operator with options');

promise_test(async t => {
  const options = {beta: 1.5};
  builder.linear(options);
}, '[linear] Test building an activation with options');

promise_test(async t => {
  const options = {beta: -Infinity};
  const input = builder.input('input', {dataType: 'float16', dimensions: []});
  assert_throws_js(TypeError, () => builder.linear(input, options));
}, '[linear] Throw if options.beta is -Infinity when building an operator');

promise_test(async t => {
  const options = {alpha: NaN};
  assert_throws_js(TypeError, () => builder.linear(options));
}, '[linear] Throw if options.alpha is NaN when building an activation');
