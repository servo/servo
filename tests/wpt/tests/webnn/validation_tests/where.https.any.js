// META: title=validation tests for WebNN API where operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleConditionDescriptor = {
  dataType: 'uint8',
  dimensions: [2, 4]
};
const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 4]
};

const tests = [
  {
    name:
        '[where] Throw if the condition data type is not uint8.',
    condition: {dataType: 'float32', dimensions: [2, 4]},
    input: {dataType: 'float32', dimensions: [2, 4]},
    other: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Throw if the data types of input and other do not match',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    input: {dataType: 'float16', dimensions: [2, 4]},
    other: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Throw if the shapes of input and other are not broadcastable',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    input: {dataType: 'float32', dimensions: [2, 4]},
    other: {dataType: 'float32', dimensions: [2, 3]},
  },
  {
    name:
        '[where] Throw if the condition shape is not broadcastable',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    input: {dataType: 'float32', dimensions: [2, 3]},
    other: {dataType: 'float32', dimensions: [2, 1]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D input and 2-D other using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 1]},
    input: {dataType: 'float32', dimensions: [2, 4]},
    other: {dataType: 'float32', dimensions: [2, 4]},
    output: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D input and 3-D other using broadcast',
    condition: {dataType: 'uint8', dimensions: [1, 4]},
    input: {dataType: 'float32', dimensions: [3, 4]},
    other: {dataType: 'float32', dimensions: [2, 3, 4]},
    output: {dataType: 'float32', dimensions: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 3-D condition, 3-D input and 2-D other using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 1, 4]},
    input: {dataType: 'float32', dimensions: [2, 3, 4]},
    other: {dataType: 'float32', dimensions: [1, 4]},
    output: {dataType: 'float32', dimensions: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 4-D condition, 3-D input and 2-D other using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 3, 4, 5]},
    input: {dataType: 'float32', dimensions: [3, 4, 5]},
    other: {dataType: 'float32', dimensions: [4, 5]},
    output: {dataType: 'float32', dimensions: [2, 3, 4, 5]},
  }
];

tests.forEach(
    test => promise_test(async t => {
      const condition = builder.input('condition', {
        dataType: test.condition.dataType,
        dimensions: test.condition.dimensions
      });
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const other = builder.input(
          'other',
          {dataType: test.other.dataType, dimensions: test.other.dimensions});
      if (test.output) {
        const output = builder.where(condition, input, other);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.where(condition, input, other));
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const conditionFromOtherBuilder =
      otherBuilder.input('condition', kExampleConditionDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const other = builder.input('other', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.where(conditionFromOtherBuilder, input, other));
}, '[where] throw if condition is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const condition = builder.input('condition', kExampleConditionDescriptor);
  const other = builder.input('other', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.where(condition, inputFromOtherBuilder, other));
}, '[where] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const otherFromOtherBuilder =
      otherBuilder.input('other', kExampleInputDescriptor);

  const condition = builder.input('condition', kExampleConditionDescriptor);
  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.where(condition, input, otherFromOtherBuilder));
}, '[where] throw if other is from another builder');
