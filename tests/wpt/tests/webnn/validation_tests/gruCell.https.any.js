// META: title=validation tests for WebNN API gruCell operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const batchSize = 3, inputSize = 4, hiddenSize = 5;

// Dimensions required of required inputs.
const kValidInputShape = [batchSize, inputSize];
const kValidWeightShape = [3 * hiddenSize, inputSize];
const kValidRecurrentWeightShape = [3 * hiddenSize, hiddenSize];
const kValidHiddenStateShape = [batchSize, hiddenSize];
// Dimensions required of optional inputs.
const kValidBiasShape = [3 * hiddenSize];
const kValidRecurrentBiasShape = [3 * hiddenSize];
// Dimensions required of required output.
const kValidOutputShape = [batchSize, hiddenSize];

// Example descriptors which are valid according to the above dimensions.
const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: kValidInputShape
};
const kExampleWeightDescriptor = {
  dataType: 'float32',
  shape: kValidWeightShape
};
const kExampleRecurrentWeightDescriptor = {
  dataType: 'float32',
  shape: kValidRecurrentWeightShape
};
const kExampleHiddenStateDescriptor = {
  dataType: 'float32',
  shape: kValidHiddenStateShape
};
const kExampleBiasDescriptor = {
  dataType: 'float32',
  shape: kValidBiasShape
};
const kExampleRecurrentBiasDescriptor = {
  dataType: 'float32',
  shape: kValidRecurrentBiasShape
};
const kExampleOutputDescriptor = {
  dataType: 'float32',
  shape: kValidOutputShape
};

const tests = [
  {
    name: '[gruCell] Test with default options',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    output: kExampleOutputDescriptor
  },
  {
    name: '[gruCell] Test with given options',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {
      bias: kExampleBiasDescriptor,
      recurrentBias: kExampleRecurrentBiasDescriptor,
      restAfter: true,
      layout: 'rzn',
      activations: ['sigmoid', 'relu']
    },
    output: kExampleOutputDescriptor
  },
  {
    name: '[gruCell] Throw if hiddenSize equals to zero',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: 0
  },
  {
    name: '[gruCell] Throw if hiddenSize is too large',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: 4294967295,
  },
  {
    name:
        '[gruCell] Throw if the data type of the inputs is not one of the floating point types',
    input: {dataType: 'uint32', shape: kValidInputShape},
    weight: {dataType: 'uint32', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'uint32', shape: kValidRecurrentWeightShape},
    hiddenState: {dataType: 'uint32', shape: kValidHiddenStateShape},
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the rank of input is not 2',
    input: {dataType: 'float32', shape: [batchSize]},
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the input.shape[1] is incorrect',
    input: {dataType: 'float32', shape: [inputSize, inputSize]},
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name:
        '[gruCell] Throw if data type of weight is not one of the floating point types',
    input: kExampleInputDescriptor,
    weight: {dataType: 'int8', shape: [3 * hiddenSize, inputSize]},
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if rank of weight is not 2',
    input: kExampleInputDescriptor,
    weight: {dataType: 'float32', shape: [3 * hiddenSize]},
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if weight.shape[0] is not 3 * hiddenSize',
    input: kExampleInputDescriptor,
    weight: {dataType: 'float32', shape: [4 * hiddenSize, inputSize]},
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name:
        '[gruCell] Throw if data type of recurrentWeight is not one of the floating point types',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: {dataType: 'int32', shape: [3 * hiddenSize, hiddenSize]},
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the rank of recurrentWeight is not 2',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: {dataType: 'float32', shape: [3 * hiddenSize]},
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the recurrentWeight.shape is invalid',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: {dataType: 'float32', shape: [4 * hiddenSize, inputSize]},
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize
  },
  {
    name:
        '[gruCell] Throw if data type of hiddenState is not one of the floating point types',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: {dataType: 'uint32', shape: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the rank of hiddenState is not 2',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: {dataType: 'float32', shape: [hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the hiddenState.shape is invalid',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: {dataType: 'float32', shape: [batchSize, 3 * hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[gruCell] Throw if the size of options.activations is not 2',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {activations: ['sigmoid', 'tanh', 'relu']}
  },
  {
    name:
        '[gruCell] Throw if data type of options.bias is not one of the floating point types',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'uint8', shape: [3 * hiddenSize]}}
  },
  {
    name: '[gruCell] Throw if the rank of options.bias is not 1',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float32', shape: [batchSize, 3 * hiddenSize]}}
  },
  {
    name: '[gruCell] Throw if options.bias.shape[0] is not 3 * hiddenSize',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float32', shape: [2 * hiddenSize]}}
  },
  {
    name:
        '[gruCell] Throw if data type of options.recurrentBias is not one of the floating point types',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {recurrentBias: {dataType: 'int8', shape: [3 * hiddenSize]}}
  },
  {
    name: '[gruCell] Throw if the rank of options.recurrentBias is not 1',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {
      recurrentBias: {dataType: 'float32', shape: [batchSize, 3 * hiddenSize]}
    }
  },
  {
    name:
        '[gruCell] Throw if options.recurrentBias.shape[0] is not 3 * hiddenSize',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    hiddenState: kExampleHiddenStateDescriptor,
    hiddenSize: hiddenSize,
    options: {recurrentBias: {dataType: 'float16', shape: [4 * hiddenSize]}}
  }
];

tests.forEach(
    test =>
        promise_test(async t => {
          const builder = new MLGraphBuilder(context);
          const input = builder.input('input', test.input);
          const weight = builder.input('weight', test.weight);
          const recurrentWeight =
              builder.input('recurrentWeight', test.recurrentWeight);
          const hiddenState = builder.input('hiddenState', test.hiddenState);

          const options = {};
          if (test.options) {
            if (test.options.bias) {
              options.bias = builder.input('bias', test.options.bias);
            }
            if (test.options.recurrentBias) {
              options.recurrentBias =
                  builder.input('recurrentBias', test.options.recurrentBias);
            }
            if (test.options.resetAfter) {
              options.resetAfter = test.options.resetAfter;
            }
            if (test.options.layout) {
              options.layout = test.options.layout;
            }
            if (test.options.activations) {
              options.activations = test.options.activations;
            }
          }

          if (test.output &&
              context.opSupportLimits().gruCell.input.dataTypes.includes(
                  test.input.dataType)) {
            const output = builder.gruCell(
                input, weight, recurrentWeight, hiddenState, test.hiddenSize,
                options);
            assert_equals(output.dataType, test.output.dataType);
            assert_array_equals(output.shape, test.output.shape);
          } else {
            const label = 'gru_cell_xxx';
            options.label = label;
            const regrexp = new RegExp('\\[' + label + '\\]');
            assert_throws_with_label(
                () => builder.gruCell(
                    input, weight, recurrentWeight, hiddenState,
                    test.hiddenSize, options),
                regrexp);
          }
        }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          inputFromOtherBuilder, weight, recurrentWeight, hiddenState,
          hiddenSize));
}, '[gruCell] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const weightFromOtherBuilder =
      otherBuilder.input('weight', kExampleWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          input, weightFromOtherBuilder, recurrentWeight, hiddenState,
          hiddenSize));
}, '[gruCell] throw if weight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentWeightFromOtherBuilder =
      otherBuilder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          input, weight, recurrentWeightFromOtherBuilder, hiddenState,
          hiddenSize));
}, '[gruCell] throw if recurrentWeight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const hiddenStateFromOtherBuilder =
      otherBuilder.input('hiddenState', kExampleHiddenStateDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          input, weight, recurrentWeight, hiddenStateFromOtherBuilder,
          hiddenSize));
}, '[gruCell] throw if hiddenState is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          input, weight, recurrentWeight, hiddenState, hiddenSize, options));
}, '[gruCell] throw if bias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentBiasFromOtherBuilder =
      otherBuilder.input('recurrentBias', kExampleRecurrentBiasDescriptor);
  const options = {recurrentBias: recurrentBiasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gruCell(
          input, weight, recurrentWeight, hiddenState, hiddenSize, options));
}, '[gruCell] throw if recurrentBias option is from another builder');
