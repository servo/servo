// META: title=validation tests for WebNN API relu operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateInputFromAnotherBuilder('relu');

validateUnaryOperation(
    'relu', [...floatingPointTypes, 'int32', 'int8'], /*alsoBuildActivation=*/ true);
