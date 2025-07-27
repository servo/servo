// META: title=validation tests for WebNN API clamp operation
// META: global=window
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
  assert_equals(output.dataType, 'float32');
  assert_array_equals(output.shape, [1, 2, 3]);
}, '[clamp] Build with options');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 0, maxValue: 0};
  const input =
      builder.input('input', {dataType: 'float32', shape: [1, 2, 3, 4]});
  const output = builder.clamp(input, options);
  assert_equals(output.dataType, 'float32');
  assert_array_equals(output.shape, [1, 2, 3, 4]);
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

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 3n, maxValue: 1n};
  if (!context.opSupportLimits().input.dataTypes.includes('int64')) {
    assert_throws_js(
        TypeError,
        () => builder.input('input', {dataType: 'int64', shape: [1, 2, 3]}));
    return;
  }
  const input = builder.input('input', {dataType: 'int64', shape: [1, 2, 3]});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue BigInt > options.maxValue BigInt');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = {minValue: 3, maxValue: 1n};
  if (!context.opSupportLimits().input.dataTypes.includes('int64')) {
    assert_throws_js(
        TypeError,
        () => builder.input('input', {dataType: 'int64', shape: [1, 2, 3]}));
    return;
  }
  const input = builder.input('input', {dataType: 'int64', shape: [1, 2, 3]});
  assert_throws_js(TypeError, () => builder.clamp(input, options));
}, '[clamp] Throw if options.minValue > options.maxValue BigInt');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const options = { minValue: 1n, maxValue: 3n };
  const data_types = ['float32', 'float16', 'int32', 'uint32', 'int8', 'uint8', 'int4', 'uint4'];
  for (const data_type of data_types) {
    if (!context.opSupportLimits().input.dataTypes.includes(data_type)) {
      assert_throws_js(
        TypeError,
        () => builder.input('input', { dataType: data_type, shape: [1, 2, 3] }));
      return;
    }
    const input = builder.input('input', { dataType: data_type, shape: [1, 2, 3] });
    assert_throws_js(TypeError, () => builder.clamp(input, options));
  }
}, '[clamp] Throw if BigInt is used for data types other than int64 and uint64');
