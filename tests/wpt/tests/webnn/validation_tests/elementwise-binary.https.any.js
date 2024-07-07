// META: title=validation tests for WebNN API element-wise binary operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kElementwiseBinaryOperators = [
  'add',
  'sub',
  'mul',
  'div',
  'max',
  'min',
  'pow',
];


const tests = [
  {
    name: '[binary] Test bidirectionally broadcastable dimensions.',
    //  Both inputs have axes of length one which are expanded
    //  during broadcasting.
    a: {dataType: 'float32', dimensions: [8, 1, 6, 1]},
    b: {dataType: 'float32', dimensions: [7, 1, 5]},
    output: {dataType: 'float32', dimensions: [8, 7, 6, 5]}
  },
  {
    name: '[binary] Test unidirectionally broadcastable dimensions.',
    // Input a has a single axis of length one which is
    // expanded during broadcasting.
    a: {dataType: 'float32', dimensions: [4, 2, 1]},
    b: {dataType: 'float32', dimensions: [4]},
    output: {dataType: 'float32', dimensions: [4, 2, 4]}
  },
  {
    name: '[binary] Test scalar broadcasting.',
    a: {dataType: 'float32', dimensions: [4, 2, 4]},
    b: {dataType: 'float32', dimensions: []},
    output: {dataType: 'float32', dimensions: [4, 2, 4]}
  },
  {
    name: '[binary] Throw if the input shapes are not broadcastable.',
    a: {dataType: 'float32', dimensions: [4, 2]},
    b: {dataType: 'float32', dimensions: [4]},
  },
  {
    name: '[binary] Throw if the input types don\'t match.',
    a: {dataType: 'float32', dimensions: [4, 2]},
    b: {dataType: 'int32', dimensions: [1]},
  },
];

function runElementWiseBinaryTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      if (!context.opSupportLimits().input.dataTypes.includes(
              test.a.dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                'a',
                {dataType: test.a.dataType, dimensions: test.a.dimensions}));
        return;
      }
      if (!context.opSupportLimits().input.dataTypes.includes(
              test.b.dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                'b',
                {dataType: test.b.dataType, dimensions: test.b.dimensions}));
        return;
      }
      const a = builder.input(
          'a', {dataType: test.a.dataType, dimensions: test.a.dimensions});
      const b = builder.input(
          'b', {dataType: test.b.dataType, dimensions: test.b.dimensions});

      if (test.output) {
        const output = builder[operatorName](a, b);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder[operatorName](a, b));
      }
    }, test.name.replace('[binary]', `[${operatorName}]`));
  });
}

kElementwiseBinaryOperators.forEach((operatorName) => {
  validateTwoInputsOfSameDataType(operatorName);
  validateTwoInputsBroadcastable(operatorName);
  validateTwoInputsFromMultipleBuilders(operatorName);
  runElementWiseBinaryTests(operatorName, tests);
});
