'use strict';

// https://webmachinelearning.github.io/webnn/#enumdef-mloperanddatatype
const allWebNNOperandDataTypes = [
  'float32',
  'float16',
  'int32',
  'uint32',
  'int64',
  'uint64',
  'int8',
  'uint8'
];

// https://webidl.spec.whatwg.org/#idl-unsigned-long
// The unsigned long type is an unsigned integer type that has values in the
// range [0, 4294967295].
// 4294967295 = 2 ** 32 - 1
const kMaxUnsignedLong = 2 ** 32 - 1;

const floatingPointTypes = ['float32', 'float16'];

const signedIntegerTypes = ['int32', 'int64', 'int8'];

const unsignedLongType = 'unsigned long';

const dimensions0D = [];
const dimensions1D = [2];
const dimensions2D = [2, 3];
const dimensions3D = [2, 3, 4];
const dimensions4D = [2, 3, 4, 5];
const dimensions5D = [2, 3, 4, 5, 6];

const adjustOffsetsArray = [
  // Decrease 1
  -1,
  // Increase 1
  1
];

// TODO
// Add more 5+ dimensions
const allWebNNDimensionsArray = [
  dimensions0D,
  dimensions1D,
  dimensions2D,
  dimensions3D,
  dimensions4D,
  dimensions5D
];

const notUnsignedLongAxisArray = [
  // String
  'abc',
  // BigInt
  BigInt(100),
  // Object
  {
    value: 1
  },
  // Array Object
  [0, 1],
  // Date Object
  new Date("2024-01-01"),
];

function getRank(inputDimensions) {
  return inputDimensions.length;
}

function getAxisArray(inputDimensions) {
  return Array.from({length: inputDimensions.length}, (_, i) => i);
}

function getAxesArrayContainSameValues(inputDimensions) {
  // TODO
  // Currently this function returns an array containing each element which all have the same value.
  // For example axes: [0, 1, 2] for 3D input tensor
  // this function returns
  // [
  //   // two values are same
  //   [0, 0],
  //   [1, 1],
  //   [2, 2],
  //   // three values are same
  //   [0, 0, 0],
  //   [1, 1, 1]
  //   [2, 2, 2]
  // ]
  // while it should return
  // [
  //   // two values are same
  //   [0, 0],
  //   [1, 1],
  //   [2, 2],
  //   [0, 0, 1],
  //   [0, 0, 2],
  //   [0, 1, 0],
  //   [0, 2, 0],
  //   [1, 0, 0],
  //   [2, 0, 0],
  //   [1, 1, 0],
  //   [1, 1, 2],
  //   [1, 0, 1],
  //   [1, 2, 1],
  //   [0, 1, 1],
  //   [2, 1, 1],
  //   [2, 2, 0],
  //   [2, 2, 1],
  //   [2, 0, 2],
  //   [2, 1, 2],
  //   [0, 2, 2],
  //   [1, 2, 2],
  //   // three (all) values are same
  //   [0, 0, 0],
  //   [1, 1, 1]
  //   [2, 2, 2]
  // ]
  const axesArrayContainSameValues = [];
  const length = inputDimensions.length;
  if (length >= 2) {
    const validAxesArrayFull = getAxisArray(inputDimensions);
    for (let index = 0; index < length; index++) {
      axesArrayContainSameValues.push(new Array(2).fill(validAxesArrayFull[index]));
      if (length > 2) {
        axesArrayContainSameValues.push(new Array(3).fill(validAxesArrayFull[index]));
      }
    }
  }
  return axesArrayContainSameValues;
}

function generateUnbroadcastableDimensionsArray(dimensions) {
  // Currently this function returns an array of some unbroadcastable dimensions.
  // for example given dimensions [2, 3, 4]
  // this function returns
  // [
  //   [3, 3, 4],
  //   [2, 2, 4],
  //   [2, 4, 4],
  //   [2, 3, 3],
  //   [2, 3, 5],
  //   [3],
  //   [5],
  //   [1, 3],
  //   [1, 5],
  //   [1, 1, 3],
  //   [1, 1, 5],
  //   [1, 1, 1, 3],
  //   [1, 1, 1, 5],
  // ]
  if (dimensions.every(v => v === 1)) {
    throw new Error(`[${dimensions}] always can be broadcasted`);
  }
  const resultDimensions = [];
  const length = dimensions.length;
  if (!dimensions.slice(0, length - 1).every(v => v === 1)) {
    for (let i = 0; i < length; i++) {
      if (dimensions[i] !== 1) {
        for (let offset of [-1, 1]) {
          const dimensionsB = dimensions.slice();
          dimensionsB[i] += offset;
          if (dimensionsB[i] !== 1) {
            resultDimensions.push(dimensionsB);
          }
        }
      }
    }
  }
  const lastDimensionSize = dimensions[length - 1];
  if (lastDimensionSize !== 1) {
    for (let j = 0; j <= length; j++) {
      if (lastDimensionSize > 2) {
        resultDimensions.push(Array(j).fill(1).concat([lastDimensionSize - 1]));
      }
      resultDimensions.push(Array(j).fill(1).concat([lastDimensionSize + 1]));
    }
  }
  return resultDimensions;
}

function generateOutOfRangeValuesArray(type) {
  let range, outsideValueArray;
  switch (type) {
    case 'unsigned long':
      range = [0, kMaxUnsignedLong];
      break;
    default:
      throw new Error(`Unsupport ${type}`);
  }
  outsideValueArray = [range[0] - 1, range[1] + 1];
  return outsideValueArray;
}

let inputIndex = 0;
let inputAIndex = 0;
let inputBIndex = 0;
let context, builder;

test(() => assert_not_equals(navigator.ml, undefined, "ml property is defined on navigator"));

promise_setup(async () => {
  if (navigator.ml === undefined) {
    return;
  }
  context = await navigator.ml.createContext();
  builder = new MLGraphBuilder(context);
}, {explicit_timeout: true});

function validateTwoInputsBroadcastable(operationName) {
  if (navigator.ml === undefined) {
    return;
  }
  promise_test(async t => {
    for (let dataType of allWebNNOperandDataTypes) {
      for (let dimensions of allWebNNDimensionsArray) {
        if (dimensions.length > 0) {
          const inputA = builder.input(`inputA${++inputAIndex}`, {dataType, dimensions});
          const unbroadcastableDimensionsArray = generateUnbroadcastableDimensionsArray(dimensions);
          for (let unbroadcastableDimensions of unbroadcastableDimensionsArray) {
            const inputB = builder.input(`inputB${++inputBIndex}`, {dataType, dimensions: unbroadcastableDimensions});
            assert_throws_js(
                TypeError, () => builder[operationName](inputA, inputB));
            assert_throws_js(
                TypeError, () => builder[operationName](inputB, inputA));
          }
        }
      }
    }
  }, `[${operationName}] TypeError is expected if two inputs aren't broadcastable`);
}

function validateTwoInputsOfSameDataType(operationName) {
  if (navigator.ml === undefined) {
    return;
  }
  let operationNameArray;
  if (typeof operationName === 'string') {
    operationNameArray = [operationName];
  } else if (Array.isArray(operationName)) {
    operationNameArray = operationName;
  } else {
    throw new Error(`${operationName} should be an operation name string or an operation name string array`);
  }
  for (let subOperationName of operationNameArray) {
    promise_test(async t => {
      for (let dataType of allWebNNOperandDataTypes) {
        for (let dimensions of allWebNNDimensionsArray) {
          const inputA = builder.input(`inputA${++inputAIndex}`, {dataType, dimensions});
          for (let dataTypeB of allWebNNOperandDataTypes) {
            if (dataType !== dataTypeB) {
              const inputB = builder.input(`inputB${++inputBIndex}`, {dataType: dataTypeB, dimensions});
              assert_throws_js(
                  TypeError, () => builder[subOperationName](inputA, inputB));
            }
          }
        }
      }
    }, `[${subOperationName}] TypeError is expected if two inputs aren't of same data type`);
  }
}

/**
 * Validate options.axes by given operation and input rank for
 * argMin/Max / layerNormalization / Reduction operations operations
 * @param {(String[]|String)} operationName - An operation name array or an
 *     operation name
 */
function validateOptionsAxes(operationName) {
  if (navigator.ml === undefined) {
    return;
  }
  let operationNameArray;
  if (typeof operationName === 'string') {
    operationNameArray = [operationName];
  } else if (Array.isArray(operationName)) {
    operationNameArray = operationName;
  } else {
    throw new Error(`${operationName} should be an operation name string or an operation name string array`);
  }
  const invalidAxisArray = generateOutOfRangeValuesArray(unsignedLongType);
  for (let subOperationName of operationNameArray) {
    // TypeError is expected if any of options.axes elements is not an unsigned long interger
    promise_test(async t => {
      for (let dataType of allWebNNOperandDataTypes) {
        for (let dimensions of allWebNNDimensionsArray) {
          const rank = getRank(dimensions);
          if (rank >= 1) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, dimensions});
            for (let invalidAxis of invalidAxisArray) {
              assert_throws_js(
                  TypeError,
                  () => builder[subOperationName](input, {axes: invalidAxis}));
            }
            for (let axis of notUnsignedLongAxisArray) {
              assert_false(
                  typeof axis === 'number' && Number.isInteger(axis),
                  `[${subOperationName}] any of options.axes elements should be of 'unsigned long'`);
              assert_throws_js(
                  TypeError,
                  () => builder[subOperationName](input, {axes: [axis]}));
            }
          }
        }
      }
    }, `[${subOperationName}] TypeError is expected if any of options.axes elements is not an unsigned long interger`);

    // TypeError is expected if any of options.axes elements is greater or equal
    // to the size of input
    promise_test(async t => {
      for (let dataType of allWebNNOperandDataTypes) {
        for (let dimensions of allWebNNDimensionsArray) {
          const rank = getRank(dimensions);
          if (rank >= 1) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, dimensions});
            assert_throws_js(
                TypeError,
                () => builder[subOperationName](input, {axes: [rank]}));
            assert_throws_js(
                TypeError,
                () => builder[subOperationName](input, {axes: [rank + 1]}));
          }
        }
      }
    }, `[${subOperationName}] TypeError is expected if any of options.axes elements is greater or equal to the size of input`);

    // TypeError is expected if two or more values are same in the axes sequence
    promise_test(async t => {
      for (let dataType of allWebNNOperandDataTypes) {
        for (let dimensions of allWebNNDimensionsArray) {
          const rank = getRank(dimensions);
          if (rank >= 2) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, dimensions});
            const axesArrayContainSameValues =
                getAxesArrayContainSameValues(dimensions);
            for (let axes of axesArrayContainSameValues) {
              assert_throws_js(
                  TypeError, () => builder[subOperationName](input, {axes}));
            }
          }
        }
      }
    }, `[${subOperationName}] TypeError is expected if two or more values are same in the axes sequence`);
  }
}

/**
 * Validate a unary operation
 * @param {String} operationName - An operation name
 * @param {Array} supportedDataTypes - Test building with these data types
 *     succeeds and test building with all other data types fails
 * @param {Boolean} alsoBuildActivation - If test building this operation as an
 *     activation
 */
function validateUnaryOperation(
    operationName, supportedDataTypes, alsoBuildActivation = false) {
  // TODO: crbug.com/345271830 - use context.opSupportLimits to get supported
  // data types for current context.
  for (let dataType of supportedDataTypes) {
    for (let dimensions of allWebNNDimensionsArray) {
      promise_test(
          async t => {
            const input = builder.input(`input`, {dataType, dimensions});
            const output = builder[operationName](input);
            assert_equals(output.dataType(), dataType);
            assert_array_equals(output.shape(), dimensions);
          },
          `[${operationName}] Test building an operator, dataType = ${
              dataType}, dimensions = [${dimensions}]`);
    }
  }

  const unsupportedDataTypes =
      new Set(allWebNNOperandDataTypes).difference(new Set(supportedDataTypes));
  for (let dataType of unsupportedDataTypes) {
    for (let dimensions of allWebNNDimensionsArray) {
      promise_test(
          async t => {
            const input = builder.input(`input`, {dataType, dimensions});
            assert_throws_js(TypeError, () => builder[operationName](input));
          },
          `[${operationName}] Throw if the dataType is not supported, dataType = ${
              dataType}, dimensions = [${dimensions}]`);
    }
  }

  if (alsoBuildActivation) {
    promise_test(async t => {
      builder[operationName]();
    }, `[${operationName}] Test building an activation`);
  }
}

/**
 * Basic test that the builder method specified by `operationName` throws if
 * given an input from another builder. Operands which do not accept a float32
 * square 2D input should pass their own `operatorDescriptor`.
 * @param {String} operationName
 * @param {String} operatorDescriptor
 */
function validateInputFromAnotherBuilder(operatorName, operatorDescriptor = {
  dataType: 'float32',
  dimensions: [2, 2]
}) {
  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder =
        otherBuilder.input('input', operatorDescriptor);
    assert_throws_js(
        TypeError, () => builder[operatorName](inputFromOtherBuilder));
  }, `[${operatorName}] throw if input is from another builder`);
};

/**
 * Basic test that the builder method specified by `operationName` throws if one
 * of its inputs is from another builder. This helper may only be used by
 * operands which accept float32 square 2D inputs.
 * @param {String} operationName
 */
function validateTwoInputsFromMultipleBuilders(operatorName) {
  const opDescriptor = {dataType: 'float32', dimensions: [2, 2]};

  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder = otherBuilder.input('other', opDescriptor);

    const input = builder.input('input', opDescriptor);
    assert_throws_js(
        TypeError, () => builder[operatorName](inputFromOtherBuilder, input));
  }, `[${operatorName}] throw if first input is from another builder`);

  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder = otherBuilder.input('other', opDescriptor);

    const input = builder.input('input', opDescriptor);
    assert_throws_js(
        TypeError, () => builder[operatorName](input, inputFromOtherBuilder));
  }, `[${operatorName}] throw if second input is from another builder`);
};

function multi_builder_test(func, description) {
  promise_test(async t => {
    const context = await navigator.ml.createContext();

    const builder = new MLGraphBuilder(context);
    const otherBuilder = new MLGraphBuilder(context);

    await func(t, builder, otherBuilder);
  }, description);
}
