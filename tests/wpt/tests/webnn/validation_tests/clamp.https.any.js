// META: title=validation tests for WebNN API clamp operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('clamp');

validateUnaryOperation('clamp', allWebNNOperandDataTypes);

promise_test(async t => {
  const options = {minValue: 1.0, maxValue: 3.0};
  const input =
      builder.input('input', {dataType: 'uint32', dimensions: [1, 2, 3]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'uint32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[clamp] Test building an operator with options');

promise_test(async t => {
  const options = {minValue: 0, maxValue: 0};
  const input =
      builder.input('input', {dataType: 'int32', dimensions: [1, 2, 3, 4]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'int32');
  assert_array_equals(output.shape(), [1, 2, 3, 4]);
}, '[clamp] Test building an operator with options.minValue == options.maxValue');

promise_test(async t => {
  const options = {minValue: 3.0, maxValue: 1.0};
  const input =
      builder.input('input', {dataType: 'uint8', dimensions: [1, 2, 3]});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue > options.maxValue when building an operator');

// To be removed once infinite `minValue` is allowed. Tracked in
// https://github.com/webmachinelearning/webnn/pull/647.
promise_test(async t => {
  const options = {minValue: -Infinity};
  const input = builder.input('input', {dataType: 'float16', dimensions: []});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue is -Infinity when building an operator');
