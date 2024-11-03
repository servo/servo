// META: title=validation tests for WebNN API where operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kExampleConditionDescriptor = {
  dataType: 'uint8',
  shape: [2, 4]
};
const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: [2, 4]
};

const tests = [
  {
    name: '[where] Throw if the condition data type is not uint8.',
    condition: {dataType: 'float32', shape: [2, 4]},
    trueValue: {dataType: 'float32', shape: [2, 4]},
    falseValue: {dataType: 'float32', shape: [2, 4]},
  },
  {
    name:
        '[where] Throw if the data types of trueValue and falseValue do not match',
    condition: {dataType: 'uint8', shape: [2, 4]},
    trueValue: {dataType: 'float16', shape: [2, 4]},
    falseValue: {dataType: 'float32', shape: [2, 4]},
  },
  {
    name:
        '[where] Throw if the shapes of trueValue and falseValue are not broadcastable',
    condition: {dataType: 'uint8', shape: [2, 4]},
    trueValue: {dataType: 'float32', shape: [2, 4]},
    falseValue: {dataType: 'float32', shape: [2, 3]},
  },
  {
    name: '[where] Throw if the condition shape is not broadcastable',
    condition: {dataType: 'uint8', shape: [2, 4]},
    trueValue: {dataType: 'float32', shape: [2, 3]},
    falseValue: {dataType: 'float32', shape: [2, 1]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', shape: [2, 1]},
    trueValue: {dataType: 'float32', shape: [2, 4]},
    falseValue: {dataType: 'float32', shape: [2, 4]},
    output: {dataType: 'float32', shape: [2, 4]},
  },
  {
    name:
        '[where] Test building where with 2-D condition, 2-D trueValue and 3-D falseValue using broadcast',
    condition: {dataType: 'uint8', shape: [1, 4]},
    trueValue: {dataType: 'float16', shape: [3, 4]},
    falseValue: {dataType: 'float16', shape: [2, 3, 4]},
    output: {dataType: 'float16', shape: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 3-D condition, 3-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', shape: [2, 1, 4]},
    trueValue: {dataType: 'int32', shape: [2, 3, 4]},
    falseValue: {dataType: 'int32', shape: [1, 4]},
    output: {dataType: 'int32', shape: [2, 3, 4]},
  },
  {
    name:
        '[where] Test building where with 4-D condition, 3-D trueValue and 2-D falseValue using broadcast',
    condition: {dataType: 'uint8', shape: [2, 3, 4, 5]},
    trueValue: {dataType: 'uint32', shape: [3, 4, 5]},
    falseValue: {dataType: 'uint32', shape: [4, 5]},
    output: {dataType: 'uint32', shape: [2, 3, 4, 5]},
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      for (let operand of [test.condition, test.trueValue, test.falseValue]) {
        if (!context.opSupportLimits().input.dataTypes.includes(
                operand.dataType)) {
          assert_throws_js(TypeError, () => builder.input('input', operand));
          return;
        }
      }

      const condition = builder.input('condition', test.condition);
      const trueValue = builder.input('trueValue', test.trueValue);
      const falseValue = builder.input('falseValue', test.falseValue);
      if (test.output &&
          context.opSupportLimits().where.condition.dataTypes.includes(
              test.condition.dataType) &&
          context.opSupportLimits().where.trueValue.dataTypes.includes(
              test.trueValue.dataType) &&
          context.opSupportLimits().where.falseValue.dataTypes.includes(
              test.falseValue.dataType)) {
        const output = builder.where(condition, trueValue, falseValue);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const label = 'where_123';
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.where(condition, trueValue, falseValue, options),
            regrexp);
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
