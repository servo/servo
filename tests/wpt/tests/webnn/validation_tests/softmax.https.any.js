// META: title=validation tests for WebNN API softmax operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js

'use strict';

const tests_without_axis = [
    {
        name: '[softmax] Test building Softmax with float32 input without axis.',
        input: { dataType: 'float32', dimensions: [4, 3] },
        output: { dataType: 'float32', dimensions: [4, 3] }
    },
    {
        name: '[softmax] Test building Softmax with float16 input without axis.',
        input: { dataType: 'float16', dimensions: [3, 5] },
        output: { dataType: 'float16', dimensions: [3, 5] }
    },
    {
        name: '[softmax] Throw if the input is not a non-floating point data.',
        input: { dataType: 'int32', dimensions: [3, 2] }
    },
    {
        name: '[softmax] Throw if the input dimensions is not 2.',
        input: { dataType: 'float32', dimensions: [1, 4, 3] }
    }
];

tests_without_axis.forEach(test =>
    promise_test(async t => {
        let input = builder.input(
            `input`, { dataType: test.input.dataType, dimensions: test.input.dimensions }
        );
        if (test.output) {
            const output = builder.softmax(input);
            assert_equals(output.dataType(), test.output.dataType);
            assert_array_equals(output.shape(), test.output.dimensions);
        } else {
            assert_throws_js(TypeError, () => builder.softmax(input));
        }
    }, test.name)
);

multi_builder_test(async (t, builder, otherBuilder) => {
    const operandDescriptor = { dataType: 'float32', dimensions: [2, 3] };
    const inputFromOtherBuilder = otherBuilder.input('input', operandDescriptor);

    assert_throws_js(
        TypeError,
        () => builder.softmax(inputFromOtherBuilder));
}, '[softmax without axis] throw if any input is from another builder');

const tests = [
    {
        name: '[softmax] Test building Softmax with float32 input.',
        input: { dataType: 'float32', dimensions: [4, 4, 3] },
        axis: 1,
        output: { dataType: 'float32', dimensions: [4, 4, 3] }
    },
    {
        name: '[softmax] Test building Softmax with float16 input.',
        input: { dataType: 'float16', dimensions: [3, 1, 5, 2] },
        axis: 2,
        output: { dataType: 'float16', dimensions: [3, 1, 5, 2] }
    },
    {
        name: '[softmax] Throw if the input is not a non-floating-point data.',
        input: { dataType: 'int32', dimensions: [3, 1, 5, 2] },
        axis: 3
    },
    {
        name: '[softmax] Throw if the axis is greater than input rank - 1.',
        input: { dataType: 'float16', dimensions: [3, 1, 5, 2] },
        axis: 4
    }
];

tests.forEach(test =>
    promise_test(async t => {
        let input = builder.input(
            `input`, { dataType: test.input.dataType, dimensions: test.input.dimensions }
        );
        if (test.output) {
            const output = builder.softmax(input, test.axis);
            assert_equals(output.dataType(), test.output.dataType);
            assert_array_equals(output.shape(), test.output.dimensions);
        } else {
            assert_throws_js(TypeError, () => builder.softmax(input, test.axis));
        }
    }, test.name)
);

multi_builder_test(async (t, builder, otherBuilder) => {
    const operandDescriptor = { dataType: 'float32', dimensions: [1, 2, 3] };
    const inputFromOtherBuilder = otherBuilder.input('input', operandDescriptor);
    const axis = 1;

    assert_throws_js(
        TypeError,
        () => builder.softmax(inputFromOtherBuilder, axis));
}, '[softmax] throw if any input is from another builder');
