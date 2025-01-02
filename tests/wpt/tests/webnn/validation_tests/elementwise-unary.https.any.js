// META: title=validation tests for WebNN API element-wise unary operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseUnaryOperators = [
  'abs', 'ceil', 'cos', 'erf', 'exp', 'floor', 'identity', 'log', 'neg',
  'reciprocal', 'sign', 'sin', 'sqrt', 'tan'
];

kElementwiseUnaryOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
});

const label = 'elementwise_unary_op';
kElementwiseUnaryOperators.forEach((operatorName) => {
  validateSingleInputOperation(operatorName, label);
});
