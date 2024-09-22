'use strict';

// https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype
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

const shape0D = [];
const shape1D = [2];
const shape2D = [2, 3];
const shape3D = [2, 3, 4];
const shape4D = [2, 3, 4, 5];
const shape5D = [2, 3, 4, 5, 6];

const adjustOffsetsArray = [
  // Decrease 1
  -1,
  // Increase 1
  1
];

// TODO
// Add more 5+ dimensions
const allWebNNShapesArray =
    [shape0D, shape1D, shape2D, shape3D, shape4D, shape5D];

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

function getRank(inputShape) {
  return inputShape.length;
}

function getAxisArray(inputShape) {
  return Array.from({length: inputShape.length}, (_, i) => i);
}

function getAxesArrayContainSameValues(inputShape) {
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
  const length = inputShape.length;
  if (length >= 2) {
    const validAxesArrayFull = getAxisArray(inputShape);
    for (let index = 0; index < length; index++) {
      axesArrayContainSameValues.push(new Array(2).fill(validAxesArrayFull[index]));
      if (length > 2) {
        axesArrayContainSameValues.push(new Array(3).fill(validAxesArrayFull[index]));
      }
    }
  }
  return axesArrayContainSameValues;
}

function generateUnbroadcastableShapes(shape) {
  // Currently this function returns an array of some unbroadcastable shapes.
  // for example given the input shape [2, 3, 4]
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
  if (shape.every(dimension => dimension === 1)) {
    throw new Error(`[${shape}] always can be broadcasted`);
  }
  const resultShapes = [];
  const length = shape.length;
  if (!shape.slice(0, length - 1).every(dimension => dimension === 1)) {
    for (let i = 0; i < length; i++) {
      if (shape[i] !== 1) {
        for (let offset of [-1, 1]) {
          const shapeB = shape.slice();
          shapeB[i] += offset;
          if (shapeB[i] !== 1) {
            resultShapes.push(shapeB);
          }
        }
      }
    }
  }
  const lastDimensionSize = shape[length - 1];
  if (lastDimensionSize !== 1) {
    for (let j = 0; j <= length; j++) {
      if (lastDimensionSize > 2) {
        resultShapes.push(Array(j).fill(1).concat([lastDimensionSize - 1]));
      }
      resultShapes.push(Array(j).fill(1).concat([lastDimensionSize + 1]));
    }
  }
  return resultShapes;
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
let context;

test(() => assert_not_equals(navigator.ml, undefined, "ml property is defined on navigator"));

promise_setup(async () => {
  if (navigator.ml === undefined) {
    return;
  }
  const deviceType = location.search.substring(1);
  context = await navigator.ml.createContext({deviceType: deviceType});
}, {explicit_timeout: true});

function assert_throws_with_label(func, regrexp) {
  try {
    func.call(this);
    assert_true(false, 'Graph builder method unexpectedly succeeded');
  } catch (e) {
    assert_equals(e.name, 'TypeError');
    const error_message = e.message;
    assert_not_equals(error_message.match(regrexp), null);
  }
}

function validateTwoInputsBroadcastable(operationName, label) {
  if (navigator.ml === undefined) {
    return;
  }
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    for (let dataType of allWebNNOperandDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
        continue;
      }
      for (let shape of allWebNNShapesArray) {
        if (shape.length > 0) {
          const inputA =
              builder.input(`inputA${++inputAIndex}`, {dataType, shape});
          const unbroadcastableShapes = generateUnbroadcastableShapes(shape);
          for (let shape of unbroadcastableShapes) {
            const inputB =
                builder.input(`inputB${++inputBIndex}`, {dataType, shape});
            assert_equals(typeof builder[operationName], 'function');
            const options = {label};
            const regrexp = new RegExp('\\[' + label + '\\]');
            assert_throws_with_label(
                () => builder[operationName](inputA, inputB, options), regrexp);
            assert_throws_with_label(
                () => builder[operationName](inputB, inputA, options), regrexp);
          }
        }
      }
    }
  }, `[${operationName}] TypeError is expected if two inputs aren't broadcastable`);
}

function validateTwoInputsOfSameDataType(operationName, label) {
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
      const builder = new MLGraphBuilder(context);
      for (let dataType of allWebNNOperandDataTypes) {
        if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
          assert_throws_js(
              TypeError,
              () => builder.input(
                  `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
          continue;
        }
        for (let shape of allWebNNShapesArray) {
          const inputA =
              builder.input(`inputA${++inputAIndex}`, {dataType, shape});
          for (let dataTypeB of allWebNNOperandDataTypes) {
            if (!context.opSupportLimits().input.dataTypes.includes(
                    dataTypeB)) {
              assert_throws_js(
                  TypeError,
                  () => builder.input(
                      `inputB${++inputBIndex}`, {dataTypeB, shape1D}));
              continue;
            }
            if (dataType !== dataTypeB) {
              const inputB = builder.input(
                  `inputB${++inputBIndex}`, {dataType: dataTypeB, shape});
              const options = {label};
              const regrexp = new RegExp('\\[' + label + '\\]');
              assert_equals(typeof builder[subOperationName], 'function');
              assert_throws_with_label(
                  () => builder[subOperationName](inputA, inputB, options),
                  regrexp);
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
      const builder = new MLGraphBuilder(context);
      for (let dataType of allWebNNOperandDataTypes) {
        if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
          assert_throws_js(
              TypeError,
              () => builder.input(
                  `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
          continue;
        }
        for (let shape of allWebNNShapesArray) {
          const rank = getRank(shape);
          if (rank >= 1) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, shape});
            for (let invalidAxis of invalidAxisArray) {
              assert_equals(typeof builder[subOperationName], 'function');
              assert_throws_js(
                  TypeError,
                  () => builder[subOperationName](input, {axes: invalidAxis}));
            }
            for (let axis of notUnsignedLongAxisArray) {
              assert_false(
                  typeof axis === 'number' && Number.isInteger(axis),
                  `[${subOperationName}] any of options.axes elements should be of 'unsigned long'`);
              assert_equals(typeof builder[subOperationName], 'function');
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
      const builder = new MLGraphBuilder(context);
      for (let dataType of allWebNNOperandDataTypes) {
        if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
          assert_throws_js(
              TypeError,
              () => builder.input(
                  `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
          continue;
        }
        for (let shape of allWebNNShapesArray) {
          const rank = getRank(shape);
          if (rank >= 1) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, shape});
            assert_equals(typeof builder[subOperationName], 'function');
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
      const builder = new MLGraphBuilder(context);
      for (let dataType of allWebNNOperandDataTypes) {
        if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
          assert_throws_js(
              TypeError,
              () => builder.input(
                  `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
          continue;
        }
        for (let shape of allWebNNShapesArray) {
          const rank = getRank(shape);
          if (rank >= 2) {
            const input =
                builder.input(`input${++inputIndex}`, {dataType, shape});
            const axesArrayContainSameValues =
                getAxesArrayContainSameValues(shape);
            for (let axes of axesArrayContainSameValues) {
              assert_equals(typeof builder[subOperationName], 'function');
              assert_throws_js(
                  TypeError, () => builder[subOperationName](input, {axes}));
            }
          }
        }
      }
    }, `[${subOperationName}] TypeError is expected if two or more values are same in the axes sequence`);
  }
}

// TODO: remove this method once all the data type limits of the unary
// operations are specified in context.OpSupportLimits().
/**
 * Validate a unary operation
 * @param {String} operationName - An operation name
 * @param {Array} supportedDataTypes - Test building with these data types
 *     succeeds and test building with all other data types fails
 */
function validateUnaryOperation(operationName, supportedDataTypes, label) {
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    for (let dataType of supportedDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
        continue;
      }
      for (let shape of allWebNNShapesArray) {
        const input = builder.input(`input`, {dataType, shape});
        assert_equals(typeof builder[operationName], 'function');
        const output = builder[operationName](input);
        assert_equals(output.dataType(), dataType);
        assert_array_equals(output.shape(), shape);
      }
    }
  }, `[${operationName}] Test building an unary operator with supported type.`);

  const unsupportedDataTypes =
      new Set(allWebNNOperandDataTypes).difference(new Set(supportedDataTypes));
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    for (let dataType of unsupportedDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
        continue;
      }
      for (let shape of allWebNNShapesArray) {
        const input = builder.input(`input`, {dataType, shape});
        assert_equals(typeof builder[operationName], 'function');
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder[operationName](input, options), regrexp);
      }
    }
  }, `[${operationName}] Throw if the dataType is not supported for an unary operator.`);
}

/**
 * Validate a single input operation
 * @param {String} operationName - An operation name
 */
function validateSingleInputOperation(operationName, label) {
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    const supportedDataTypes =
        context.opSupportLimits()[operationName].input.dataTypes;
    for (let dataType of supportedDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        continue;
      }
      for (let shape of allWebNNShapesArray) {
        const input = builder.input(`input`, {dataType, shape});
        const output = builder[operationName](input);
        assert_equals(output.dataType(), dataType);
        assert_array_equals(output.shape(), shape);
      }
    }
  }, `[${operationName}] Test building the operator with supported data type.`);

  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    const unsupportedDataTypes =
        new Set(allWebNNOperandDataTypes)
            .difference(new Set(
                context.opSupportLimits()[operationName].input.dataTypes));
    for (let dataType of unsupportedDataTypes) {
      if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
        assert_throws_js(
            TypeError,
            () => builder.input(
                `inputA${++inputAIndex}`, {dataType, shape: shape1D}));
        continue;
      }
      for (let shape of allWebNNShapesArray) {
        const input = builder.input(`input`, {dataType, shape});
        assert_equals(typeof builder[operationName], 'function');
        const options = {label};
        const regrexp = new RegExp('\\[' + label + '\\]');
        assert_throws_with_label(
            () => builder[operationName](input, options), regrexp);
      }
    }
  }, `[${operationName}] Throw if the data type is not supported for the operator.`);
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
  shape: [2, 2]
}) {
  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder =
        otherBuilder.input('input', operatorDescriptor);
    assert_equals(typeof builder[operatorName], 'function');
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
  const opDescriptor = {dataType: 'float32', shape: [2, 2]};

  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder = otherBuilder.input('other', opDescriptor);

    const input = builder.input('input', opDescriptor);
    assert_equals(typeof builder[operatorName], 'function');
    assert_throws_js(
        TypeError, () => builder[operatorName](inputFromOtherBuilder, input));
  }, `[${operatorName}] throw if first input is from another builder`);

  multi_builder_test(async (t, builder, otherBuilder) => {
    const inputFromOtherBuilder = otherBuilder.input('other', opDescriptor);

    const input = builder.input('input', opDescriptor);
    assert_equals(typeof builder[operatorName], 'function');
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
