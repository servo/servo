// META: title=validation tests for WebNN API elu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('elu');

const label = 'elu_xxx';
const regrexp = new RegExp('\\[' + label + '\\]');

validateSingleInputOperation('elu', label);

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: 1.0};
  const input = builder.input('input', {dataType: 'float32', shape: [1, 2, 3]});
  const output = builder.elu(input, options);
  assert_equals(output.dataType(), 'float32');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[elu] Build with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    alpha: -1.0,
    label: label,
  };
  const input = builder.input('input', {dataType: 'float32', shape: [1, 2, 3]});
  assert_throws_with_label(() => builder.elu(input, options), regrexp);
}, '[elu] Throw if options.alpha < 0');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {
    alpha: 0,
    label: label,
  };
  const input = builder.input('input', {dataType: 'float32', shape: [1]});
  assert_throws_with_label(() => builder.elu(input, options), regrexp);
}, '[elu] Throw if options.alpha == 0');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: NaN};
  const input = builder.input('input', {dataType: 'float16', shape: []});
  assert_throws_js(TypeError, () => builder.elu(input, options));
}, '[elu] Throw if options.alpha is NaN');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: Infinity};
  const input = builder.input('input', {dataType: 'float32', shape: [1]});
  assert_throws_js(TypeError, () => builder.elu(input, options));
}, '[elu] Throw if options.alpha is Infinity');
