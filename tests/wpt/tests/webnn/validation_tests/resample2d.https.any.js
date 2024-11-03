// META: title=validation tests for WebNN API resample2d operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const label = 'resample-2d';
// Tests for resample2d(input, options)
const tests = [
  {
    name: '[resample2d] Test building resample2d with default options',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    output: {dataType: 'float32', shape: [1, 1, 2, 4]},
  },
  {
    name: '[resample2d] Test building resample2d with scales=[2.0, 2.0]',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0]},
    output: {dataType: 'float32', shape: [1, 1, 4, 8]},
  },
  {
    name: '[resample2d] Test building resample2d with scales=[0.5, 0.5]',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    options: {scales: [0.5, 0.5]},
    output: {dataType: 'float32', shape: [1, 1, 2, 2]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[0.5, 0.5] and explicit axes=[2, 3]',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    options: {scales: [0.5, 0.5], axes: [2, 3]},
    output: {dataType: 'float32', shape: [1, 1, 2, 2]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[1.0, 2.0] and axes=[0, 1]',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {scales: [1.0, 2.0], axes: [0, 1]},
    output: {dataType: 'float32', shape: [1, 2, 2, 4]},
  },
  {
    name:
        '[resample2d] Test building resample2d with scales=[2.0, 2.0] and axes=[1, 2]',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0], axes: [1, 2]},
    output: {dataType: 'float32', shape: [1, 2, 4, 4]},
  },
  {
    name:
        '[resample2d] Test building resample2d with sizes=[3, 6] ignored scales',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {scales: [2.0, 2.0], sizes: [3, 6]},
    output: {dataType: 'float32', shape: [1, 1, 3, 6]},
  },
  {
    name:
        '[resample2d] Test building resample2d with non consecutive axes=[0,2]',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      axes: [0, 2],
      label: label,
    },
    output: {dataType: 'float32', shape: [1, 1, 2, 4]},
  },
  {
    name:
        '[resample2d] Throw if the dataType of input is not float32 or float16',
    input: {dataType: 'int32', shape: [2, 4]},
    options: {label},
  },
  {
    name: '[resample2d] Throw if the rank of input is not 4',
    input: {dataType: 'float32', shape: [2, 4]},
    options: {label},
  },
  {
    name: '[resample2d] Throw if the length of scales is not 2',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      scales: [1.0, 1.0, 2.0, 2.0],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if any scale value is negative',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      scales: [1.0, -2.0],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if any scale value is 0',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      scales: [0, 2.0],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if the length of sizes is not 2',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      sizes: [1, 1, 4, 6],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if sizes[0] is not a valid dimension',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      sizes: [0, 1],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if sizes[1] is not a valid dimension',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      sizes: [1, 0],
      label: label,
    },
  },
  {
    name:
        '[resample2d] Throw if any size value is out of \'unsigned long\' value range',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {sizes: [kMaxUnsignedLong + 1, kMaxUnsignedLong + 1]},
  },
  {
    name:
        '[resample2d] Throw if outputHeight being floor(scaleHeight*inputHeight) is too large',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    // The maximum dimension size is kMaxUnsignedLong (2 ** 32 - 1).
    // Here scaleHeight=kMaxUnsignedLong and inputHeight=2,
    // so outputHeight being kMaxUnsignedLong*2 > kMaxUnsignedLong .
    options: {scales: /*[scaleHeight, scaleWidth]*/[kMaxUnsignedLong, 1]},
  },
  {
    name: '[resample2d] Throw if scaleHeight is too small',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    // Here scaleHeight=0.02 and inputHeight=2,
    // so outputHeight would be 0.
    // Link to https://github.com/webmachinelearning/webnn/issues/391.
    options: {
      scales: /*[scaleHeight, scaleWidth]*/[0.02, 0.8],
      label: label,
    },
  },
  {
    name:
        '[resample2d] Throw if outputWidth being floor(scaleWidth*inputWidth) is too large',
    input: {dataType: 'float32', shape: [1, 1, 4, 2]},
    // The maximum dimension size is kMaxUnsignedLong (2 ** 32 - 1).
    // Here scaleWidth=kMaxUnsignedLong and inputWidth=2,
    // so outputWidth being kMaxUnsignedLong*2 > kMaxUnsignedLong .
    options: {scales: /*[scaleHeight, scaleWidth]*/[1, kMaxUnsignedLong]},
  },
  {
    name: '[resample2d] Throw if scaleWidth is too small',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    // Here scaleWidth=0.1 and inputWidth=4,
    // so outputWidth would be 0.
    // Link to https://github.com/webmachinelearning/webnn/issues/391.
    options: {
      scales: /*[scaleHeight, scaleWidth]*/[0.7, 0.1],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if the length of axes is not 2',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      axes: [0, 1, 2],
      label: label,
    },
  },
  {
    name:
        '[resample2d] Throw if any axis value is greater than or equal to the input rank',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      axes: [3, 4],
      label: label,
    },
  },
  {
    name: '[resample2d] Throw if the values of axes are same',
    input: {dataType: 'float32', shape: [1, 1, 2, 4]},
    options: {
      axes: [0, 0],
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const options = test.options ?? {};
      if (test.output) {
        const output = builder.resample2d(input, options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const options = {...test.options};
        if (options.label) {
          const regrexp = new RegExp('\\[' + label + '\\]');
          assert_throws_with_label(
              () => builder.resample2d(input, options), regrexp);
        } else {
          assert_throws_js(TypeError, () => builder.resample2d(input, options));
        }
      }
    }, test.name));

validateInputFromAnotherBuilder(
    'resample2d', {dataType: 'float32', shape: [2, 2, 2, 2]});

promise_test(async t => {
  for (let dataType of allWebNNOperandDataTypes) {
    if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
      continue;
    }
    const builder = new MLGraphBuilder(context);
    const shape = [1, 1, 2, 4];
    const input = builder.input(`input`, {dataType, shape});
    if (context.opSupportLimits().resample2d.input.dataTypes.includes(
            dataType)) {
      const output = builder.resample2d(input);
      assert_equals(output.dataType, dataType);
      assert_array_equals(output.shape, shape);
    } else {
      assert_throws_js(TypeError, () => builder.resample2d(input));
    }
  }
}, `[resample2d] Test resample2d with all of the data types.`);
