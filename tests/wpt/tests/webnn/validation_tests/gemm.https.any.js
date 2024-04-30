// META: title=validation tests for WebNN API gemm operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kExampleInputDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2]
};

validateTwoInputsFromMultipleBuilders('gemm');

multi_builder_test(async (t, builder, otherBuilder) => {
  const cFromOtherBuilder = otherBuilder.input('c', kExampleInputDescriptor);
  const options = {c: cFromOtherBuilder};

  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  assert_throws_js(TypeError, () => builder.gemm(a, b, options));
}, '[gemm] throw if c option is from another builder');

const tests = [
  {
    name: '[gemm] Test building gemm with default option.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    output: {dataType: 'float32', dimensions: [2, 4]}
  },
  {
    name:
        '[gemm] Throw if inputShapeA[1] is not equal to inputShapeB[0] default options.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name: '[gemm] Test building gemm with aTranspose=true.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [2, 4]},
    options: {
      aTranspose: true,
    },
    output: {dataType: 'float32', dimensions: [3, 4]}
  },
  {
    name:
        '[gemm] Throw if inputShapeA[0] is not equal to inputShapeB[0] with aTranspose=true.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    options: {
      aTranspose: true,
    },
  },
  {
    name: '[gemm] Test building gemm with bTranspose=true.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [4, 3]},
    options: {
      bTranspose: true,
    },
    output: {dataType: 'float32', dimensions: [2, 4]}
  },
  {
    name:
        '[gemm] Throw if inputShapeA[0] is not equal to inputShapeB[0] with bTranspose=true.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    options: {
      bTranspose: true,
    },
  },
  {
    name: '[gemm] Throw if the rank of inputA is not 2.',
    a: {dataType: 'float32', dimensions: [2, 3, 1]},
    b: {dataType: 'float32', dimensions: [2, 4]},
  },
  {
    name: '[gemm] Throw if the rank of inputB is not 2.',
    a: {dataType: 'float32', dimensions: [2, 4]},
    b: {dataType: 'float32', dimensions: [2, 3, 1]},
  },
  {
    name: '[gemm] Throw if data types of two inputs do not match.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float16', dimensions: [3, 4]},
  },
  {
    name: '[gemm] Test building gemm with inputC.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    options: {
      c: {dataType: 'float32', dimensions: [4]},
    },
    output: {dataType: 'float32', dimensions: [2, 4]}
  },
  {
    name: '[gemm] Test building gemm with scalar inputC.',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    options: {
      c: {dataType: 'float32', dimensions: []},
    },
    output: {dataType: 'float32', dimensions: [2, 4]}
  },
  {
    name:
        '[gemm] Throw if inputShapeC is not unidirectionally broadcastable to the output shape [inputShapeA[0], inputShapeB[1]].',
    a: {dataType: 'float32', dimensions: [2, 3]},
    b: {dataType: 'float32', dimensions: [3, 4]},
    options: {
      c: {dataType: 'float32', dimensions: [2, 3]},
    },
  },
  {
    name: '[gemm] Throw if the input data type is not floating point.',
    a: {dataType: 'int32', dimensions: [2, 3]},
    b: {dataType: 'int32', dimensions: [3, 4]}
  },
  {
    name:
        '[gemm] Throw if data type of inputC does not match ones of inputA and inputB.',
    a: {dataType: 'float32', dimensions: [3, 2]},
    b: {dataType: 'float32', dimensions: [4, 3]},
    options: {
      c: {dataType: 'float16', dimensions: [2, 4]},
      aTranspose: true,
      bTranspose: true,
    },
  },
  {
    name: '[gemm] Throw if the rank of inputC is 3.',
    a: {dataType: 'float32', dimensions: [3, 2]},
    b: {dataType: 'float32', dimensions: [4, 3]},
    options: {
      c: {dataType: 'float32', dimensions: [2, 3, 4]},
      aTranspose: true,
      bTranspose: true,
    },
  },
];

tests.forEach(
    test => promise_test(async t => {
      const a = builder.input(
          'a', {dataType: test.a.dataType, dimensions: test.a.dimensions});
      const b = builder.input(
          'b', {dataType: test.b.dataType, dimensions: test.b.dimensions});
      if (test.options && test.options.c) {
        test.options.c = builder.input('c', {
          dataType: test.options.c.dataType,
          dimensions: test.options.c.dimensions
        });
      }
      if (test.output) {
        const output = builder.gemm(a, b, test.options);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder.gemm(a, b, test.options));
      }
    }, test.name));
