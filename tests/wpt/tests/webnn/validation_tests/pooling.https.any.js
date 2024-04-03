// META: title=validation tests for WebNN API pooling operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kPoolingOperators = ['averagePool2d', 'l2Pool2d', 'maxPool2d'];

kPoolingOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(
      operatorName, {dataType: 'float32', dimensions: [2, 2, 2, 2]});
});
