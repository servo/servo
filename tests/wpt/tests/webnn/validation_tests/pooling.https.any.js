// META: title=validation tests for WebNN API pooling operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kPoolingOperators = ['averagePool2d', 'l2Pool2d', 'maxPool2d'];

kPoolingOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(
      operatorName, {dataType: 'float32', shape: [2, 2, 2, 2]});
});

const label = 'pool_2d_xxx';
const tests = [
  {
    name: 'Test pool2d with default options.',
    input: {dataType: 'float32', shape: [1, 3, 4, 4]},
    output: {dataType: 'float32', shape: [1, 3, 1, 1]}
  },
  {
    name: 'Test pool2d with windowDimensions',
    input: {dataType: 'float16', shape: [1, 3, 4, 4]},
    options: {
      windowDimensions: [3, 3],
    },
    output: {dataType: 'float16', shape: [1, 3, 2, 2]}
  },
  {
    name: 'Test pool2d with padding.',
    input: {dataType: 'float32', shape: [1, 3, 5, 5]},
    options: {
      windowDimensions: [5, 5],
      padding: [2, 2, 2, 2],
    },
    output: {dataType: 'float32', shape: [1, 3, 5, 5]}
  },
  {
    name: 'Test pool2d with strides.',
    input: {dataType: 'float16', shape: [1, 3, 5, 5]},
    options: {
      windowDimensions: [2, 2],
      strides: [2, 2],
    },
    output: {dataType: 'float16', shape: [1, 3, 2, 2]}
  },
  {
    name: 'Test pool2d with strides and padding.',
    input: {dataType: 'float32', shape: [1, 3, 5, 5]},
    options: {
      windowDimensions: [3, 3],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
    },
    output: {dataType: 'float32', shape: [1, 3, 3, 3]}
  },
  {
    name: 'Test pool2d with strides and asymmetric padding.',
    input: {dataType: 'float32', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [2, 1, 2, 1],
      strides: [2, 2],
    },
    output: {dataType: 'float32', shape: [1, 3, 4, 4]}
  },
  {
    name: 'Test pool2d with strides, padding and roundingType="floor".',
    input: {dataType: 'float32', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      roundingType: 'floor',
    },
    output: {dataType: 'float32', shape: [1, 3, 3, 3]}
  },
  {
    name: 'Test pool2d with strides, padding and roundingType="ceil".',
    input: {dataType: 'float16', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      roundingType: 'ceil',
    },
    output: {dataType: 'float16', shape: [1, 3, 4, 4]}
  },
  {
    name: 'Test pool2d with explicit outputSizes ignored roundingType',
    input: {dataType: 'float32', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      roundingType: 'ceil',
      outputSizes: [3, 3],
    },
    output: {dataType: 'float32', shape: [1, 3, 3, 3]}
  },
  {
    name: 'Test pool2d with strides, padding and outputSizes=[3, 3].',
    input: {dataType: 'float32', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      outputSizes: [3, 3],
    },
    output: {dataType: 'float32', shape: [1, 3, 3, 3]}
  },
  {
    name: 'Test pool2d with strides, padding and outputSizes=[4, 4].',
    input: {dataType: 'float32', shape: [1, 3, 7, 7]},
    options: {
      windowDimensions: [4, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      outputSizes: [4, 4],
    },
    output: {dataType: 'float32', shape: [1, 3, 4, 4]}
  },
  {
    name: 'Test pool2d with layout="nchw".',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [3, 3],
      layout: 'nchw',
    },
    output: {dataType: 'float32', shape: [1, 2, 3, 3]}
  },
  {
    name: 'Test pool2d with layout="nhwc".',
    input: {dataType: 'float16', shape: [1, 5, 5, 2]},
    options: {
      windowDimensions: [3, 3],
      layout: 'nhwc',
    },
    output: {dataType: 'float16', shape: [1, 3, 3, 2]}
  },
  {
    name: 'Throw if the input is not a 4-D tensor.',
    input: {dataType: 'float32', shape: [1, 5, 5]},
    options: {label},
  },
  {
    name: 'Throw if the output sizes is incorrect.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [2, 2],
      padding: [2, 2, 2, 2],
      strides: [2, 2],
      outputSizes: [3, 3],
      label: label,
    },
  },
  {
    name: 'Throw if the length of output sizes is not 2.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [2, 2],
      padding: [2, 2, 2, 2],
      strides: [2, 2],
      outputSizes: [1, 2, 4, 4],
      label: label,
    },
  },
  {
    name: 'Throw if outputSizes[0] is not greater than 0.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [2, 2],
      padding: [2, 2, 2, 2],
      strides: [2, 2],
      outputSizes: [0, 4],
      label: label,
    },
  },
  {
    name: 'Throw if outputSizes[1] is not greater than 0.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [2, 2],
      padding: [2, 2, 2, 2],
      strides: [2, 2],
      outputSizes: [4, 0],
      label: label,
    },
  },
  {
    name: 'Throw if the length of window dimensions is not 2.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [1, 1, 1, 1],
      label: label,
    },
  },
  {
    name: 'Throw if any window dimension is lesser than 1.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [0, 2],
      label: label,
    },
  },
  {
    name:
        'Throw if the input height is too small to fill the pool window height.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [8, 2],
      label: label,
    },
  },
  {
    name:
        'Throw if the input width is too small to fill the pool window width.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [2, 8],
      label: label,
    },
  },
  {
    name: 'Throw if the calculated output height is equal to 0.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [6, 3],
      label: label,
    },
  },
  {
    name: 'Throw if the calculated output width is equal to 0.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      windowDimensions: [3, 6],
      label: label,
    },
  },
  {
    name: 'Throw if the length of padding is not 4.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      padding: [2, 2],
      label: label,
    },
  },
  {
    name: 'Throw if the length of strides is not 2.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      strides: [2],
      label: label,
    },
  },
  {
    name: 'Throw if one stride value is smaller than 1.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      strides: [0, 2],
      label: label,
    },
  },
  {
    name: 'Throw if the length of dilations is not 2.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      dilations: [1, 1, 2],
      label: label,
    },
  },
  {
    name: 'Throw if one dilation value is smaller than 1.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    options: {
      dilations: [1, 0],
      label: label,
    },
  },
  {
    name: 'Throw if the padding height value is too large',
    input: {dataType: 'float32', shape: [1, 3, 5, 5]},
    options: {
      padding: [kMaxUnsignedLong, kMaxUnsignedLong, 0, 0],
      label: label,
    },
  },
  {
    name: 'Throw if the padding width value is too large',
    input: {dataType: 'float32', shape: [1, 3, 5, 5]},
    options: {
      padding: [0, 0, kMaxUnsignedLong, kMaxUnsignedLong],
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      kPoolingOperators.forEach((operatorName) => {
        if (test.output) {
          const output = builder[operatorName](input, test.options);
          assert_equals(output.dataType, test.output.dataType);
          assert_array_equals(output.shape, test.output.shape);
        } else {
          const regrexp = new RegExp('\\[' + label + '\\]');
          assert_throws_with_label(
              () => builder[operatorName](input, test.options), regrexp);
        }
      });
    }, test.name));

['int32', 'uint32', 'int8', 'uint8'].forEach(
    dataType => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', {dataType, shape: [1, 3, 4, 4]});
      const output = builder.maxPool2d(input);
      assert_equals(output.dataType, dataType);
      assert_array_equals(output.shape, [1, 3, 1, 1]);
    }, `[maxPool2d] Test maxPool2d with data type ${dataType}`));

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const input =
      builder.input('input', {dataType: 'int64', shape: [1, 2, 3, 3]});
  assert_throws_js(TypeError, () => builder.averagePool2d(input));
}, '[averagePool2d] Throw if the input data type is not floating point');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const input =
      builder.input('input', {dataType: 'uint8', shape: [1, 2, 4, 4]});
  assert_throws_js(TypeError, () => builder.l2Pool2d(input));
}, '[l2Pool2d] Throw if the input data type is not floating point');
