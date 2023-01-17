// META: title=test WebNN API slice operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-slice

const buildSlice = (operationName, builder, resources) => {
  // MLOperand slice(MLOperand input, sequence<long> starts, sequence<long> sizes, optional MLSliceOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.slice()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.starts, resources.sizes, resources.options);
  return namedOutputOperand;
};

testWebNNOperation('slice', buildSlice);