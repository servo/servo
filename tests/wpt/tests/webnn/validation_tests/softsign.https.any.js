// META: title=validation tests for WebNN API softsign operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('softsign');

const label = 'softsign_xxx';
validateSingleInputOperation('softsign', label);
