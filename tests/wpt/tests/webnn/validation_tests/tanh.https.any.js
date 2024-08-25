// META: title=validation tests for WebNN API tanh operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('tanh');

const label = 'tanh-xxx';
validateSingleInputOperation('tanh', label);
