// META: title=validation tests for WebNN API element-wise binary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseBinaryOperators = [
  'add',
  'sub',
  'mul',
  'div',
  'max',
  'min',
  'pow',
];

kElementwiseBinaryOperators.forEach((operatorName) => {
  validateTwoInputsOfSameDataType(operatorName);
  validateTwoInputsBroadcastable(operatorName);
  validateTwoInputsFromMultipleBuilders(operatorName);
});
