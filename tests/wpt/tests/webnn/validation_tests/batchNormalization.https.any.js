// META: title=validation tests for WebNN API batchNormalization operation
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

let meanIndex = 0;
let varianceIndex = 0;

const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: [2, 2]
};
// 1D tensor descriptor which may be used for `mean`, `variance`, `scale`, or
// `bias` inputs.
const kExample1DTensorDescriptor = {
  dataType: 'float32',
  shape: [kExampleInputDescriptor.shape[/* axis */ 1]]
};

multi_builder_test(async (t, builder, otherBuilder) => {
  const inputFromOtherBuilder =
      otherBuilder.input('input', kExampleInputDescriptor);

  const mean = builder.input('mean', kExample1DTensorDescriptor);
  const variance = builder.input('variance', kExample1DTensorDescriptor);
  assert_throws_js(
      TypeError, () => builder.batchNormalization(inputFromOtherBuilder, mean, variance));
}, '[batchNormalization] throw if input is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const meanFromOtherBuilder =
      otherBuilder.input('mean', kExample1DTensorDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const variance = builder.input('variance', kExample1DTensorDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.batchNormalization(input, meanFromOtherBuilder, variance));
}, '[batchNormalization] throw if mean is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const varianceFromOtherBuilder =
      otherBuilder.input('variance', kExample1DTensorDescriptor);

  const input = builder.input('input', kExampleInputDescriptor);
  const mean = builder.input('mean', kExample1DTensorDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.batchNormalization(input, mean, varianceFromOtherBuilder));
}, '[batchNormalization] throw if variance is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExample1DTensorDescriptor);
  const options = {scale: scaleFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const mean = builder.input('mean', kExample1DTensorDescriptor);
  const variance = builder.input('variance', kExample1DTensorDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.batchNormalization(input, mean, variance, options));
}, '[batchNormalization] throw if scale option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExample1DTensorDescriptor);
  const options = {scale: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  const mean = builder.input('mean', kExample1DTensorDescriptor);
  const variance = builder.input('variance', kExample1DTensorDescriptor);
  assert_throws_js(
      TypeError,
      () => builder.batchNormalization(input, mean, variance, options));
}, '[batchNormalization] throw if bias option is from another builder');

const label = `batchNormalization_?_123`;
const tests = [
  {
    name: '[batchNormalization] Test with default options.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [2]},
    variance: {dataType: 'float32', shape: [2]},
    output: {dataType: 'float32', shape: [1, 2, 5, 5]},
  },
  {
    name: '[batchNormalization] Test with axis = 2 and epsilon = 0.0001.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [5]},
    variance: {dataType: 'float32', shape: [5]},
    options: {
      axis: 2,
      epsilon: 1e-4,  // 1e-5 is the default value of epsilon.
    },
    output: {dataType: 'float32', shape: [1, 2, 5, 5]},
  },
  {
    name:
        '[batchNormalization] Throw if the input data type is not one of floating point types.',
    input: {dataType: 'int32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'int32', shape: [2]},
    variance: {dataType: 'int32', shape: [2]},
    options: {
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the mean data type is not the same as the input data type.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float16', shape: [2]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      label: label,
    },
  },
  {
    name: '[batchNormalization] Throw if the mean operand is not a 1-D tensor.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [1, 2]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the size of mean operand is not equal to the size of the input dimension denoted by axis.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [3]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      axis: 1,
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the variance data type is not the same as the input data type.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [2]},
    variance: {dataType: 'float16', shape: [2]},
    options: {
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the variance operand is not a 1-D tensor.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [2]},
    variance: {dataType: 'float32', shape: [2, 2]},
    options: {
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the size of variance operand is not equal to the size of the input dimension denoted by axis.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [5]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      axis: 2,
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the scale data type is not the same as the input data type.',
    input: {dataType: 'float16', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float16', shape: [2]},
    variance: {dataType: 'float16', shape: [2]},
    options: {
      scale: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the scale operand is not a 1-D tensor.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [2]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      scale: {dataType: 'float32', shape: [2, 1]},
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the size of scale operand is not equal to the size of the input dimension denoted by axis.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [5]},
    variance: {dataType: 'float32', shape: [5]},
    options: {
      axis: 2,
      scale: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the bias data type is not the same as the input data type.',
    input: {dataType: 'float16', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float16', shape: [2]},
    variance: {dataType: 'float16', shape: [2]},
    options: {
      bias: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name: '[batchNormalization] Throw if the bias operand is not a 1-D tensor.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [2]},
    variance: {dataType: 'float32', shape: [2]},
    options: {
      bias: {dataType: 'float32', shape: [2, 1]},
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the size of bias operand is not equal to the size of the input dimension denoted by axis.',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [5]},
    variance: {dataType: 'float32', shape: [5]},
    options: {
      axis: 2,
      bias: {dataType: 'float32', shape: [2]},
      label: label,
    },
  },
  {
    name:
        '[batchNormalization] Throw if the value of axis is not in the range of [0,N-1].',
    input: {dataType: 'float32', shape: [1, 2, 5, 5]},
    mean: {dataType: 'float32', shape: [5]},
    variance: {dataType: 'float32', shape: [5]},
    options: {
      axis: 4,
      label: label,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const builder = new MLGraphBuilder(context);
      const input = builder.input('input', test.input);
      const mean = builder.input('mean', test.mean);
      const variance = builder.input('variance', test.variance);

      if (test.options && test.options.bias) {
        test.options.bias = builder.input('bias', test.options.bias);
      }
      if (test.options && test.options.scale) {
        test.options.scale = builder.input('scale', test.options.scale);
      }

      if (test.output) {
        const output =
            builder.batchNormalization(input, mean, variance, test.options);
        assert_equals(output.dataType, test.output.dataType);
        assert_array_equals(output.shape, test.output.shape);
      } else {
        const regrexp = /\[batchNormalization_\?_123\]/;
        assert_throws_with_label(
            () =>
                builder.batchNormalization(input, mean, variance, test.options),
            regrexp);
      }
    }, test.name));
