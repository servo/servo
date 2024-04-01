// META: title=validation tests for WebNN API tanh operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('tanh');

validateUnaryOperation(
    'tanh', floatingPointTypes, /*alsoBuildActivation=*/ true);
