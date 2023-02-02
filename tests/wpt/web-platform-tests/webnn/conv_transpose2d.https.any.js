// META: title=test WebNN API convTranspose2d operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-convtranspose2d

const buildConvTranspose2d = (operationName, builder, resources) => {
  // MLOperand convTranspose2d(MLOperand input, MLOperand filter, optional MLConvTranspose2dOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, filterOperand] = createMultiInputOperands(builder, resources);
  let convTranspose2dOptions = {...resources.options};
  if (convTranspose2dOptions.bias) {
    convTranspose2dOptions.bias = createConstantOperand(builder, convTranspose2dOptions.bias);
  }
  if (convTranspose2dOptions.activation) {
    convTranspose2dOptions.activation = builder[convTranspose2dOptions.activation]();
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, filterOperand, convTranspose2dOptions);
  return namedOutputOperand;
};

testWebNNOperation('convTranspose2d', buildConvTranspose2d);