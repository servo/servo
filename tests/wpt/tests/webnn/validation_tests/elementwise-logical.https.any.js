// META: title=validation tests for WebNN API element-wise logical operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseLogicalBinaryOperators = [
  'equal',
  'greater',
  'greaterOrEqual',
  'lesser',
  'lesserOrEqual',
  'logicalAnd',
  'logicalOr',
  'logicalXor',
];

const label = 'elementwise_logic_op';

kElementwiseLogicalBinaryOperators.forEach((operatorName) => {
  validateTwoInputsOfSameDataType(operatorName, label);
  validateTwoInputsFromMultipleBuilders(operatorName);
  validateTwoInputsBroadcastable(operatorName, label);
});

// The `logicalNot()` operator is unary.
validateInputFromAnotherBuilder('logicalNot');
