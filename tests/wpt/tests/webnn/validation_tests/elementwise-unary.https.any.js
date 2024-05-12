// META: title=validation tests for WebNN API element-wise unary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseUnaryOperators = [
  'abs', 'ceil', 'cos', 'erf', 'exp', 'floor', 'identity', 'log', 'neg',
  'reciprocal', 'sin', 'sqrt', 'tan'
];

kElementwiseUnaryOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
});

const kElementwiseUnaryOperations = [
  {
    name: 'abs',
    supportedDataTypes: [...floatingPointTypes, 'int32', 'int8']
  },
  {name: 'ceil', supportedDataTypes: floatingPointTypes},
  {name: 'exp', supportedDataTypes: floatingPointTypes},
  {name: 'floor', supportedDataTypes: floatingPointTypes},
  {name: 'log', supportedDataTypes: floatingPointTypes}, {
    name: 'neg',
    supportedDataTypes: [...floatingPointTypes, 'int32', 'int8']
  },
  {name: 'sin', supportedDataTypes: floatingPointTypes},
  {name: 'tan', supportedDataTypes: floatingPointTypes},
  {name: 'erf', supportedDataTypes: floatingPointTypes},
  {name: 'identity', supportedDataTypes: allWebNNOperandDataTypes},
  {name: 'logicalNot', supportedDataTypes: ['uint8']},
  {name: 'reciprocal', supportedDataTypes: floatingPointTypes},
  {name: 'sqrt', supportedDataTypes: floatingPointTypes}
];

kElementwiseUnaryOperations.forEach((operation) => {
  validateUnaryOperation(operation.name, operation.supportedDataTypes);
});
