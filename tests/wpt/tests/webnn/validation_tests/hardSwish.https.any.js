// META: title=validation tests for WebNN API hardSwish operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('hardSwish');

const label = 'hard_swish';
validateSingleInputOperation('hardSwish', label);
