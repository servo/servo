// META: title=validation tests for WebNN API matmul operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

validateTwoInputsFromMultipleBuilders('matmul');

const tests = [
  {
    name: '[matmul] Throw if first input\'s rank is less than 2',
    inputs: {
      a: {dataType: 'float32', dimensions: [2]},
      b: {dataType: 'float32', dimensions: [2, 2]}
    }
  },
  {
    name: '[matmul] Throw if second input\'s rank is less than 2',
    inputs: {
      a: {dataType: 'float32', dimensions: [2, 2]},
      b: {dataType: 'float32', dimensions: [2]}
    }
  },
  {
    name: '[matmul] Test with 2-D input and 4-D input',
    inputs: {
      a: {dataType: 'float32', dimensions: [1, 4]},
      b: {dataType: 'float32', dimensions: [2, 2, 4, 2]}
    },
    output: {dataType: 'float32', dimensions: [2, 2, 1, 2]}
  },
  {
    name: '[matmul] Test with 2-D input and 2-D input',
    inputs: {
      a: {dataType: 'float32', dimensions: [4, 2]},
      b: {dataType: 'float32', dimensions: [2, 3]}
    },
    output: {dataType: 'float32', dimensions: [4, 3]}
  },
  {
    // batchShape is a clone of inputShape with the spatial dimensions
    // (last 2 items) removed.
    name:
        '[matmul] Test with 3-D input and 3-D input of broadcastable batchShape',
    inputs: {
      a: {dataType: 'float32', dimensions: [2, 3, 4]},
      b: {dataType: 'float32', dimensions: [1, 4, 1]}
    },
    output: {dataType: 'float32', dimensions: [2, 3, 1]}
  },
  {
    // batchShape is a clone of inputShape with the spatial dimensions
    // (last 2 items) removed.
    name:
        '[matmul] Test with 4-D input and 3-D input of broadcastable batchShape',
    inputs: {
      a: {dataType: 'float32', dimensions: [2, 2, 3, 4]},
      b: {dataType: 'float32', dimensions: [1, 4, 5]}
    },
    output: {dataType: 'float32', dimensions: [2, 2, 3, 5]}
  },
  {
    name: '[matmul] Test with 3-D input and 3-D input',
    inputs: {
      a: {dataType: 'float32', dimensions: [2, 3, 4]},
      b: {dataType: 'float32', dimensions: [2, 4, 5]}
    },
    output: {dataType: 'float32', dimensions: [2, 3, 5]}
  },
  {
    name: '[matmul] Throw if the input data type is not floating point',
    inputs: {
      a: {dataType: 'uint32', dimensions: [2, 3, 4]},
      b: {dataType: 'uint32', dimensions: [2, 4, 5]}
    }
  },
  {
    name: '[matmul] Throw if data type of two inputs don\'t match',
    inputs: {
      a: {dataType: 'float32', dimensions: [2, 3, 4]},
      b: {dataType: 'float16', dimensions: [2, 4, 5]}
    }
  },
  {
    name:
        '[matmul] Throw if columns of first input\'s shape doesn\'t match the rows of second input\'s shape',
    inputs: {
      a: {dataType: 'float32', dimensions: /* [rows, columns] */[2, 3]},
      b: {dataType: 'float32', dimensions: /* [rows, columns] */[2, 4]}
    },
  },
  {
    // batchShape is a clone of inputShape with the spatial dimensions
    // (last 2 items) removed.
    name: '[matmul] Throw if batchShapes aren\'t bidirectionally broadcastable',
    inputs: {
      a: {dataType: 'float32', dimensions: [3, 3, 4]},
      b: {dataType: 'float32', dimensions: [2, 4, 1]}
    },
  },
];

tests.forEach(test => promise_test(async t => {
                const inputA = builder.input('a', {
                  dataType: test.inputs.a.dataType,
                  dimensions: test.inputs.a.dimensions
                });
                const inputB = builder.input('b', {
                  dataType: test.inputs.b.dataType,
                  dimensions: test.inputs.b.dimensions
                });
                if (test.output) {
                  const output = builder.matmul(inputA, inputB);
                  assert_equals(output.dataType(), test.output.dataType);
                  assert_array_equals(output.shape(), test.output.dimensions);
                } else {
                  assert_throws_js(
                      TypeError, () => builder.matmul(inputA, inputB));
                }
              }, test.name));
