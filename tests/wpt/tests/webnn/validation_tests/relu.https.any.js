// META: title=validation tests for WebNN API relu operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('relu');

const label = 'relu_1';
validateSingleInputOperation('relu', label);
