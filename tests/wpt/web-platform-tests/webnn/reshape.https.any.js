// META: title=test WebNN API reshape operation
// META: global=window,dedicatedworker
// META: script=./resources/utils.js
// META: timeout=long

'use strict';

// https://webmachinelearning.github.io/webnn/#api-mlgraphbuilder-reshape

const buildReshape = (operationName, builder, resources) => {
  // MLOperand reshape(MLOperand input, sequence<long> newShape);
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.reshape()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.newShape);
  return namedOutputOperand;
};

testWebNNOperation('reshape', buildReshape);

