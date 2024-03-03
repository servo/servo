// META: title=validation tests for WebNN API reduction  operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js
// META: timeout=long

'use strict';

[
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
].forEach((operationName) => {
  validateOptionsAxes(operationName);
});
