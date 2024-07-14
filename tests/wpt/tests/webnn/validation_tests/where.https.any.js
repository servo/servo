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
    name: '[where] Throw if the condition data type is not uint8.',
    condition: {dataType: 'float32', dimensions: [2, 4]},
    trueValue: {dataType: 'float32', dimensions: [2, 4]},
    falseValue: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Throw if the data types of trueValue and falseValue do not match',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    trueValue: {dataType: 'float16', dimensions: [2, 4]},
    falseValue: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Throw if the shapes of trueValue and falseValue are not broadcastable',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    trueValue: {dataType: 'float32', dimensions: [2, 4]},
    falseValue: {dataType: 'float32', dimensions: [2, 3]},
  },
  {
    name: '[where] Throw if the condition shape is not broadcastable',
    condition: {dataType: 'uint8', dimensions: [2, 4]},
    trueValue: {dataType: 'float32', dimensions: [2, 3]},
    falseValue: {dataType: 'float32', dimensions: [2, 1]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 1]},
    trueValue: {dataType: 'float32', dimensions: [2, 4]},
    falseValue: {dataType: 'float32', dimensions: [2, 4]},
    output: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D trueValue and 3-D falseValue using broadcast',
    condition: {dataType: 'uint8', dimensions: [1, 4]},
    trueValue: {dataType: 'float16', dimensions: [3, 4]},
    falseValue: {dataType: 'float16', dimensions: [2, 3, 4]},
    output: {dataType: 'float16', dimensions: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 3-D condition, 3-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 1, 4]},
    trueValue: {dataType: 'int32', dimensions: [2, 3, 4]},
    falseValue: {dataType: 'int32', dimensions: [1, 4]},
    output: {dataType: 'int32', dimensions: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 4-D condition, 3-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', dimensions: [2, 3, 4, 5]},
    trueValue: {dataType: 'uint32', dimensions: [3, 4, 5]},
    falseValue: {dataType: 'uint32', dimensions: [4, 5]},
    output: {dataType: 'uint32', dimensions: [2, 3, 4, 5]},
  }
];

tests.forEach(
    test => promise_test(async t => {
      for (let operand of [test.condition, test.trueValue, test.falseValue]) {
        if (!context.opSupportLimits().input.dataTypes.includes(
                operand.dataType)) {
          assert_throws_js(TypeError, () => builder.input('input', {
            dataType: operand.dataType,
            dimensions: operand.dimensions
          }));
          return;
        }
      }

      const condition = builder.input('condition', {
        dataType: test.condition.dataType,
        dimensions: test.condition.dimensions
      });
      const trueValue = builder.input('trueValue', {
        dataType: test.trueValue.dataType,
        dimensions: test.trueValue.dimensions
      });
      const falseValue = builder.input('falseValue', {
        dataType: test.falseValue.dataType,
        dimensions: test.falseValue.dimensions
      });
      if (test.output &&
          context.opSupportLimits().where.condition.dataTypes.includes(
              test.condition.dataType) &&
          context.opSupportLimits().where.trueValue.dataTypes.includes(
              test.trueValue.dataType) &&
          context.opSupportLimits().where.falseValue.dataTypes.includes(
              test.falseValue.dataType)) {
        const output = builder.where(condition, trueValue, falseValue);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.where(condition, trueValue, falseValue));
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const conditionFromOtherBuilder =
      otherBuilder.input('condition', kExampleConditionDescriptor);

  const trueValue = builder.input('trueValue', kExampleInputDescriptor);
  const falseValue = builder.input('falseValue', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.where(conditionFromOtherBuilder, trueValue, falseValue));
}, '[where] throw if condition is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const trueValueFromOtherBuilder =
      otherBuilder.input('trueValue', kExampleInputDescriptor);

  const condition = builder.input('condition', kExampleConditionDescriptor);
  const falseValue = builder.input('falseValue', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.where(condition, trueValueFromOtherBuilder, falseValue));
}, '[where] throw if trueValue is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const falseValueFromOtherBuilder =
      otherBuilder.input('falseValue', kExampleInputDescriptor);

  const condition = builder.input('condition', kExampleConditionDescriptor);
  const trueValue = builder.input('trueValue', kExampleInputDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.where(condition, trueValue, falseValueFromOtherBuilder));
}, '[where] throw if falseValue is from another builder');
