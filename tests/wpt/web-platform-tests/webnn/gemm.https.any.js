// META: title=test WebNN API gemm operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-gemm

const buildGemm= (operationName, builder, resources) => {
  // MLOperand gemm(MLOperand a, MLOperand b, optional MLGemmOptions options = {});
  const namedOutputOperand = {};
  const [inputOperandA, inputOperandB] = createMultiInputOperands(builder, resources);
  let gemmOptions = {...resources.options};
  if (gemmOptions.c) {
    if (gemmOptions.c.shape) {
      gemmOptions.c = createConstantOperand(builder, gemmOptions.c);
    } else {
      // MLOperand c;
      // Create a single-value operand when c is a scalar
      gemmOptions.c = builder.constant(gemmOptions.c);
    }
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperandA, inputOperandB, gemmOptions);
  return namedOutputOperand;
};

testWebNNOperation('gemm', buildGemm);