// META: title=validation tests for WebNN API hardSigmoid operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('hardSigmoid');

validateUnaryOperation(
    'hardSigmoid', floatingPointTypes, /*alsoBuildActivation=*/ true);

promise_test(async t => {
  const options = {alpha: 0.5, beta: 1.0};
  const input =
      builder.input('input', {dataType: 'float16', dimensions: [1, 2, 3]});
  const output = builder.hardSigmoid(input, options);
  assert_equals(output.dataType(), 'float16');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[hardSigmoid] Test building an operator with options');

promise_test(async t => {
  const options = {alpha: 0.2};
  builder.hardSigmoid(options);
}, '[hardSigmoid] Test building an activation with options');

promise_test(async t => {
  const options = {beta: NaN};
  const input = builder.input('input', {dataType: 'float32', dimensions: []});
  assert_throws_js(TypeError, () => builder.hardSigmoid(input, options));
}, '[hardSigmoid] Throw if options.beta is NaN when building an operator');

promise_test(async t => {
  const options = {alpha: Infinity};
  assert_throws_js(TypeError, () => builder.hardSigmoid(options));
}, '[hardSigmoid] Throw if options.alpha is Infinity when building an activation');
