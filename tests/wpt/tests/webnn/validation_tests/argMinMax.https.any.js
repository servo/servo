// META: title=validation tests for WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kArgMinMaxOperators = [
  'argMin',
  'argMax',
];

kArgMinMaxOperators.forEach((operatorName) => {
  validateOptionsAxes(operatorName);
  validateInputFromAnotherBuilder(operatorName);
});
