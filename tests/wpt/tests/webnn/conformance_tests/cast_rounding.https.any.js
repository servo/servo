// META: title=WebNN float16 cast rounding in compositions
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-cast
// https://www.w3.org/TR/webnn/#casting
// A float32 -> float16 -> float32 composition must retain the narrowing cast.
// Narrowed expected values are binary16 values exactly representable in
// binary32, so widening adds no rounding error.
const castRoundingVectors = [
  {
    name: 'normal values and ties to even',
    input: [
      1.0003, -1.0003, 1.0007, -1.0007, 1.00048828125, -1.00048828125,
      1.00146484375, -1.00146484375, 2049, -2049, 2051, -2051
    ],
    expected: [
      1, -1, 1.0009765625, -1.0009765625, 1, -1, 1.001953125,
      -1.001953125, 2048, -2048, 2052, -2052
    ]
  },
  {
    name: 'subnormal values and signed underflow',
    input: [
      2 ** -24, -(2 ** -24), 2 ** -25, -(2 ** -25),
      3 * 2 ** -25, -3 * 2 ** -25, 5 * 2 ** -25, -5 * 2 ** -25,
      1023 * 2 ** -24, -1023 * 2 ** -24,
      2047 * 2 ** -25, -2047 * 2 ** -25, 0, -0
    ],
    expected: [
      2 ** -24, -(2 ** -24), 0, -0,
      2 ** -23, -(2 ** -23), 2 ** -23, -(2 ** -23),
      1023 * 2 ** -24, -1023 * 2 ** -24,
      2 ** -14, -(2 ** -14), 0, -0
    ]
  },
  {
    name: 'overflow and exceptional values',
    input: [
      65504, -65504, 65519, -65519, 65520, -65520, 65536, -65536,
      Infinity, -Infinity, NaN
    ],
    expected: [
      65504, -65504, 65504, -65504, Infinity, -Infinity, Infinity,
      -Infinity, Infinity, -Infinity, NaN
    ]
  }
];

const castRoundingTests = [];
for (const vector of castRoundingVectors) {
  for (const constant of [false, true]) {
    for (const exposeIntermediate of [false, true]) {
      const descriptor = {shape: [vector.input.length], dataType: 'float32'};
      const expectedOutputs = {
        widened: {data: vector.expected, descriptor}
      };
      if (exposeIntermediate) {
        expectedOutputs.narrowed = {
          data: vector.expected,
          descriptor: {shape: [vector.input.length], dataType: 'float16'}
        };
      }
      castRoundingTests.push({
        name: 'cast float32 -> float16 -> float32 ' + vector.name +
            (constant ? ' constant' : ' input') +
            (exposeIntermediate ? ' with both cast outputs' :
                                  ' with only the widened output'),
        graph: {
          inputs: {
            input: {data: vector.input, descriptor, constant}
          },
          operators: [
            {
              name: 'cast',
              arguments: [{input: 'input'}, {type: 'float16'}],
              outputs: 'narrowed'
            },
            {
              name: 'cast',
              arguments: [{input: 'narrowed'}, {type: 'float32'}],
              outputs: 'widened'
            }
          ],
          expectedOutputs
        }
      });
    }
  }
}

// Keep the scalar and constant-folding paths covered as well.
for (const constant of [false, true]) {
  castRoundingTests.push({
    name: 'cast float32 -> float16 -> float32 scalar tie to even ' +
        (constant ? 'constant' : 'input'),
    graph: {
      inputs: {
        input: {
          data: [1.00146484375],
          descriptor: {shape: [], dataType: 'float32'},
          constant
        }
      },
      operators: [
        {
          name: 'cast',
          arguments: [{input: 'input'}, {type: 'float16'}],
          outputs: 'narrowed'
        },
        {
          name: 'cast',
          arguments: [{input: 'narrowed'}, {type: 'float32'}],
          outputs: 'widened'
        }
      ],
      expectedOutputs: {
        widened: {
          data: [1.001953125],
          descriptor: {shape: [], dataType: 'float32'}
        }
      }
    }
  });
}

// An idempotent second cast pair must also preserve the first pair's result.
// Exposing each logical output distinguishes narrowing from later transport.
const successiveCastInput = castRoundingVectors.flatMap(vector => vector.input);
const successiveCastExpected =
    castRoundingVectors.flatMap(vector => vector.expected);
castRoundingTests.push({
  name: 'successive float16 cast boundaries preserve each logical output',
  graph: {
    inputs: {
      input: {
        data: successiveCastInput,
        descriptor: {shape: [successiveCastInput.length], dataType: 'float32'}
      }
    },
    operators: [
      {
        name: 'cast',
        arguments: [{input: 'input'}, {type: 'float16'}],
        outputs: 'firstNarrowed'
      },
      {
        name: 'cast',
        arguments: [{input: 'firstNarrowed'}, {type: 'float32'}],
        outputs: 'firstWidened'
      },
      {
        name: 'cast',
        arguments: [{input: 'firstWidened'}, {type: 'float16'}],
        outputs: 'secondNarrowed'
      },
      {
        name: 'cast',
        arguments: [{input: 'secondNarrowed'}, {type: 'float32'}],
        outputs: 'secondWidened'
      }
    ],
    expectedOutputs: {
      firstNarrowed: {
        data: successiveCastExpected,
        descriptor: {shape: [successiveCastInput.length], dataType: 'float16'}
      },
      firstWidened: {
        data: successiveCastExpected,
        descriptor: {shape: [successiveCastInput.length], dataType: 'float32'}
      },
      secondNarrowed: {
        data: successiveCastExpected,
        descriptor: {shape: [successiveCastInput.length], dataType: 'float16'}
      },
      secondWidened: {
        data: successiveCastExpected,
        descriptor: {shape: [successiveCastInput.length], dataType: 'float32'}
      }
    }
  }
});

// A same-type cast must not introduce an implicit float16 conversion.
const sameTypeFloat32Input = [
  2, -4, 7, 1.0003, -1.0003, 1.0007, -1.0007,
  2 ** -24, -(2 ** -24), 0, -0, Infinity, -Infinity, NaN
];
castRoundingTests.push({
  name: 'cast float32 -> float32 preserves values without float16 rounding',
  graph: {
    inputs: {
      input: {
        data: sameTypeFloat32Input,
        descriptor: {shape: [sameTypeFloat32Input.length], dataType: 'float32'}
      }
    },
    operators: [{
      name: 'cast',
      arguments: [{input: 'input'}, {type: 'float32'}],
      outputs: 'result'
    }],
    expectedOutputs: {
      result: {
        data: sameTypeFloat32Input.map(Math.fround),
        descriptor: {shape: [sameTypeFloat32Input.length], dataType: 'float32'}
      }
    }
  }
});

// Widening is exact for every binary16 value, independently of the tensor's
// rank or native physical layout. Cover singleton and non-aligned dimensions.
const wideningPattern = [
  0, -0, 2 ** -24, -(2 ** -24), 1023 * 2 ** -24,
  -1023 * 2 ** -24, 2 ** -14, -(2 ** -14), 1, -1,
  1.0009765625, -1.0009765625, 65504, -65504, Infinity, NaN
];
for (const shape of [[16, 1, 1], [1, 16, 1, 1], [1, 16, 4, 1], [1, 4, 17]]) {
  const count = shape.reduce((product, dimension) => product * dimension, 1);
  const data = Array.from(
      {length: count}, (_, index) => wideningPattern[index % wideningPattern.length]);
  for (const constant of [false, true]) {
    castRoundingTests.push({
      name: 'cast float16 -> float32 exact widening shape [' + shape + '] ' +
          (constant ? 'constant' : 'input'),
      graph: {
        inputs: {
          input: {data, descriptor: {shape, dataType: 'float16'}, constant}
        },
        operators: [{
          name: 'cast',
          arguments: [{input: 'input'}, {type: 'float32'}],
          outputs: 'result'
        }],
        expectedOutputs: {
          result: {data, descriptor: {shape, dataType: 'float32'}}
        }
      }
    });
  }
}

// The usual ULP comparison treats +0 and -0 as equal. Check the cast's exact
// semantics first, including zero sign and NaN classification (not its payload).
const buildAndCheckCastRounding = async (context, builder, graphResources) => {
  const execution = await buildAndExecuteGraph(context, builder, graphResources);
  for (const [name, expected] of Object.entries(graphResources.expectedOutputs)) {
    const actual = execution.result[name];
    assert_equals(actual.length, expected.data.length, name + ' length');
    const dataType = expected.descriptor.castedType ||
        expected.descriptor.dataType;
    for (let index = 0; index < actual.length; ++index) {
      const value = dataType === 'float16' ?
          float16AsUint16ToNumber(actual[index]) : actual[index];
      if (Number.isNaN(expected.data[index])) {
        assert_true(Number.isNaN(value), name + '[' + index + '] is NaN');
      } else {
        assert_true(Object.is(value, expected.data[index]),
            name + '[' + index + '] retains cast rounding and zero sign');
      }
    }
  }
  return execution;
};

webnn_conformance_test(
    castRoundingTests, buildAndCheckCastRounding, getZeroULPTolerance);
