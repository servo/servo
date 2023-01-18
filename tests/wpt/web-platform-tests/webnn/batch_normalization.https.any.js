// META: title=test WebNN API batchNormalization operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-batchnorm

const buildBatchNorm = (operationName, builder, resources) => {
  // MLOperand batchNormalization(MLOperand input, MLOperand mean, MLOperand variance,
  //                              optional MLBatchNormalizationOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, meanOperand, varianceOperand] = createMultiInputOperands(builder, resources);
  const batchNormOptions = {...resources.options};
  if (batchNormOptions.scale) {
    batchNormOptions.scale = createConstantOperand(builder, batchNormOptions.scale);
  }
  if (batchNormOptions.bias) {
    batchNormOptions.bias = createConstantOperand(builder, batchNormOptions.bias);
  }
  if (batchNormOptions.activation) {
    batchNormOptions.activation = builder[batchNormOptions.activation]();
  }
  // invoke builder.batchNormalization()
  namedOutputOperand[resources.expected.name] =
      builder[operationName](inputOperand, meanOperand, varianceOperand, batchNormOptions);
  return namedOutputOperand;
};

testWebNNOperation('batchNormalization', buildBatchNorm);