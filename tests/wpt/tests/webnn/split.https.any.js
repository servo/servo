// META: title=test WebNN API split operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-split

const buildSplit = (operationName, builder, resources) => {
  // sequence<MLOperand> split(MLOperand input,
  //                           (unsigned long or sequence<unsigned long>) splits,
  //                           optional MLSplitOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.split()
  const outputOperands = builder[operationName](inputOperand, resources.splits, resources.options);
  resources.expected.forEach((resourceDict, index) => {
    namedOutputOperand[resourceDict.name] = outputOperands[index];
  });
  return namedOutputOperand;
};

testWebNNOperation('split', buildSplit);