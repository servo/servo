// META: title=validation tests for WebNN API gru operation
// META: global=window,dedicatedworker
// META: script=../resources/utils_validation.js
// META: timeout=long

'use strict';

const steps = 2, batchSize = 3, inputSize = 4, hiddenSize = 5,
    numDirections = 1;

const tests = [
    {
        name: '[gru] Test with default options',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        outputs: [
            { dataType: 'float32', dimensions: [numDirections, batchSize, hiddenSize] }
        ]
    },
    {
        name: '[gru] Test with given options',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [/*numDirections=*/ 2, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [/*numDirections=*/ 2, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: {
            bias: {
                dataType: 'float32',
                dimensions: [/*numDirections=*/ 2, 3 * hiddenSize]
            },
            recurrentBias: {
                dataType: 'float32',
                dimensions: [/*numDirections=*/ 2, 3 * hiddenSize]
            },
            initialHiddenState: {
                dataType: 'float32',
                dimensions: [/*numDirections=*/ 2, batchSize, hiddenSize]
            },
            restAfter: true,
            returnSequence: true,
            direction: 'both',
            layout: 'rzn',
            activations: ['sigmoid', 'relu']
        },
        outputs: [
            {
                dataType: 'float32',
                dimensions: [/*numDirections=*/ 2, batchSize, hiddenSize]
            },
            {
                dataType: 'float32',
                dimensions: [steps, /*numDirections=*/ 2, batchSize, hiddenSize]
            }
        ]
    },
    {
        name: '[gru] TypeError is expected if steps equals to zero',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 4 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 4 * hiddenSize, hiddenSize]
        },
        steps: 0,
        hiddenSize: hiddenSize,
    },
    {
        name: '[gru] TypeError is expected if hiddenSize equals to zero',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: 0
    },
    {
        name: '[gru] TypeError is expected if hiddenSize is too large',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: 4294967295,
    },
    {
        name:
            '[gru] TypeError is expected if the data type of the inputs is not one of the floating point types',
        input: { dataType: 'uint32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'uint32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'uint32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name:
            '[gru] TypeError is expected if the rank of input is not 3',
        input: { dataType: 'float32', dimensions: [steps, batchSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name:
            '[gru] TypeError is expected if input.dimensions[0] is not equal to steps',
        input: { dataType: 'float32', dimensions: [1000, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name: '[gru] TypeError is expected if weight.dimensions[1] is not 3 * hiddenSize',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 4 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name:
            '[gru] TypeError is expected if the rank of recurrentWeight is not 3',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight:
            { dataType: 'float32', dimensions: [numDirections, 3 * hiddenSize] },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name:
            '[gru] TypeError is expected if the recurrentWeight.dimensions is invalid',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight:
            { dataType: 'float32', dimensions: [numDirections, 4 * hiddenSize, inputSize] },
        steps: steps,
        hiddenSize: hiddenSize
    },
    {
        name:
            '[gru] TypeError is expected if the size of options.activations is not 2',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: { activations: ['sigmoid', 'tanh', 'relu'] }
    },
    {
        name:
            '[gru] TypeError is expected if the rank of options.bias is not 2',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: { bias: { dataType: 'float32', dimensions: [numDirections] } }
    },
    {
        name:
            '[gru] TypeError is expected if options.bias.dimensions[1] is not 3 * hiddenSize',
        input: { dataType: 'float32', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float32',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: { bias: { dataType: 'float32', dimensions: [numDirections, hiddenSize] } }
    },
    {
        name:
            '[gru] TypeError is expected if options.recurrentBias.dimensions[1] is not 3 * hiddenSize',
        input: { dataType: 'float16', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: {
            recurrentBias: { dataType: 'float16', dimensions: [numDirections, 4 * hiddenSize] }
        }
    },
    {
        name:
            '[gru] TypeError is expected if the rank of options.initialHiddenState is not 3',
        input: { dataType: 'float16', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: {
            initialHiddenState: {
                dataType: 'float16',
                dimensions: [numDirections, batchSize]
            }
        }
    },
    {
        name:
            '[gru] TypeError is expected if options.initialHiddenState.dimensions[2] is not inputSize',
        input: { dataType: 'float16', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: {
            initialHiddenState: {
                dataType: 'float16',
                dimensions: [numDirections, batchSize, 3 * hiddenSize]
            }
        }
    },
    {
        name:
            '[gru] TypeError is expected if the dataType of options.initialHiddenState is incorrect',
        input: { dataType: 'float16', dimensions: [steps, batchSize, inputSize] },
        weight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, inputSize]
        },
        recurrentWeight: {
            dataType: 'float16',
            dimensions: [numDirections, 3 * hiddenSize, hiddenSize]
        },
        steps: steps,
        hiddenSize: hiddenSize,
        options: {
            initialHiddenState: {
                dataType: 'uint64',
                dimensions: [numDirections, batchSize, hiddenSize]
            }
        }
    }
];

tests.forEach(
    test => promise_test(async t => {
        const input = builder.input(
            'input',
            { dataType: test.input.dataType, dimensions: test.input.dimensions });
        const weight = builder.input(
            'weight',
            { dataType: test.weight.dataType, dimensions: test.weight.dimensions });
        const recurrentWeight = builder.input('recurrentWeight', {
            dataType: test.recurrentWeight.dataType,
            dimensions: test.recurrentWeight.dimensions
        });

        const options = {};
        if (test.options) {
            if (test.options.bias) {
                options.bias = builder.input('bias', {
                    dataType: test.options.bias.dataType,
                    dimensions: test.options.bias.dimensions
                });
            }
            if (test.options.recurrentBias) {
                options.bias = builder.input('recurrentBias', {
                    dataType: test.options.recurrentBias.dataType,
                    dimensions: test.options.recurrentBias.dimensions
                });
            }
            if (test.options.initialHiddenState) {
                options.initialHiddenState = builder.input('initialHiddenState', {
                    dataType: test.options.initialHiddenState.dataType,
                    dimensions: test.options.initialHiddenState.dimensions
                });
            }
            if (test.options.resetAfter) {
                options.resetAfter = test.options.resetAfter;
            }
            if (test.options.returnSequence) {
                options.returnSequence = test.options.returnSequence;
            }
            if (test.options.direction) {
                options.direction = test.options.direction;
            }
            if (test.options.layout) {
                options.layout = test.options.layout;
            }
            if (test.options.activations) {
                options.activations = [];
                test.options.activations.forEach(
                    activation => options.activations.push(builder[activation]()));
            }
        }

        if (test.outputs) {
            const outputs = builder.gru(
                input, weight, recurrentWeight, test.steps, test.hiddenSize,
                options);
            assert_equals(outputs.length, test.outputs.length);
            for (let i = 0; i < outputs.length; ++i) {
                assert_equals(outputs[i].dataType(), test.outputs[i].dataType);
                assert_array_equals(outputs[i].shape(), test.outputs[i].dimensions);
            }
        } else {
            assert_throws_js(
                TypeError,
                () => builder.gru(
                    input, weight, recurrentWeight, test.steps, test.hiddenSize,
                    options));
        }
    }, test.name));