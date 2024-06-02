// META: title=validation tests for WebNN API argMin/Max operations
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const kArgMinMaxOperators = [
  'argMin',
  'argMax',
];

const tests = [
  {
    name: '[argMin/Max] Test with default options.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    output: {dataType: 'float32', dimensions: []}
  },
  {
    name: '[argMin/Max] Test with axes=[].',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [],
    },
    output: {dataType: 'float32', dimensions: [1, 2, 3, 4]}
  },
  {
    name: '[argMin/Max] Test scalar input with empty axes.',
    input: {dataType: 'float32', dimensions: []},
    options: {
      axes: [],
    },
    output: {dataType: 'float32', dimensions: []}
  },
  {
    name: '[argMin/Max] Test with axes=[1].',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1],
    },
    output: {dataType: 'float32', dimensions: [1, 3, 4]}
  },
  {
    name: '[argMin/Max] Test with axes=[1, 3] and keepDimensions=true.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1, 3],
      keepDimensions: true,
    },
    output: {dataType: 'float32', dimensions: [1, 1, 3, 1]}
  },
  {
    name: '[argMin/Max] Test with axes=[1, 3] and keepDimensions=false.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1, 3],
      keepDimensions: false,
    },
    output: {dataType: 'float32', dimensions: [1, 3]}
  },
  {
    name: '[argMin/Max] Test with axes=[1] and selectLastIndex=true.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1],
      selectLastIndex: true,
    },
    output: {dataType: 'float32', dimensions: [1, 3, 4]}
  },
  {
    name: '[argMin/Max] Test with axes=[1] and selectLastIndex=false.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1],
      selectLastIndex: false,
    },
    output: {dataType: 'float32', dimensions: [1, 3, 4]}
  },
  {
    name:
        '[argMin/Max] Throw if the value in axes is greater than or equal to input rank.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [4],
    },
  },
  {
    name:
        '[argMin/Max] Throw if two or more values are same in the axes sequence.',
    input: {dataType: 'float32', dimensions: [1, 2, 3, 4]},
    options: {
      axes: [1, 1],
    },
  },
  {
    name: '[argMin/Max] Throw if input is a scalar and axes is non-empty.',
    input: {dataType: 'float32', dimensions: []},
    options: {
      axes: [1],
    },
  },
];

function runTests(operatorName, tests) {
  tests.forEach(test => {
    promise_test(async t => {
      const input = builder.input(
          'input',
          {dataType: test.input.dataType, dimensions: test.input.dimensions});

      if (test.output) {
        const output = builder[operatorName](input, test.options);
        assert_equals(output.dataType(), 'int64');
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(
            TypeError, () => builder[operatorName](input, test.options));
      }
    }, test.name.replace('[argMin/Max]', `[${operatorName}]`));
  });
}

kArgMinMaxOperators.forEach((operatorName) => {
  validateInputFromAnotherBuilder(operatorName);
  runTests(operatorName, tests);
});
