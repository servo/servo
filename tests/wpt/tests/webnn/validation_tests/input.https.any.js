// META: title=validation tests for WebNN API input interface
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js
// META: timeout=long

'use strict';

// Tests for input(name, descriptor)
const tests = [
  {
    testName:
        '[input] Test building a 0-D scalar input without presenting dimensions',
    name: 'input',
    descriptor: {dataType: 'float32'},
    output: {dataType: 'float32', dimensions: []},
  },
  {
    testName: '[input] Test building a 0-D scalar input with empty dimensions',
    name: 'input',
    descriptor: {dataType: 'float32', dimensions: []},
    output: {dataType: 'float32', dimensions: []},
  },
  {
    testName: '[input] Test building a 1-D input with int64 data type',
    name: 'input',
    descriptor: {dataType: 'int64', dimensions: [3]},
    output: {dataType: 'int64', dimensions: [3]},
  },
  {
    testName: '[input] Test building a 2-D input without errors',
    name: 'input',
    descriptor: {dataType: 'float32', dimensions: [3, 4]},
    output: {dataType: 'float32', dimensions: [3, 4]},
  },
  {
    testName: '[input] Throw if the name is empty',
    name: '',
    descriptor: {dataType: 'float32', dimensions: [3, 4]}
  },
  {
    testName: '[input] Throw if a dimension size is 0',
    name: 'input',
    descriptor: {dataType: 'float32', dimensions: [3, 0]}
  },
  {
    testName: '[input] Throw if the number of elements is too large',
    name: 'input',
    descriptor: {
      dataType: 'float32',
      dimensions: [
        // Refer to
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Invalid_array_length
        2 ** 33 + 1
      ]
    }
  }
];

tests.forEach(
    test => promise_test(async t => {
      if (test.output) {
        const inputOperand = builder.input(test.name, test.descriptor);
        assert_equals(inputOperand.dataType(), test.output.dataType);
        assert_array_equals(inputOperand.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.input(test.name, test.descriptor));
      }
    }, test.testName));
