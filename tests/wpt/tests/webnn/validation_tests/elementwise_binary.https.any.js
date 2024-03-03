// META: title=validation tests for WebNN API element-wise binary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js
// META: timeout=long

'use strict';

['add', 'sub', 'mul', 'div', 'max', 'min', 'pow'].forEach((operationName) => {
  validateTwoInputsOfSameDataType(operationName);
  validateTwoInputsBroadcastable(operationName);
});
