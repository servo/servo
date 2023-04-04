// META: title=test WebNN API pad operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-pad

const buildPad = (operationName, builder, resources) => {
  // MLOperand pad(MLOperand input, sequence<unsigned long> beginningPadding, sequence<unsigned long> endingPadding, optional MLPadOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.pad()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.beginningPadding, resources.endingPadding, resources.options);
  return namedOutputOperand;
};

testWebNNOperation('pad', buildPad);