// META: title=validation tests for WebNN API element-wise logical operations
// META: global=window
// META: variant=?op=equal&device=cpu
// META: variant=?op=equal&device=gpu
// META: variant=?op=equal&device=npu
// META: variant=?op=greater&device=cpu
// META: variant=?op=greater&device=gpu
// META: variant=?op=greater&device=npu
// META: variant=?op=greaterOrEqual&device=cpu
// META: variant=?op=greaterOrEqual&device=gpu
// META: variant=?op=greaterOrEqual&device=npu
// META: variant=?op=lesser&device=cpu
// META: variant=?op=lesser&device=gpu
// META: variant=?op=lesser&device=npu
// META: variant=?op=lesserOrEqual&device=cpu
// META: variant=?op=lesserOrEqual&device=gpu
// META: variant=?op=lesserOrEqual&device=npu
// META: variant=?op=notEqual&device=cpu
// META: variant=?op=notEqual&device=gpu
// META: variant=?op=notEqual&device=npu
// META: variant=?op=logicalAnd&device=cpu
// META: variant=?op=logicalAnd&device=gpu
// META: variant=?op=logicalAnd&device=npu
// META: variant=?op=logicalOr&device=cpu
// META: variant=?op=logicalOr&device=gpu
// META: variant=?op=logicalOr&device=npu
// META: variant=?op=logicalXor&device=cpu
// META: variant=?op=logicalXor&device=gpu
// META: variant=?op=logicalXor&device=npu
// META: variant=?op=logicalNot&device=cpu
// META: variant=?op=logicalNot&device=gpu
// META: variant=?op=logicalNot&device=npu
// META: script=../resources/utils_validation.js

'use strict';

const queryParams = new URLSearchParams(window.location.search);
const operatorName = queryParams.get('op');

if (operatorName === 'logicalNot') {
  // The `logicalNot()` operator is unary.
  validateInputFromAnotherBuilder(operatorName);
} else {
  const label = 'elementwise_logic_op';
  validateTwoInputsOfSameDataType(operatorName, label);
  validateTwoInputsFromMultipleBuilders(operatorName);
  validateTwoInputsBroadcastable(operatorName, label);
}
