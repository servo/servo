// META: title=validation tests for WebNN API clamp operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('clamp');

const label = '123_clamp';

validateSingleInputOperation('clamp', label);

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 1.0, maxValue: 3.0};
  const input = builder.input('input', {dataType: 'float32', shape: [1, 2, 3]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[clamp] Build with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 0, maxValue: 0};
  const input =
      builder.input('input', {dataType: 'float32', shape: [1, 2, 3, 4]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3, 4]);
}, '[clamp] Build with options.minValue == options.maxValue');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    minValue: 3.0,
    maxValue: 1.0,
    label: label,
  };
  const input = builder.input('input', {dataType: 'float32', shape: [1, 2, 3]});
  const regrexp = new RegExp('\\[' + label + '\\]');
  assert_throws_with_label(() => builder.clamp(input, options), regrexp);
}, '[clamp] Throw if options.minValue > options.maxValue');

// To be removed once infinite `minValue` is allowed. Tracked in
// https://github.com/webmachinelearning/webnn/pull/647.
promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    minValue: -Infinity,
    label: label,
  };
  const input = builder.input('input', {dataType: 'float32', shape: []});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue is -Infinity');
