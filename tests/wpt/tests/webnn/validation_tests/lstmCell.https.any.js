// META: title=validation tests for WebNN API lstmCell operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const batchSize = 3, inputSize = 4, hiddenSize = 5;

// Dimensions required of required inputs.
const kValidInputDimensions = [batchSize, inputSize];
const kValidWeightDimensions = [4 * hiddenSize, inputSize];
const kValidRecurrentWeightDimensions = [4 * hiddenSize, hiddenSize];
const kValidHiddenStateDimensions = [batchSize, hiddenSize];
const kValidCellStateDimensions = [batchSize, hiddenSize];
// Dimensions required of optional inputs.
const kValidBiasDimensions = [4 * hiddenSize];
const kValidPeepholeWeightDimensions = [3 * hiddenSize];

// Example descriptors which are valid according to the above dimensions.
const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: kValidInputDimensions
};
const kExampleWeightDescriptor = {
  dataType: 'float32',
  dimensions: kValidWeightDimensions
};
const kExampleRecurrentWeightDescriptor = {
  dataType: 'float32',
  dimensions: kValidRecurrentWeightDimensions
};
const kExampleHiddenStateDescriptor = {
  dataType: 'float32',
  dimensions: kValidHiddenStateDimensions
};
const kExampleCellStateDescriptor = {
  dataType: 'float32',
  dimensions: kValidCellStateDimensions
};
const kExampleBiasDescriptor = {
  dataType: 'float32',
  dimensions: kValidBiasDimensions
};
const kExamplePeepholeWeightDescriptor = {
  dataType: 'float32',
  dimensions: kValidPeepholeWeightDimensions
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          inputFromOtherBuilder, weight, recurrentWeight, hiddenState,
          cellState, hiddenSize));
}, '[lstmCell] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const weightFromOtherBuilder =
      otherBuilder.input('weight', kExampleWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weightFromOtherBuilder, recurrentWeight, hiddenState,
          cellState, hiddenSize));
}, '[lstmCell] throw if weight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentWeightFromOtherBuilder =
      otherBuilder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeightFromOtherBuilder, hiddenState,
          cellState, hiddenSize));
}, '[lstmCell] throw if recurrentWeight is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const hiddenStateFromOtherBuilder =
      otherBuilder.input('hiddenState', kExampleHiddenStateDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenStateFromOtherBuilder,
          cellState, hiddenSize));
}, '[lstmCell] throw if hiddenState is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const cellStateFromOtherBuilder =
      otherBuilder.input('cellState', kExampleCellStateDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenState,
          cellStateFromOtherBuilder, hiddenSize));
}, '[lstmCell] throw if cellState is from another builder');

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
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenState, cellState, hiddenSize,
          options));
}, '[lstmCell] throw if bias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const recurrentBiasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {recurrentBias: recurrentBiasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenState, cellState, hiddenSize,
          options));
}, '[lstmCell] throw if recurrentBias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const peepholeWeightFromOtherBuilder =
      otherBuilder.input('peepholeWeight', kExamplePeepholeWeightDescriptor);
  const options = {peepholeWeight: peepholeWeightFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenState, cellState, hiddenSize,
          options));
}, '[lstmCell] throw if peepholeWeight option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const activation = builder.relu();
  const activationFromOtherBuilder = otherBuilder.relu();
  const options = {activations: [activation, activationFromOtherBuilder]};

  const input = builder.input('input', kExampleInputDescriptor);
  const weight = builder.input('weight', kExampleWeightDescriptor);
  const recurrentWeight =
      builder.input('recurrentWeight', kExampleRecurrentWeightDescriptor);
  const hiddenState =
      builder.input('hiddenState', kExampleHiddenStateDescriptor);
  const cellState = builder.input('cellState', kExampleCellStateDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.lstmCell(
          input, weight, recurrentWeight, hiddenState, cellState, hiddenSize,
          options));
}, '[lstmCell] throw if activation option is from another builder');

const tests = [
  {
    name: '[lstmCell] Test with default options',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    outputs: [
      {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
      {dataType: 'float16', dimensions: [batchSize, hiddenSize]}
    ]
  },
  {
    name: '[lstmCell] Test with given options',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {
      bias: {dataType: 'float32', dimensions: [4 * hiddenSize]},
      recurrentBias: {dataType: 'float32', dimensions: [4 * hiddenSize]},
      peepholeWeight: {dataType: 'float32', dimensions: [3 * hiddenSize]},
      layout: 'ifgo',
      activations: ['sigmoid', 'relu', 'tanh']
    },
    outputs: [
      {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
      {dataType: 'float32', dimensions: [batchSize, hiddenSize]}
    ]
  },
  {
    name: '[lstmCell] Throw if hiddenSize is equal to zero',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: 0
  },
  {
    name: '[lstmCell] Throw if hiddenSize is too large',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: 4294967295
  },
  {
    name:
        '[lstmCell] Throw if the input data type is not one of the floating point types',
    input: {dataType: 'uint32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the rank of input is not 2',
    input: {dataType: 'float32', dimensions: [batchSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the shape of input is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, 1000]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the data type of weight is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the rank of weight is not 2',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize, 1000]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the shape of weight is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [1000, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the data type of recurrentWeight is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the rank of recurrentWeight is not 2',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [1000, 4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the shape of recurrentWeight is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight: {dataType: 'float32', dimensions: [1000, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the data type of hiddenState is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'int64', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the rank of hiddenState is not 2',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the shape of hiddenState is incorrect',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, 1000]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the data type of cellState is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the rank of cellState is not 2',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the shape of cellState is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, 1000]},
    hiddenSize: hiddenSize
  },
  {
    name: '[lstmCell] Throw if the data type of options.bias is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'int8', dimensions: [4 * hiddenSize]}}
  },
  {
    name: '[lstmCell] Throw if the rank of options.bias is not 1',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float16', dimensions: [4 * hiddenSize, 1000]}}
  },
  {
    name: '[lstmCell] Throw if the shape of options.bias is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {bias: {dataType: 'float16', dimensions: [1000]}}
  },
  {
    name:
        '[lstmCell] Throw if the data type of options.recurrentBias is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {recurrentBias: {dataType: 'uint8', dimensions: [4 * hiddenSize]}}
  },
  {
    name: '[lstmCell] Throw if the rank of options.recurrentBias is not 1',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {
      recurrentBias: {dataType: 'float16', dimensions: [4 * hiddenSize, 1000]}
    }
  },
  {
    name: '[lstmCell] Throw if the shape of options.recurrentBias is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {recurrentBias: {dataType: 'float16', dimensions: [1000]}}
  },
  {
    name:
        '[lstmCell] Throw if the data type of options.peepholeWeight is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options:
        {peepholeWeight: {dataType: 'float32', dimensions: [3 * hiddenSize]}}
  },
  {
    name: '[lstmCell] Throw if the rank of options.peepholeWeight is not 1',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {peepholeWeight: {dataType: 'float16', dimensions: []}}
  },
  {
    name:
        '[lstmCell] Throw if the shape of options.peepholeWeight is incorrect',
    input: {dataType: 'float16', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float16', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float16', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float16', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {peepholeWeight: {dataType: 'float16', dimensions: [1000]}}
  },
  {
    name: '[lstmCell] Throw if the size of options.activations is not 3',
    input: {dataType: 'float32', dimensions: [batchSize, inputSize]},
    weight: {dataType: 'float32', dimensions: [4 * hiddenSize, inputSize]},
    recurrentWeight:
        {dataType: 'float32', dimensions: [4 * hiddenSize, hiddenSize]},
    hiddenState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    cellState: {dataType: 'float32', dimensions: [batchSize, hiddenSize]},
    hiddenSize: hiddenSize,
    options: {activations: ['sigmoid', 'tanh', 'sigmoid', 'tanh']}
  }
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const weight = builder.input(
          'weight',
          {dataType: test.weight.dataType, dimensions: test.weight.dimensions});
      const recurrentWeight = builder.input('recurrentWeight', {
        dataType: test.recurrentWeight.dataType,
        dimensions: test.recurrentWeight.dimensions
      });
      const hiddenState = builder.input('hiddenState', {
        dataType: test.hiddenState.dataType,
        dimensions: test.hiddenState.dimensions
      });
      const cellState = builder.input('cellState', {
        dataType: test.cellState.dataType,
        dimensions: test.cellState.dimensions
      });

      const options = {};
      if (test.options) {
        if (test.options.bias) {
          options.bias = builder.input('bias', {
            dataType: test.options.bias.dataType,
            dimensions: test.options.bias.dimensions
          });
        }
        if (test.options.recurrentBias) {
          options.bias = builder.input('recurrentBias', {
            dataType: test.options.recurrentBias.dataType,
            dimensions: test.options.recurrentBias.dimensions
          });
        }
        if (test.options.peepholeWeight) {
          options.peepholeWeight = builder.input('peepholeWeight', {
            dataType: test.options.peepholeWeight.dataType,
            dimensions: test.options.peepholeWeight.dimensions
          });
        }
        if (test.options.layout) {
          options.layout = test.options.layout;
        }
        if (test.options.activations) {
          options.activations = [];
          test.options.activations.forEach(
              activation => options.activations.push(builder[activation]()));
        }
      }

      if (test.outputs) {
        const outputs = builder.lstmCell(
            input, weight, recurrentWeight, hiddenState, cellState,
            test.hiddenSize, options);
        assert_equals(outputs.length, test.outputs.length);
        for (let i = 0; i < outputs.length; ++i) {
          assert_equals(outputs[i].dataType(), test.outputs[i].dataType);
          assert_array_equals(outputs[i].shape(), test.outputs[i].dimensions);
        }
      } else {
        assert_throws_js(
            TypeError,
            () => builder.lstmCell(
                input, weight, recurrentWeight, hiddenState, cellState,
                test.hiddenSize, options));
      }
    }, test.name));
