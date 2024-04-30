// META: title=validation tests for WebNN API softplus operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('softplus');

validateUnaryOperation(
    'softplus', floatingPointTypes, /*alsoBuildActivation=*/ true);
