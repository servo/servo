// META: title=validation tests for WebNN API pad operation
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const label = 'pad_xxx';
const regrexp = new RegExp('\\[' + label + '\\]');

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [2, 2]});

  const beginningPadding = [1, 1];
  const endingPadding = [1, 1];
  assert_throws_js(
      TypeError,
      () =>
          builder.pad(inputFromOtherBuilder, beginningPadding, endingPadding));
}, '[pad] throw if input is from another builder');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);

  const input = builder.input('input', {
      dataType: 'float32',
      shape: [1, context.opSupportLimits().maxTensorByteLength / 4]});

  const options = {};
  options.value = 0;
  options.mode = 'constant';
  options.label = label;
  const beginningPadding = [1, 2];
  const endingPadding = [1, 2];
  assert_throws_with_label(
      () => builder.pad(input, beginningPadding, endingPadding, options), regrexp);
}, '[pad] throw if the output tensor byte length exceeds limit');

const tests = [
  {
    name:
        '[pad] Test with default options, beginningPadding=[1, 2] and endingPadding=[1, 2].',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [1, 2],
    endingPadding: [1, 2],
    options: {
      mode: 'constant',
      value: 0,
    },
    output: {dataType: 'float32', shape: [4, 7]}
  },
  {
    name:
        '[pad] Test pad for scalar input with empty beginningPadding and endingPadding.',
    input: {dataType: 'float32', shape: []},
    beginningPadding: [],
    endingPadding: [],
    output: {dataType: 'float32', shape: []}
  },
  {
    name:
        '[pad] Throw if the length of beginningPadding is not equal to the input rank.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [1],
    endingPadding: [1, 2],
    options: {
      mode: 'edge',
      value: 0,
      label: label,
    },
  },
  {
    name:
        '[pad] Throw if the length of endingPadding is not equal to the input rank.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [1, 0],
    endingPadding: [1, 2, 0],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
  {
    name:
        '[pad] Throw if beginningPadding[index] is equal to inputShape[index] on reflection mode.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [2, 0],
    endingPadding: [1, 2],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
  {
    name:
        '[pad] Throw if beginningPadding[index] is greater than inputShape[index] on reflection mode.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [3, 0],
    endingPadding: [1, 2],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
  {
    name:
        '[pad] Throw if endingPadding[index] is equal to inputShape[index] on reflection mode.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [1, 0],
    endingPadding: [1, 3],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
  {
    name:
        '[pad] Throw if endingPadding[index] is greater than inputShape[index] on reflection mode.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [1, 0],
    endingPadding: [1, 4],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
  {
    name: '[pad] Throw if the padding of one dimension is too large.',
    input: {dataType: 'float32', shape: [2, 3]},
    beginningPadding: [2294967295, 0],
    endingPadding: [3294967295, 2],
    options: {
      mode: 'reflection',
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      if (test.output) {
        const output = builder.pad(
            input, test.beginningPadding, test.endingPadding, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        assert_throws_with_label(
            () => builder.pad(
                input, test.beginningPadding, test.endingPadding, test.options),
            regrexp);
      }
    }, test.name));
