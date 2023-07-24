// META: title=test WebNN API conv2d operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-conv2d

const buildConv2d= (operationName, builder, resources) => {
  // MLOperand conv2d(MLOperand input, MLOperand filter, optional MLConv2dOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, filterOperand] = createMultiInputOperands(builder, resources);
  let conv2dOptions = {...resources.options};
  if (conv2dOptions.bias) {
    conv2dOptions.bias = createConstantOperand(builder, conv2dOptions.bias);
  }
  if (conv2dOptions.activation) {
    conv2dOptions.activation = builder[conv2dOptions.activation]();
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, filterOperand, conv2dOptions);
  return namedOutputOperand;
};

testWebNNOperation('conv2d', buildConv2d);