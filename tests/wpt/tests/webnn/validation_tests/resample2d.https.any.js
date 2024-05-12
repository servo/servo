// META: title=validation tests for WebNN API resample2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

// Tests for resample2d(input, options)
const tests = [
  {
    name: '[resample2d] Test building resample2d with default options',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    output: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
  },
  {
    name: '[resample2d] Test building resample2d with scales=[2.0, 2.0]',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0]},
    output: {dataType: 'float32', dimensions: [1, 1, 4, 8]},
  },
  {
    name: '[resample2d] Test building resample2d with scales=[0.5, 0.5]',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    options: {scales: [0.5, 0.5]},
    output: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
  },
  {
    name:
        '[resample2d] Test building resample2d with input\'s dataType = float16',
    input: {dataType: 'float16', dimensions: [1, 1, 5, 5]},
    options: {scales: [0.5, 0.5]},
    output: {dataType: 'float16', dimensions: [1, 1, 2, 2]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[0.5, 0.5] and explicit axes=[2, 3]',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    options: {scales: [0.5, 0.5], axes: [2, 3]},
    output: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[1.0, 2.0] and axes=[0, 1]',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [1.0, 2.0], axes: [0, 1]},
    output: {dataType: 'float32', dimensions: [1, 2, 2, 4]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[2.0, 2.0] and axes=[1, 2]',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0], axes: [1, 2]},
    output: {dataType: 'float32', dimensions: [1, 2, 4, 4]},
  },
  {
    name:
        '[resample2d] Test building resample2d with sizes=[3, 6] ignored scales',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0], sizes: [3, 6]},
    output: {dataType: 'float32', dimensions: [1, 1, 3, 6]},
  },
  {
    name:
        '[resample2d] Throw if the dataType of input is not float32 or float16',
    input: {dataType: 'int32', dimensions: [2, 4]},
  },
  {
    name: '[resample2d] Throw if the rank of input is not 4',
    input: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name: '[resample2d] Throw if the length of scales is not 2',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [1.0, 1.0, 2.0, 2.0]},
  },
  {
    name: '[resample2d] Throw if any scale value is negative',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [1.0, -2.0]},
  },
  {
    name: '[resample2d] Throw if any scale value is 0',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {scales: [0, 2.0]},
  },
  {
    name: '[resample2d] Throw if the length of sizes is not 2',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {sizes: [1, 1, 4, 6]},
  },
  {
    name: '[resample2d] Throw if input data type is not floating type',
    input: {dataType: 'int32', dimensions: [1, 1, 2, 4]},
    options: {sizes: [1, 1, 4, 6]},
  },
  {
    name:
        '[resample2d] Throw if any size value is out of \'unsigned long\' value range',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {sizes: [kMaxUnsignedLong + 1, kMaxUnsignedLong + 1]},
  },
  {
    name:
        '[resample2d] Throw if outputHeight being floor(scaleHeight*inputHeight) is too large',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    // The maximum dimension size is kMaxUnsignedLong (2 ** 32 - 1).
    // Here scaleHeight=kMaxUnsignedLong and inputHeight=2,
    // so outputHeight being kMaxUnsignedLong*2 > kMaxUnsignedLong .
    options: {scales: /*[scaleHeight, scaleWidth]*/[kMaxUnsignedLong, 1]},
  },
  {
    name: '[resample2d] Throw if scaleHeight is too small',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    // Here scaleHeight=0.02 and inputHeight=2,
    // so outputHeight would be 0.
    // Link to https://github.com/webmachinelearning/webnn/issues/391.
    options: {scales: /*[scaleHeight, scaleWidth]*/[0.02, 0.8]},
  },
  {
    name:
        '[resample2d] Throw if outputWidth being floor(scaleWidth*inputWidth) is too large',
    input: {dataType: 'float32', dimensions: [1, 1, 4, 2]},
    // The maximum dimension size is kMaxUnsignedLong (2 ** 32 - 1).
    // Here scaleWidth=kMaxUnsignedLong and inputWidth=2,
    // so outputWidth being kMaxUnsignedLong*2 > kMaxUnsignedLong .
    options: {scales: /*[scaleHeight, scaleWidth]*/[1, kMaxUnsignedLong]},
  },
  {
    name: '[resample2d] Throw if scaleWidth is too small',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    // Here scaleWidth=0.1 and inputWidth=4,
    // so outputWidth would be 0.
    // Link to https://github.com/webmachinelearning/webnn/issues/391.
    options: {scales: /*[scaleHeight, scaleWidth]*/[0.7, 0.1]},
  },
  {
    name: '[resample2d] Throw if the length of axes is not 2',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {axes: [0, 1, 2]},
  },
  {
    name:
        '[resample2d] Throw if any axis value is greater than or equal to the input rank',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {axes: [3, 4]},
  },
  {
    // The valid values in the axes sequence are [0, 1], [1, 2] or [2, 3]
    name: '[resample2d] Throw if the values of axes are inconsecutive',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {axes: [0, 2]},
  },
  {
    name: '[resample2d] Throw if the values of axes are same',
    input: {dataType: 'float32', dimensions: [1, 1, 2, 4]},
    options: {axes: [0, 0]},
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const options = test.options ?? {};
      if (test.output) {
        const output = builder.resample2d(input, options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder.resample2d(input, options));
      }
    }, test.name));

validateInputFromAnotherBuilder(
    'resample2d', {dataType: 'float32', dimensions: [2, 2, 2, 2]});
