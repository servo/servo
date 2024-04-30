// META: title=validation tests for WebNN API layerNormalization operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2]
};

validateInputFromAnotherBuilder('layerNormalization');

multi_builder_test(async (t, builder, otherBuilder) => {
  const scaleFromOtherBuilder =
      otherBuilder.input('scale', kExampleInputDescriptor);
  const options = {scale: scaleFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.layerNormalization(input, options));
}, '[layerNormalization] throw if scale option is from another builder');

multi_builder_test(async (t, builder, otherBuilder) => {
  const biasFromOtherBuilder =
      otherBuilder.input('bias', kExampleInputDescriptor);
  const options = {bias: biasFromOtherBuilder};

  const input = builder.input('input', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.layerNormalization(input, options));
}, '[layerNormalization] throw if bias option is from another builder');

const tests = [
  {
    name: '[layerNormalization] Test with default options for scalar input.',
    input: {dataType: 'float32', dimensions: []},
    output: {dataType: 'float32', dimensions: []},
  },
  {
    name: '[layerNormalization] Test when the input data type is float16.',
    input: {dataType: 'float16', dimensions: []},
    output: {dataType: 'float16', dimensions: []},
  },
  {
    name: '[layerNormalization] Test with given axes.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [3],
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
  },
  {
    name: '[layerNormalization] Test with given scale.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {dataType: 'float32', dimensions: [2, 3, 4]},
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
  },
  {
    name: '[layerNormalization] Test with a non-default epsilon value.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      epsilon: 1e-4,  // default epsilon=1e-5
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
  },
  {
    name: '[layerNormalization] Test with given axes, scale and bias.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {dataType: 'float32', dimensions: [3, 4]},
      bias: {dataType: 'float32', dimensions: [3, 4]},
      axes: [2, 3],
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
  },
  {
    name: '[layerNormalization] Test with nonconsecutive axes.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4, 5, 6]},
    options: {
      scale: {dataType: 'float32', dimensions: [2, 4, 6]},
      bias: {dataType: 'float32', dimensions: [2, 4, 6]},
      axes: [1, 3, 5],
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4, 5, 6]},
  },
  {
    name: '[layerNormalization] Test with axes in descending order.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4, 5, 6]},
    options: {
      scale: {dataType: 'float32', dimensions: [6, 5, 4, 3, 2]},
      bias: {dataType: 'float32', dimensions: [6, 5, 4, 3, 2]},
      axes: [5, 4, 3, 2, 1]
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4, 5, 6]},
  },
  {
    name:
        '[layerNormalization] Throw if the input data type is not one of the floating point types.',
    input: {dataType: 'uint32', dimensions: [1, 2, 3, 4]},
  },
  {
    name:
        '[layerNormalization] Throw if the axis is greater than the input rank.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1, 2, 4],
    },
  },
  {
    name: '[layerNormalization] Throw if the axes have duplications.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {axes: [3, 3]},
  },
  {
    name:
        '[layerNormalization] Throw if the bias data type doesn\'t match input data type',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {dataType: 'float32', dimensions: [3, 4]},
      bias: {dataType: 'float16', dimensions: [3, 4]},
      axes: [2, 3],
    },
  },
  {
    name:
        '[layerNormalization] Throw if the scale data type doesn\'t match input data type',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {dataType: 'float16', dimensions: [3, 4]},
      bias: {dataType: 'float32', dimensions: [3, 4]},
      axes: [2, 3],
    },
  },
  {
    name:
        '[layerNormalization] Throw if the bias dimensions doesn\'t match axis dimensions.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      bias: {
        dataType: 'float32',
        dimensions: [3, 3, 4]
      },  // for 4D input, default axes = [1,2,3]
    },
  },
  {
    name:
        '[layerNormalization] Throw if the scale dimensions doesn\'t match axis dimensions.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {
        dataType: 'float32',
        dimensions: [3, 3, 4]
      },  // for 4D input, default axes = [1,2,3]
    },
  },
  {
    name:
        '[layerNormalization] Throw if the bias rank doesn\'t match axis rank.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      bias: {
        dataType: 'float32',
        dimensions: [1, 2, 3, 4]
      },  // for 4D input, default axes = [1,2,3]
    },
  },
  {
    name:
        '[layerNormalization] Throw if the scale rank doesn\'t match axis rank.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      scale: {
        dataType: 'float32',
        dimensions: [1, 2, 3, 4]
      },  // for 4D input, default axes = [1,2,3]
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
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

      if (test.output) {
        const output = builder.layerNormalization(input, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder.layerNormalization(input, test.options));
      }
    }, test.name));
