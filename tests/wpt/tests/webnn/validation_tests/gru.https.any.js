// META: title=validation tests for WebNN API gru operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const steps = 2, batchSize = 3, inputSize = 4, hiddenSize = 5, oneDirection = 1,
      bothDirections = 2;

// Dimensions required of required inputs.
const kValidInputShape = [steps, batchSize, inputSize];
const kValidWeightShape = [oneDirection, 3 * hiddenSize, inputSize];
const kValidRecurrentWeightShape = [oneDirection, 3 * hiddenSize, hiddenSize];
// Dimensions required of optional inputs.
const kValidBiasShape = [oneDirection, 3 * hiddenSize];
const kValidRecurrentBiasShape = [oneDirection, 3 * hiddenSize];
const kValidInitialHiddenStateShape = [oneDirection, batchSize, hiddenSize];

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
const kExampleBiasDescriptor = {
  dataType: 'float32',
  shape: kValidBiasShape
};
const kExampleRecurrentBiasDescriptor = {
  dataType: 'float32',
  shape: kValidRecurrentBiasShape
};
const kExampleInitialHiddenStateDescriptor = {
  dataType: 'float32',
  shape: kValidInitialHiddenStateShape
};

const tests = [
  {
    name: '[gru] Test with default options',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize,
    outputs:
        [{dataType: 'float32', shape: [oneDirection, batchSize, hiddenSize]}]
  },
  {
    name: '[gru] Test with given options',
    input: kExampleInputDescriptor,
    weight: {
      dataType: 'float32',
      shape: [bothDirections, 3 * hiddenSize, inputSize]
    },
    recurrentWeight: {
      dataType: 'float32',
      shape: [bothDirections, 3 * hiddenSize, hiddenSize]
    },
    steps: steps,
    hiddenSize: hiddenSize,
    options: {
      bias: {dataType: 'float32', shape: [bothDirections, 3 * hiddenSize]},
      recurrentBias:
          {dataType: 'float32', shape: [bothDirections, 3 * hiddenSize]},
      initialHiddenState:
          {dataType: 'float32', shape: [bothDirections, batchSize, hiddenSize]},
      restAfter: true,
      returnSequence: true,
      direction: 'both',
      layout: 'rzn',
      activations: ['sigmoid', 'relu']
    },
    outputs: [
      {dataType: 'float32', shape: [bothDirections, batchSize, hiddenSize]}, {
        dataType: 'float32',
        shape: [steps, bothDirections, batchSize, hiddenSize]
      }
    ]
  },
  {
    name: '[gru] TypeError is expected if steps equals to zero',
    input: kExampleInputDescriptor,
    weight:
        {dataType: 'float32', shape: [oneDirection, 4 * hiddenSize, inputSize]},
    recurrentWeight: {
      dataType: 'float32',
      shape: [oneDirection, 4 * hiddenSize, hiddenSize]
    },
    steps: 0,
    hiddenSize: hiddenSize,
  },
  {
    name: '[gru] TypeError is expected if hiddenSize equals to zero',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: 0
  },
  {
    name: '[gru] TypeError is expected if hiddenSize is too large',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: 4294967295,
  },
  {
    name:
        '[gru] TypeError is expected if the data type of the inputs is not one of the floating point types',
    input: {dataType: 'uint32', shape: kValidInputShape},
    weight: {dataType: 'uint32', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'uint32', shape: kValidRecurrentWeightShape},
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name: '[gru] TypeError is expected if the rank of input is not 3',
    input: {dataType: 'float32', shape: [steps, batchSize]},
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name: '[gru] TypeError is expected if input.shape[0] is not equal to steps',
    input: {dataType: 'float32', shape: [1000, batchSize, inputSize]},
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name:
        '[gru] TypeError is expected if weight.shape[1] is not 3 * hiddenSize',
    input: kExampleInputDescriptor,
    weight:
        {dataType: 'float32', shape: [oneDirection, 4 * hiddenSize, inputSize]},
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name: '[gru] TypeError is expected if the rank of recurrentWeight is not 3',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight:
        {dataType: 'float32', shape: [oneDirection, 3 * hiddenSize]},
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name: '[gru] TypeError is expected if the recurrentWeight.shape is invalid',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight:
        {dataType: 'float32', shape: [oneDirection, 4 * hiddenSize, inputSize]},
    steps: steps,
    hiddenSize: hiddenSize
  },
  {
    name:
        '[gru] TypeError is expected if the size of options.activations is not 2',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize,
    options: {activations: ['sigmoid', 'tanh', 'relu']}
  },
  {
    name: '[gru] TypeError is expected if the rank of options.bias is not 2',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float32', shape: [oneDirection]}}
  },
  {
    name:
        '[gru] TypeError is expected if options.bias.shape[1] is not 3 * hiddenSize',
    input: kExampleInputDescriptor,
    weight: kExampleWeightDescriptor,
    recurrentWeight: kExampleRecurrentWeightDescriptor,
    steps: steps,
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float32', shape: [oneDirection, hiddenSize]}}
  },
  {
    name:
        '[gru] TypeError is expected if options.recurrentBias.shape[1] is not 3 * hiddenSize',
    input: {dataType: 'float16', shape: kValidInputShape},
    weight: {dataType: 'float16', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'float16', shape: kValidRecurrentWeightShape},
    steps: steps,
    hiddenSize: hiddenSize,
    options: {
      recurrentBias:
          {dataType: 'float16', shape: [oneDirection, 4 * hiddenSize]}
    }
  },
  {
    name:
        '[gru] TypeError is expected if the rank of options.initialHiddenState is not 3',
    input: {dataType: 'float16', shape: kValidInputShape},
    weight: {dataType: 'float16', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'float16', shape: kValidRecurrentWeightShape},
    steps: steps,
    hiddenSize: hiddenSize,
    options: {
      initialHiddenState:
          {dataType: 'float16', shape: [oneDirection, batchSize]}
    }
  },
  {
    name:
        '[gru] TypeError is expected if options.initialHiddenState.shape[2] is not inputSize',
    input: {dataType: 'float16', shape: kValidInputShape},
    weight: {dataType: 'float16', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'float16', shape: kValidRecurrentWeightShape},
    steps: steps,
    hiddenSize: hiddenSize,
    options: {
      initialHiddenState: {
        dataType: 'float16',
        shape: [oneDirection, batchSize, 3 * hiddenSize]
      }
    }
  },
  {
    name:
        '[gru] TypeError is expected if the dataType of options.initialHiddenState is incorrect',
    input: {dataType: 'float16', shape: kValidInputShape},
    weight: {dataType: 'float16', shape: kValidWeightShape},
    recurrentWeight: {dataType: 'float16', shape: kValidRecurrentWeightShape},
    steps: steps,
    hiddenSize: hiddenSize,
    options: {
      initialHiddenState:
          {dataType: 'uint64', shape: [oneDirection, batchSize, hiddenSize]}
    }
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const weight = builder.input('weight', test.weight);
      const recurrentWeight =
          builder.input('recurrentWeight', test.recurrentWeight);

      const options = {};
      if (test.options) {
        if (test.options.bias) {
          options.bias = builder.input('bias', test.options.bias);
        }
        if (test.options.recurrentBias) {
          options.recurrentBias =
              builder.input('recurrentBias', test.options.recurrentBias);
        }
        if (test.options.initialHiddenState) {
          options.initialHiddenState = builder.input(
              'initialHiddenState', test.options.initialHiddenState);
        }
        if (test.options.resetAfter) {
          options.resetAfter = test.options.resetAfter;
        }
        if (test.options.returnSequence) {
          options.returnSequence = test.options.returnSequence;
        }
        if (test.options.direction) {
          options.direction = test.options.direction;
        }
        if (test.options.layout) {
          options.layout = test.options.layout;
        }
        if (test.options.activations) {
          options.activations = test.options.activations;
        }
      }

      if (test.outputs &&
          context.opSupportLimits().gru.input.dataTypes.includes(
              test.input.dataType)) {
        const outputs = builder.gru(
            input, weight, recurrentWeight, test.steps, test.hiddenSize,
            options);
        assert_equals(outputs.length, test.outputs.length);
        for (let i = 0; i < outputs.length; ++i) {
          assert_equals(outputs[i].dataType(), test.outputs[i].dataType);
          assert_array_equals(outputs[i].shape(), test.outputs[i].shape);
        }
      } else {
        const label = 'gru_xxx';
        options.label = label;
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.gru(
                input, weight, recurrentWeight, test.steps, test.hiddenSize,
                options),
            regrexp);
      }
    }, test.name));

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          inputFromOtherBuilder, weight, recurrentWeight, steps, hiddenSize));
}, '[gru] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const weightFromOtherBuilder =
      otherBuilder.input('weight', kExampleWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          input, weightFromOtherBuilder, recurrentWeight, steps, hiddenSize));
}, '[gru] throw if weight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentWeightFromOtherBuilder =
      otherBuilder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          input, weight, recurrentWeightFromOtherBuilder, steps, hiddenSize));
}, '[gru] throw if recurrentWeight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          input, weight, recurrentWeight, steps, hiddenSize, options));
}, '[gru] throw if bias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentBiasFromOtherBuilder =
      otherBuilder.input('recurrentBias', kExampleRecurrentBiasDescriptor);
  const options = {recurrentBias: recurrentBiasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          input, weight, recurrentWeight, steps, hiddenSize, options));
}, '[gru] throw if recurrentBias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const initialHiddenStateFromOtherBuilder = otherBuilder.input(
      'initialHiddenState', kExampleInitialHiddenStateDescriptor);
  const options = {initialHiddenState: initialHiddenStateFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.gru(
          input, weight, recurrentWeight, steps, hiddenSize, options));
}, '[gru] throw if initialHiddenState option is from another builder');
