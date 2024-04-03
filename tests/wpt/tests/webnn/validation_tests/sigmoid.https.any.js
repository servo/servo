// META: title=validation tests for WebNN API sigmoid operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('sigmoid');

validateUnaryOperation(
    'sigmoid', floatingPointTypes, /*alsoBuildActivation=*/ true);
