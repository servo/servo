// META: title=validation tests for WebNN API instanceNormalization operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2, 2, 2]
};
// 1D tensor descriptor which may be used for `scale`, or `bias` inputs.
const kExample1DTensorDescriptor = {
  dataType: 'float32',
  dimensions: [2]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  assert_throws_js(
      TypeError, () => builder.instanceNormalization(inputFromOtherBuilder));
}, '[instanceNormalization] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExample1DTensorDescriptor);
  const options = {scale: scaleFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.instanceNormalization(input, options));
}, '[instanceNormalization] throw if scale option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExample1DTensorDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(
      TypeError, () => builder.instanceNormalization(input, options));
}, '[instanceNormalization] throw if bias option is from another builder');

const label = 'instance_normalization';
const tests = [
  {
    name: '[instanceNormalization] Test with default options for 4-D input.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]}
  },
  {
    name:
        '[instanceNormalization] Test with scale, bias and default epsilon value.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {dataType: 'float32', dimensions: [2]},
      bias: {dataType: 'float32', dimensions: [2]},
      epsilon: 1e-5,
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]}
  },
  {
    name: '[instanceNormalization] Test with a non-default epsilon value.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      epsilon: 1e-4,
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]}
  },
  {
    name: '[instanceNormalization] Test with layout=nhwc.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      layout: 'nhwc',
      scale: {dataType: 'float32', dimensions: [4]},
      bias: {dataType: 'float32', dimensions: [4]},
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]}
  },
  {
    name: '[instanceNormalization] Test when the input data type is float16.',
    input: {dataType: 'float16', dimensions: [1, 2, 3, 4]},
    output: {dataType: 'float16', dimensions: [1, 2, 3, 4]},
    options: {label}
  },
  {
    name: '[instanceNormalization] Throw if the input is not a 4-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5, 2]},
    options: {label}
  },
  {
    name:
        '[instanceNormalization] Throw if the input data type is not one of floating point types.',
    input: {dataType: 'int32', dimensions: [1, 2, 5, 5]},
    options: {label}
  },
  {
    name:
        '[instanceNormalization] Throw if the scale data type is not the same as the input data type.',
    input: {dataType: 'float16', dimensions: [1, 2, 5, 5]},
    options: {
      scale: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the scale operand is not a 1-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      scale: {dataType: 'float32', dimensions: [2, 1]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the size of scale operand is not equal to the size of the feature dimension of the input with layout=nhwc.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      layout: 'nhwc',
      scale: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the size of scale operand is not equal to the size of the feature dimension of the input with layout=nchw.',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    options: {
      layout: 'nchw',
      scale: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the bias data type is not the same as the input data type.',
    input: {dataType: 'float16', dimensions: [1, 2, 5, 5]},
    options: {
      bias: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the bias operand is not a 1-D tensor.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      scale: {dataType: 'float32', dimensions: [2, 1]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the size of bias operand is not equal to the size of the feature dimension of the input with layout=nhwc.',
    input: {dataType: 'float32', dimensions: [1, 2, 5, 5]},
    options: {
      layout: 'nhwc',
      bias: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
  {
    name:
        '[instanceNormalization] Throw if the size of bias operand is not equal to the size of the feature dimension of the input with layout=nchw.',
    input: {dataType: 'float32', dimensions: [1, 5, 5, 2]},
    options: {
      layout: 'nchw',
      bias: {dataType: 'float32', dimensions: [2]},
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});

      if (test.options && test.options.bias) {
        test.options.bias = builder.input('bias', {
          dataType: test.options.bias.dataType,
          dimensions: test.options.bias.dimensions
        });
      }
      if (test.options && test.options.scale) {
        test.options.scale = builder.input('scale', {
          dataType: test.options.scale.dataType,
          dimensions: test.options.scale.dimensions
        });
      }

      if (test.output &&
          context.opSupportLimits()
              .instanceNormalization.input.dataTypes.includes(
                  test.input.dataType)) {
        const output = builder.instanceNormalization(input, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder.instanceNormalization(input, test.options), regrexp);
      }
    }, test.name));
