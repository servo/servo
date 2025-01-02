// META: title=validation tests for WebNN API convTranspose2d operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

// Example input in NCHW layout.
const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: [1, 1, 5, 5]
};
// Example filter in OIHW layout.
const kExampleFilterDescriptor = {
  dataType: 'float32',
  shape: [1, 1, 3, 3]
};
const kExampleBiasDescriptor = {
  dataType: 'float32',
  shape: [/* output channels */ 1]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(
      TypeError, () => builder.convTranspose2d(inputFromOtherBuilder, filter));
}, '[convTranspose2d] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const filterFromOtherBuilder =
      otherBuilder.input('filter', kExampleFilterDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.convTranspose2d(input, filterFromOtherBuilder));
}, '[convTranspose2d] throw if filter is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {inputLayout: 'nchw', bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(
      TypeError, () => builder.convTranspose2d(input, filter, options));
}, '[convTranspose2d] throw if bias option is from another builder');

const label = 'conv_transpose_2d';
const tests = [
  {
    name: '[convTranspose2d] Test with default options.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    output: {dataType: 'float32', shape: [1, 1, 5, 5]}
  },
  {
    name:
        '[convTranspose2d] Test with inputLayout="nchw" and filterLayout="hwoi".',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [3, 3, 2, 1]},
    options: {
      filterLayout: 'hwoi',
      inputLayout: 'nchw',
    },
    output: {dataType: 'float32', shape: [1, 2, 5, 5]}
  },
  {
    name:
        '[convTranspose2d] Test with inputLayout="nchw" and filterLayout="ohwi".',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [2, 3, 3, 1]},
    options: {
      filterLayout: 'ohwi',
      inputLayout: 'nchw',
    },
    output: {dataType: 'float32', shape: [1, 2, 5, 5]}
  },
  {
    name:
        '[convTranspose2d] Test with inputLayout="nhwc" and filterLayout="iohw".',
    input: {dataType: 'float32', shape: [1, 3, 3, 1]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      filterLayout: 'iohw',
      inputLayout: 'nhwc',
    },
    output: {dataType: 'float32', shape: [1, 5, 5, 2]}
  },
  {
    name:
        '[convTranspose2d] Test with inputLayout="nhwc" and filterLayout="hwoi".',
    input: {dataType: 'float32', shape: [1, 3, 3, 1]},
    filter: {dataType: 'float32', shape: [3, 3, 2, 1]},
    options: {
      filterLayout: 'hwoi',
      inputLayout: 'nhwc',
    },
    output: {dataType: 'float32', shape: [1, 5, 5, 2]}
  },
  {
    name:
        '[convTranspose2d] Test with inputLayout="nhwc" and filterLayout="ohwi".',
    input: {dataType: 'float32', shape: [1, 3, 3, 1]},
    filter: {dataType: 'float32', shape: [2, 3, 3, 1]},
    options: {
      filterLayout: 'ohwi',
      inputLayout: 'nhwc',
    },
    output: {dataType: 'float32', shape: [1, 5, 5, 2]}
  },
  {
    name: '[convTranspose2d] Test with strides=[3, 2], outputSizes=[10, 8].',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputSizes: [10, 8],
    },
    output: {dataType: 'float32', shape: [1, 2, 10, 8]}
  },
  {
    name: '[convTranspose2d] Test with strides=[3, 2], outputPadding=[1, 1].',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputPadding: [1, 1],
    },
    output: {dataType: 'float32', shape: [1, 2, 10, 8]}
  },
  {
    name: '[convTranspose2d] Test with padding=1.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
    },
    output: {dataType: 'float32', shape: [1, 1, 5, 5]}
  },
  {
    name: '[convTranspose2d] Test with padding=1, groups=3.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
      groups: 3,
    },
    output: {dataType: 'float32', shape: [1, 3, 5, 5]}
  },
  {
    name: '[convTranspose2d] Test with strides=2.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [2, 2],
    },
    output: {dataType: 'float32', shape: [1, 2, 7, 7]}
  },
  {
    name: '[convTranspose2d] Test with strides=2 and padding=1.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
      strides: [2, 2],
    },
    output: {dataType: 'float32', shape: [1, 1, 5, 5]}
  },
  {
    name:
        '[convTranspose2d] Test when the output sizes are explicitly specified, the output padding values are ignored though padding value is not smaller than stride along the same axis.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      outputPadding: [3, 3],
      strides: [3, 2],
      outputSizes: [10, 8],
    },
    output: {dataType: 'float32', shape: [1, 2, 10, 8]}
  },
  {
    name: '[convTranspose2d] Throw if the input is not a 4-D tensor.',
    input: {dataType: 'float32', shape: [1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {label},
  },
  {
    name:
        '[convTranspose2d] Throw if the input data type is not floating point.',
    input: {dataType: 'int32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'int32', shape: [1, 1, 2, 2]},
    options: {label},
  },
  {
    name: '[convTranspose2d] Throw if the filter is not a 4-D tensor.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [2, 2]},
    options: {label},
  },
  {
    name:
        '[convTranspose2d] Throw if the filter data type doesn\'t match the input data type.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'int32', shape: [1, 1, 2, 2]},
    options: {
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the length of padding is not 4.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      padding: [2, 2],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the length of strides is not 2.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      strides: [2],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if one stride value is smaller than 1.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      strides: [1, 0],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the length of dilations is not 2.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      dilations: [1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the one dilation value is smaller than 1.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      dilations: [1, 0],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels with inputLayout="nchw" and filterLayout="iohw".',
    input: {dataType: 'float32', shape: [1, 3, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      filterLayout: 'iohw',
      inputLayout: 'nchw',
      groups: 1,
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels with inputLayout="nchw" and filterLayout="hwoi".',
    input: {dataType: 'float32', shape: [1, 3, 3, 3]},
    filter: {dataType: 'float32', shape: [3, 1, 2, 1]},
    options: {
      filterLayout: 'hwoi',
      inputLayout: 'nchw',
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels with inputLayout="nchw" and filterLayout="ohwi".',
    input: {dataType: 'float32', shape: [1, 2, 3, 3]},
    filter: {dataType: 'float32', shape: [2, 3, 3, 1]},
    options: {
      filterLayout: 'ohwi',
      inputLayout: 'nchw',
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels with inputLayout="nhwc" and filterLayout="iohw".',
    input: {dataType: 'float32', shape: [1, 3, 3, 2]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      filterLayout: 'iohw',
      inputLayout: 'nhwc',
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels inputLayout="nhwc" and filterLayout="hwoi".',
    input: {dataType: 'float32', shape: [1, 3, 3, 2]},
    filter: {dataType: 'float32', shape: [3, 3, 2, 1]},
    options: {
      filterLayout: 'hwoi',
      inputLayout: 'nhwc',
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the input channels is not equal to the filter input channels with inputLayout="nhwc" and filterLayout="ohwi".',
    input: {dataType: 'float32', shape: [1, 3, 3, 2]},
    filter: {dataType: 'float32', shape: [2, 3, 3, 1]},
    options: {
      filterLayout: 'ohwi',
      inputLayout: 'nhwc',
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if output channels is too large.',
    input: {dataType: 'float32', shape: [1, 4, 5, 5]},
    filter: {dataType: 'float32', shape: [4, 2, 2, 2]},
    options: {
      groups: kMaxUnsignedLong,
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the groups is smaller than 1.',
    input: {dataType: 'float32', shape: [1, 4, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      groups: 0,
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to overflow when calculating the effective filter height.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 434983, 2]},
    options: {
      dilations: [328443, 1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to overflow when calculating the effective filter width.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 234545]},
    options: {
      dilations: [2, 843452],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to overflow when dilation height is too large.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 2]},
    options: {
      dilations: [kMaxUnsignedLong, 1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to overflow when dilation width is too large.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 2]},
    options: {
      dilations: [1, kMaxUnsignedLong],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the bias is not a 1-D tensor.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      bias: {dataType: 'float32', shape: [1, 2]},
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="iohw".',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      filterLayout: 'iohw',
      bias: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="hwoi".',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [2, 2, 1, 1]},
    options: {
      filterLayout: 'hwoi',
      bias: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="ohwi".',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 2, 2, 1]},
    options: {
      filterLayout: 'ohwi',
      bias: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the bias data type doesn\'t match input data type.',
    input: {dataType: 'float32', shape: [1, 1, 5, 5]},
    filter: {dataType: 'float32', shape: [1, 1, 2, 2]},
    options: {
      bias: {dataType: 'int32', shape: [1]},
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the outputPadding is not a sequence of length 2.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputPadding: [1, 1, 1, 1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the outputPadding is not smaller than stride along the width dimension.',
    input: {dataType: 'float32', shape: [1, 1, 2, 2]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [0, 0, 3, 3],
      strides: [2, 2],
      outputPadding: [0, 2],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the outputPadding is not smaller than stride along the height dimension.',
    input: {dataType: 'float32', shape: [1, 1, 2, 2]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [0, 0, 3, 3],
      strides: [2, 2],
      outputPadding: [2, 0],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw if the outputSizes is not a sequence of length 2.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputSizes: [1, 2, 10, 8],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if outputSizes[0] is not greater than 0.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputSizes: [0, 7],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if outputSizes[1] is not greater than 0.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 2, 3, 3]},
    options: {
      strides: [3, 2],
      outputSizes: [9, 0],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the padding height is too large.',
    input: {dataType: 'float32', shape: [1, 1, 2, 2]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [4, 4, 0, 0],
      strides: [2, 2],
      outputPadding: [1, 0],
      label: label,
    },
  },
  {
    name: '[convTranspose2d] Throw if the padding width is too large.',
    input: {dataType: 'float32', shape: [1, 1, 2, 2]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [0, 0, 4, 4],
      strides: [2, 2],
      outputPadding: [0, 1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to outputSizes values are smaller than the output sizes calculated by not using outputPadding.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      outputSizes: [4, 4],
      outputPadding: [1, 1],
      label: label,
    },
  },
  {
    name:
        '[convTranspose2d] Throw due to outputSizes values are greater than the output sizes calculated by not using outputPadding.',
    input: {dataType: 'float32', shape: [1, 1, 3, 3]},
    filter: {dataType: 'float32', shape: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
      strides: [2, 2],
      outputSizes: [6, 8],
      outputPadding: [1, 1],
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const filter = builder.input('filter', test.filter);

      if (test.options && test.options.bias) {
        test.options.bias = builder.input('bias', test.options.bias);
      }

      if (test.output &&
          context.opSupportLimits().convTranspose2d.input.dataTypes.includes(
              test.input.dataType)) {
        const output = builder.convTranspose2d(input, filter, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.convTranspose2d(input, filter, test.options),
            regrexp);
      }
    }, test.name));
