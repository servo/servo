// META: title=validation tests for WebNN API concat operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const tests = [
  {
    name: '[concat] Test building Concat with one input.',
    inputs: [{dataType: 'float32', dimensions: [4,4,3]}],
    axis: 2,
    output: {dataType: 'float32', dimensions: [4,4,3]}
  },
  {
    name: '[concat] Test building Concat with two inputs',
    inputs: [{dataType: 'float32', dimensions: [3,1,5]},
             {dataType: 'float32', dimensions: [3,2,5]}],
    axis: 1,
    output: {dataType: 'float32', dimensions: [3,3,5]}
  },
  {
    name: '[concat] Test building Concat with three inputs',
    inputs: [{dataType: 'float32', dimensions: [3,5,1]},
             {dataType: 'float32', dimensions: [3,5,2]},
             {dataType: 'float32', dimensions: [3,5,3]}],
    axis: 2,
    output: {dataType: 'float32', dimensions: [3,5,6]}
  },
  {
    name: '[concat] Test building Concat with two 1D inputs.',
    inputs: [{dataType: 'float32', dimensions: [1]},
             {dataType: 'float32', dimensions: [1]}],
    axis: 0,
    output: {dataType: 'float32', dimensions: [2]}
  },
  {
    name: '[concat] Throw if the inputs are empty.',
    axis: 0,
  },
  {
    name: '[concat] Throw if the argument types are inconsistent.',
    inputs: [{dataType: 'float32', dimensions: [1,1]},
             {dataType: 'int32', dimensions: [1,1]}],
    axis: 0,
  },
  {
    name: '[concat] Throw if the inputs have different ranks.',
    inputs: [{dataType: 'float32', dimensions: [1,1]},
             {dataType: 'float32', dimensions: [1,1,1]}],
    axis: 0,
  },
  {
    name: '[concat] Throw if the axis is equal to or greater than the size of ranks',
    inputs: [{dataType: 'float32', dimensions: [1,1]},
             {dataType: 'float32', dimensions: [1,1]}],
    axis: 2,
  },
  {
    name: '[concat] Throw if concat with two 0-D scalars.',
    inputs: [{dataType: 'float32', dimensions: []},
             {dataType: 'float32', dimensions: []}],
    axis: 0,
  },
  {
    name: '[concat] Throw if the inputs have other axes with different sizes except on the axis.',
    inputs: [{dataType: 'float32', dimensions: [1,1,1]},
             {dataType: 'float32', dimensions: [1,2,3]}],
    axis: 1,
  },

];

tests.forEach(test =>
    promise_test(async t => {
      let inputs = [];
      if (test.inputs) {
        for (let i = 0; i < test.inputs.length; ++i) {
          inputs[i] = builder.input(
            `inputs[${i}]`,
            { dataType: test.inputs[i].dataType, dimensions: test.inputs[i].dimensions }
          );
        }
      }
      if (test.output) {
        const output = builder.concat(inputs, test.axis);
        assert_equals(output.dataType(), test.output.dataType);
        assert_array_equals(output.shape(), test.output.dimensions);
      } else {
        assert_throws_js(TypeError, () => builder.concat(inputs, test.axis));
      }
    }, test.name)
  );

multi_builder_test(async (t, builder, otherBuilder) => {
  const operandDescriptor = {dataType: 'float32', dimensions: [2, 2]};

  const inputFromOtherBuilder = otherBuilder.input('input', operandDescriptor);

  const input1 = builder.input('input', operandDescriptor);
  const input2 = builder.input('input', operandDescriptor);
  const input3 = builder.input('input', operandDescriptor);

  assert_throws_js(
      TypeError,
      () => builder.concat([input1, input2, inputFromOtherBuilder, input3]));
}, '[concat] throw if any input is from another builder');
