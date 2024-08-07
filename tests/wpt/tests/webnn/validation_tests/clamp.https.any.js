// META: title=validation tests for WebNN API clamp operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('clamp');

validateUnaryOperation('clamp', allWebNNOperandDataTypes);

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 1.0, maxValue: 3.0};
  if (!context.opSupportLimits().input.dataTypes.includes('uint32')) {
    assert_throws_js(
        TypeError,
        () => builder.input(
            'input', {dataType: 'uint32', dimensions: [1, 2, 3]}));
    return;
  }
  const input =
      builder.input('input', {dataType: 'uint32', dimensions: [1, 2, 3]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'uint32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[clamp] Build with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 0, maxValue: 0};
  if (!context.opSupportLimits().input.dataTypes.includes('int32')) {
    assert_throws_js(
        TypeError,
        () => builder.input(
            'input', {dataType: 'int32', dimensions: [1, 2, 3, 4]}));
    return;
  }
  const input =
      builder.input('input', {dataType: 'int32', dimensions: [1, 2, 3, 4]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'int32');
  assert_array_equals(output.shape(), [1, 2, 3, 4]);
}, '[clamp] Build with options.minValue == options.maxValue');

const label = '123_clamp';
promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    minValue: 3.0,
    maxValue: 1.0,
    label: label,
  };
  if (!context.opSupportLimits().input.dataTypes.includes('uint8')) {
    assert_throws_js(
        TypeError,
        () =>
            builder.input('input', {dataType: 'uint8', dimensions: [1, 2, 3]}));
    return;
  }
  const input =
      builder.input('input', {dataType: 'uint8', dimensions: [1, 2, 3]});
  try {
    builder.clamp(input, options);
  } catch (e) {
    assert_equals(e.name, 'TypeError');
    const error_message = e.message;
    const regrexp = new RegExp('\\[' + label + '\\]');
    assert_not_equals(error_message.match(regrexp), null);
  }
}, '[clamp] Throw if options.minValue > options.maxValue');

// To be removed once infinite `minValue` is allowed. Tracked in
// https://github.com/webmachinelearning/webnn/pull/647.
promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    minValue: -Infinity,
    label: label,
  };
  const input = builder.input('input', {dataType: 'float16', dimensions: []});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue is -Infinity');
