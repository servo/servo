// META: title=validation tests for WebNN API element-wise logical operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseLogicalBinaryOperators = [
  'equal',
  'greater',
  'greaterOrEqual',
  'lesser',
  'lesserOrEqual',
];

kElementwiseLogicalBinaryOperators.forEach((operatorName) => {
  validateTwoInputsOfSameDataType(operatorName);
  validateTwoInputsFromMultipleBuilders(operatorName);
  validateTwoInputsBroadcastable(operatorName);
});

// The `logicalNot()` operator is unary.
validateInputFromAnotherBuilder('logicalNot');
