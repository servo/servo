// META: title=validation tests for WebNN API hardSwish operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('hardSwish');

validateUnaryOperation(
    'hardSwish', floatingPointTypes, /*alsoBuildActivation=*/ true);
