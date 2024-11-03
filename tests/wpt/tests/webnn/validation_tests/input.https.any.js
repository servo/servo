// META: title=validation tests for WebNN API input interface
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

// Tests for input(name, descriptor)
const tests = [
  {
    testName: '[input] Test building a 0-D scalar input with empty shape',
    name: 'input',
    descriptor: {dataType: 'float32', shape: []},
    output: {dataType: 'float32', shape: []},
  },
  {
    testName: '[input] Test building a 1-D input with int64 data type',
    name: 'input',
    descriptor: {dataType: 'int64', shape: [3]},
    output: {dataType: 'int64', shape: [3]},
  },
  {
    testName: '[input] Test building a 2-D input without errors',
    name: 'input',
    descriptor: {dataType: 'float32', shape: [3, 4]},
    output: {dataType: 'float32', shape: [3, 4]},
  },
  {
    testName: '[input] Throw if the name is empty',
    name: '',
    descriptor: {dataType: 'float32', shape: [3, 4]}
  },
  {
    testName: '[input] Throw if a dimension size is 0',
    name: 'input',
    descriptor: {dataType: 'float32', shape: [3, 0]}
  },
  {
    testName:
        '[input] Throw if the value of any element in dimensions is outside the \'unsigned long\' value range',
    name: 'input',
    descriptor: {dataType: 'float32', shape: [kMaxUnsignedLong + 1]}
  },
  {
    testName: '[input] Throw if the number of elements is too large',
    name: 'input',
    descriptor: {
      dataType: 'float32',
      shape: [kMaxUnsignedLong, kMaxUnsignedLong, kMaxUnsignedLong]
    }
  }
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      if (test.output) {
        const inputOperand = builder.input(test.name, test.descriptor);
        assert_equals(inputOperand.dataType, test.output.dataType);
        assert_array_equals(inputOperand.shape, test.output.shape);
      } else {
        assert_throws_js(
            TypeError, () => builder.input(test.name, test.descriptor));
      }
    }, test.testName));
