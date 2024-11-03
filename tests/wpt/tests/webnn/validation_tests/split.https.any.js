// META: title=validation tests for WebNN API split operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', {dataType: 'float32', shape: [4, 4]});

  const splits = 2;
  assert_throws_js(
      TypeError, () => builder.split(inputFromOtherBuilder, splits));
}, '[split] throw if input is from another builder');

const label = 'xxx-split';
const tests = [
  {
    name: '[split] Test with default options.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: [2],
    outputs: [
      {dataType: 'float32', shape: [2, 6]},
    ]
  },
  {
    name:
        '[split] Test with a sequence of unsigned long splits and with options.axis = 1.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: [1, 2, 3],
    options: {axis: 1},
    outputs: [
      {dataType: 'float32', shape: [2, 1]},
      {dataType: 'float32', shape: [2, 2]},
      {dataType: 'float32', shape: [2, 3]},
    ]
  },
  {
    name: '[split] Throw if splitting a scalar.',
    input: {dataType: 'float32', shape: []},
    splits: [2],
    options: {label}
  },
  {
    name: '[split] Throw if axis is larger than input rank.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: [2],
    options: {
      axis: 2,
      label: label,
    }
  },
  {
    name: '[split] Throw if splits is equal to 0.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: [0],
    options: {
      axis: 0,
      label: label,
    }
  },
  {
    name: '[split] Throw if splits (scalar) is equal to 0.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: 0,
    options: {
      axis: 0,
      label: label,
    },
  },
  {
    name:
        '[split] Throw if the splits can not evenly divide the dimension size of input along options.axis.',
    input: {dataType: 'float32', shape: [2, 5]},
    splits: [2],
    options: {
      axis: 1,
      label: label,
    }
  },
  {
    name:
        '[split] Throw if splits (scalar) can not evenly divide the dimension size of input along options.axis.',
    input: {dataType: 'float32', shape: [2, 5]},
    splits: 2,
    options: {
      axis: 1,
      label: label,
    },
  },
  {
    name:
        '[split] Throw if the sum of splits sizes not equal to the dimension size of input along options.axis.',
    input: {dataType: 'float32', shape: [2, 6]},
    splits: [2, 2, 3],
    options: {
      axis: 1,
      label: label,
    }
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      if (test.outputs) {
        const outputs = builder.split(input, test.splits, test.options);
        assert_equals(outputs.length, test.outputs.length);
        for (let i = 0; i < outputs.length; ++i) {
          assert_equals(outputs[i].dataType, test.outputs[i].dataType);
          assert_array_equals(outputs[i].shape, test.outputs[i].shape);
        }
      } else {
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.split(input, test.splits, test.options), regrexp);
      }
    }, test.name));
