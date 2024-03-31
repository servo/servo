// META: title=validation tests for WebNN API reduction operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kReductionOperators = [
  'reduceL1',
  'reduceL2',
  'reduceLogSum',
  'reduceLogSumExp',
  'reduceMax',
  'reduceMean',
  'reduceMin',
  'reduceProduct',
  'reduceSum',
  'reduceSumSquare',
];

kReductionOperators.forEach((operatorName) => {
  validateOptionsAxes(operatorName);
  validateInputFromAnotherBuilder(operatorName);
});
