// META: title=validation tests for WebNN API conv2d operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

// Example input in NCHW layout.
const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [1, 1, 5, 5]
};
// Example filter in OIHW layout.
const kExampleFilterDescriptor = {
  dataType: 'float32',
  dimensions: [1, 1, 3, 3]
};
const kExampleBiasDescriptor = {
  dataType: 'float32',
  dimensions: [/* output channels */ 1]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(
      TypeError, () => builder.conv2d(inputFromOtherBuilder, filter));
}, '[conv2d] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const filterFromOtherBuilder =
      otherBuilder.input('filter', kExampleFilterDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.conv2d(input, filterFromOtherBuilder));
}, '[conv2d] throw if filter is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleBiasDescriptor);
  const options = {inputLayout: 'nchw', bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(TypeError, () => builder.conv2d(input, filter, options));
}, '[conv2d] throw if bias option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const activationFromOtherBuilder = otherBuilder.clamp();
  const options = {activation: activationFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const filter = builder.input('filter', kExampleFilterDescriptor);
  assert_throws_js(TypeError, () => builder.conv2d(input, filter, options));
}, '[conv2d] throw if activation option is from another builder');

const tests = [
  {
    name: '[conv2d] Test with default options.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 3, 3]},
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with padding.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
    },
    output: {dataType: 'float32', dimensions: [1, 1, 5, 5]}
  },
  {
    name: '[conv2d] Test with strides and padding.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 3, 3]},
    options: {
      padding: [1, 1, 1, 1],
      strides: [2, 2],
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with strides and asymmetric padding.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 4, 2]},
    options: {
      padding: [1, 2, 0, 1],
      strides: [2, 2],
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test depthwise conv2d by setting groups to input channels.',
    input: {dataType: 'float32', dimensions: [1, 4, 2, 2]},
    filter: {dataType: 'float32', dimensions: [4, 1, 2, 2]},
    options: {
      groups: 4,
    },
    output: {dataType: 'float32', dimensions: [1, 4, 1, 1]}
  },
  {
    name:
        '[conv2d] Test depthwise conv2d with groups, inputLayout="nhwc" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 2, 2, 4]},
    filter: {dataType: 'float32', dimensions: [1, 2, 2, 4]},
    options: {
      groups: 4,
      inputLayout: 'nhwc',
      filterLayout: 'ihwo',
    },
    output: {dataType: 'float32', dimensions: [1, 1, 1, 4]}
  },
  {
    name:
        '[conv2d] Test with dilations, inputLayout="nhwc" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 65, 65, 1]},
    filter: {dataType: 'float32', dimensions: [1, 3, 3, 1]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'ihwo',
      dilations: [4, 4],
    },
    output: {dataType: 'float32', dimensions: [1, 57, 57, 1]}
  },
  {
    name: '[conv2d] Test with inputLayout="nchw" and filterLayout="oihw".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 2, 3, 3]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'oihw',
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with inputLayout="nchw" and filterLayout="hwio".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [3, 3, 2, 1]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'hwio',
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with inputLayout="nchw" and filterLayout="ohwi".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 3, 3, 2]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'ohwi',
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with inputLayout="nchw" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [2, 3, 3, 1]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'ihwo',
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 3]}
  },
  {
    name: '[conv2d] Test with inputLayout="nhwc" and filterLayout="oihw".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [1, 2, 3, 3]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'oihw',
    },
    output: {dataType: 'float32', dimensions: [1, 3, 3, 1]}
  },
  {
    name: '[conv2d] Test with inputLayout="nhwc" and filterLayout="hwio".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [3, 3, 2, 1]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'hwio',
    },
    output: {dataType: 'float32', dimensions: [1, 3, 3, 1]}
  },
  {
    name: '[conv2d] Test with inputLayout="nhwc" and filterLayout="ohwi".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [1, 3, 3, 2]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'ohwi',
    },
    output: {dataType: 'float32', dimensions: [1, 3, 3, 1]}
  },
  {
    name: '[conv2d] Test with inputLayout="nhwc" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [2, 3, 3, 1]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'ihwo',
    },
    output: {dataType: 'float32', dimensions: [1, 3, 3, 1]}
  },
  {
    name: '[conv2d] Throw if the input is not a 4-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 2, 2, 1]},
  },
  {
    name: '[conv2d] Throw if the input data type is not floating point.',
    input: {dataType: 'int32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'int32', dimensions: [1, 1, 2, 2]},
  },
  {
    name: '[conv2d] Throw if the filter is not a 4-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [2, 2]},
  },
  {
    name:
        '[conv2d] Throw if the filter data type doesn\'t match the input data type.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'int32', dimensions: [1, 1, 2, 2]},
  },
  {
    name: '[conv2d] Throw if the length of padding is not 4.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      padding: [2, 2],
    },
  },
  {
    name: '[conv2d] Throw if the length of strides is not 2.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      strides: [2],
    },
  },
  {
    name: '[conv2d] Throw if strideHeight is smaller than 1.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      strides: [0, 1],
    },
  },
  {
    name: '[conv2d] Throw if strideWidth is smaller than 1.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      strides: [1, 0],
    },
  },
  {
    name: '[conv2d] Throw if the length of dilations is not 2.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      dilations: [1],
    },
  },
  {
    name: '[conv2d] Throw if dilationHeight is smaller than 1.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      dilations: [0, 1],
    },
  },
  {
    name: '[conv2d] Throw if dilationWidth is smaller than 1.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      dilations: [1, 0],
    },
  },
  {
    name: '[conv2d] Throw if inputChannels % groups is not 0.',
    input: {dataType: 'float32', dimensions: [1, 4, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      groups: 3,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels.',
    input: {dataType: 'float32', dimensions: [1, 4, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      groups: 2,
    },
  },
  {
    name: '[conv2d] Throw if the groups is smaller than 1.',
    input: {dataType: 'float32', dimensions: [1, 4, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      groups: 0,
    },
  },
  {
    name:
        '[conv2d] Throw due to overflow when calculating the effective filter height.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 434983, 2]},
    options: {
      dilations: [328442, 1],
    },
  },
  {
    name:
        '[conv2d] Throw due to overflow when calculating the effective filter width.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 234545]},
    options: {
      dilations: [2, 843452],
    },
  },
  {
    name: '[conv2d] Throw due to overflow when dilation height is too large.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 3, 3]},
    options: {
      dilations: [kMaxUnsignedLong, 1],
    },
  },
  {
    name: '[conv2d] Throw due to overflow when dilation width is too large.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 3, 3]},
    options: {
      dilations: [1, kMaxUnsignedLong],
    },
  },
  {
    name: '[conv2d] Throw due to underflow when calculating the output height.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 4, 2]},
    options: {
      dilations: [4, 1],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
    },
  },
  {
    name: '[conv2d] Throw due to underflow when calculating the output width.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 8]},
    options: {
      dilations: [1, 4],
      padding: [1, 1, 1, 1],
      strides: [2, 2],
    },
  },
  {
    name: '[conv2d] Throw if the bias is not a 1-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      bias: {dataType: 'float32', dimensions: [1, 2]},
    },
  },
  {
    name:
        '[conv2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="oihw".',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      bias: {dataType: 'float32', dimensions: [2]},
    },
  },
  {
    name:
        '[conv2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="hwio".',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [2, 2, 1, 1]},
    options: {
      bias: {dataType: 'float32', dimensions: [2]},
    },
  },
  {
    name:
        '[conv2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="ohwi".',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 2, 2, 1]},
    options: {
      bias: {dataType: 'float32', dimensions: [2]},
    },
  },
  {
    name:
        '[conv2d] Throw if the bias shape is not equal to [output_channels] with filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 2, 2, 1]},
    options: {
      bias: {dataType: 'float32', dimensions: [2]},
    },
  },
  {
    name:
        '[conv2d] Throw if the bias data type doesn\'t match input data type.',
    input: {dataType: 'float32', dimensions: [1, 1, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 1, 2, 2]},
    options: {
      bias: {dataType: 'int32', dimensions: [1]},
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nchw" and filterLayout="oihw".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 2, 3, 3]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'oihw',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nchw" and filterLayout="hwio".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [3, 3, 2, 1]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'hwio',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nchw" and filterLayout="ohwi".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [1, 3, 3, 2]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'ohwi',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nchw" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    filter: {dataType: 'float32', dimensions: [2, 3, 3, 1]},
    options: {
      inputLayout: 'nchw',
      filterLayout: 'ihwo',
      groups: 2,
    },

  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nhwc" and filterLayout="oihw".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [1, 2, 3, 3]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'oihw',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nhwc" and filterLayout="hwio".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [3, 3, 2, 1]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'hwio',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nhwc" and filterLayout="ohwi".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [1, 3, 3, 2]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'ohwi',
      groups: 2,
    },
  },
  {
    name:
        '[conv2d] Throw if inputChannels / groups is not equal to filterInputChannels with inputLayout="nhwc" and filterLayout="ihwo".',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    filter: {dataType: 'float32', dimensions: [2, 3, 3, 1]},
    options: {
      inputLayout: 'nhwc',
      filterLayout: 'ihwo',
      groups: 2,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});
      const filter = builder.input(
          'filter',
          {dataType: test.filter.dataType, dimensions: test.filter.dimensions});

      if (test.options && test.options.bias) {
        test.options.bias = builder.input('bias', {
          dataType: test.options.bias.dataType,
          dimensions: test.options.bias.dimensions
        });
      }

      if (test.output) {
        const output = builder.conv2d(input, filter, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.conv2d(input, filter, test.options));
      }
    }, test.name));
