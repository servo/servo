// META: title=validation tests for WebNN API hardSigmoid operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('hardSigmoid');

const label = 'hard_sigmoid';

validateSingleInputOperation('hardSigmoid', label);

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: 0.5, beta: 1.0};
  const input = builder.input('input', {dataType: 'float16', shape: [1, 2, 3]});
  const output = builder.hardSigmoid(input, options);
  assert_equals(output.dataType(), 'float16');
  assert_array_equals(output.shape(), [1, 2, 3]);
}, '[hardSigmoid] Test building with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {beta: NaN};
  const input = builder.input('input', {dataType: 'float32', shape: []});
  assert_throws_js(TypeError, () => builder.hardSigmoid(input, options));
}, '[hardSigmoid] Throw if options.beta is NaN');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {alpha: Infinity};
  const input = builder.input('input', {dataType: 'float32', shape: [1]});
  assert_throws_js(TypeError, () => builder.hardSigmoid(input, options));
}, '[hardSigmoid] Throw if options.alpha is Infinity');
