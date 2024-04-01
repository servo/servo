// META: title=validation tests for WebNN API softsign operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('softsign');

validateUnaryOperation(
    'softsign', floatingPointTypes, /*alsoBuildActivation=*/ true);
