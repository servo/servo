// META: title=validation tests for WebNN API softplus operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('softplus');

const label = 'softplus_xxx';
validateSingleInputOperation('softplus', label);
