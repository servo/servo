// META: title=validation tests for WebNN API sigmoid operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('sigmoid');

const label = 'sigmoid_xxx';
validateSingleInputOperation('sigmoid', label);
