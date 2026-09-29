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

// Numeric-category orderings used by `findCompatibleType` to widen a
// declared operand dtype (e.g. int8) to a globally-creatable dtype in the
// same category (e.g. int32). Mirrored — with the same ordering — from
// `resources/utils.js` because most validation tests do not load utils.js.
// A few validation tests (e.g. pooling-and-reduction-keep-dims) do load
// both scripts, so the declarations are guarded to avoid redeclaring the
// bindings that utils.js already installed.
if (typeof kIntTypes === 'undefined') {
  globalThis.kIntTypes =
      ['uint4', 'int4', 'uint8', 'int8', 'uint32', 'int32', 'uint64', 'int64'];
  globalThis.kFloatTypes = ['float16', 'float32'];
}

/**
 * Given a target `dataType` (typically an operand's declared dtype, which may
 * not be creatable as a graph input), find a dtype in `supportedTypes`
 * (typically the globally-creatable input dtypes) that `MLGraphBuilder.cast()`
 * can convert into `dataType`. The caller creates the operand with the
 * returned dtype and casts it to `dataType`, so the returned dtype is the
 * cast's input and `dataType` is the cast's output.
 *
 * Candidates are restricted to the same numeric category as `dataType`
 * (kIntTypes / kFloatTypes) and to those later in that category's ordering,
 * i.e. wide enough to hold the values `dataType` can represent.
 *
 * Note: `findCompatibleType` in `resources/utils.js` has the cast input and
 * output checks the other way round; that copy still needs the same fix.
 *
 * @param {string} dataType - The dtype the operand must end up with, i.e. the
 *   cast's output dtype.
 * @param {string[]} supportedTypes - Candidate dtypes for the cast's input.
 * @param {object} castOpSupportLimits - `context.opSupportLimits().cast`.
 * @return {string|null} A dtype from `supportedTypes` that can be cast to
 *   `dataType`, or `null` if there is none / `dataType` is not supported as a
 *   cast output.
 */
if (typeof findCompatibleType === 'undefined') {
  globalThis.findCompatibleType = function(dataType, supportedTypes,
                                           castOpSupportLimits) {
    if (!castOpSupportLimits.output.dataTypes.includes(dataType)) {
      return null;
    }
    for (let supportedType of supportedTypes) {
      if (kIntTypes.includes(dataType) &&
          castOpSupportLimits.input.dataTypes.includes(supportedType) &&
          kIntTypes.indexOf(supportedType) > kIntTypes.indexOf(dataType)) {
        return supportedType;
      }
      if (kFloatTypes.includes(dataType) &&
          castOpSupportLimits.input.dataTypes.includes(supportedType) &&
          kFloatTypes.indexOf(supportedType) > kFloatTypes.indexOf(dataType)) {
        return supportedType;
      }
    }
    return null;
  };
}

const shape0D = [];
const shape1D = [2];
const shape2D = [2, 3];
const shape3D = [2, 3, 4];
const shape4D = [2, 3, 4, 5];
const shape5D = [2, 3, 4, 5, 6];

// Placeholder dim size used by rank-validation helpers (and their
// per-operator `buildFn` callbacks) when synthesising input / companion
// tensor shapes of arbitrary rank. Chosen as 2: the smallest non-degenerate
// (> 1) value that keeps total elements small (2^rank; 256 at rank 8) and
// lets companion tensors indexed by any axis share a uniform
// [kExampleDimSize] shape.
const kExampleDimSize = 2;

/**
 * Default shape builder used by the rank-validation helpers: a uniform
 * [kExampleDimSize, ...] shape of the requested rank. Operators whose rank
 * test needs a differently shaped input — e.g. `expand`, which can only
 * broadcast dimensions of size 1 — pass their own builder to the helpers.
 *
 * @param {number} rank
 * @return {number[]}
 */
function buildExampleShape(rank) {
  return Array(rank).fill(kExampleDimSize);
}

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
  const deviceType = new URLSearchParams(location.search).get('device') ||
      location.search.substring(1);
  context = await navigator.ml.createContext({deviceType: deviceType});
}, {explicit_timeout: true});

function assert_throws_with_label(func, regrexp) {
  try {
    func.call(this);
    assert_unreached('Graph builder method unexpectedly succeeded');
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

function validateTwoBroadcastableInputsTensorLimit(operationName, label) {
  if (navigator.ml === undefined) {
    return;
  }
  promise_test(async t => {
    const builder = new MLGraphBuilder(context);

    const a = builder.input('a', {dataType: 'float32',
        shape: [context.opSupportLimits().maxTensorByteLength / 4, 1]});
    const b = builder.input('b', {dataType: 'float32', shape: [1, 5] });

    const options = {label};
    const regrexp = new RegExp('\\[' + label + '\\]');
    assert_throws_with_label(
        () => builder[operationName](a, b, options), regrexp);
  }, `[${operationName}] throw if the output tensor byte length exceeds limit`);
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
        assert_equals(output.dataType, dataType);
        assert_array_equals(output.shape, shape);
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
        assert_equals(output.dataType, dataType);
        assert_array_equals(output.shape, shape);
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
    const builder = new MLGraphBuilder(context);
    const otherBuilder = new MLGraphBuilder(context);

    await func(t, builder, otherBuilder);
  }, description);
}

/**
 * Return the global input rank max declared by context.opSupportLimits().
 * Convenience wrapper used by rank validation helpers and bespoke Cartesian
 * rank tests. `rankRange` and its `min`/`max` are mandatory, so no fallback
 * is needed.
 */
function getGlobalRankMax() {
  return context.opSupportLimits().input.rankRange.max;
}

/**
 * Pick a data-type info object suitable for materialising an operand of the
 * given `operandLimits` on the current backend.
 *
 * Returns `{sourceDataType, targetDataType}`:
 *  - `sourceDataType` is the dtype to pass to `MLGraphBuilder.input()` (the
 *    cast source when a cast is needed).
 *  - `targetDataType` is the operand's declared dtype — an entry in
 *    `operandLimits.dataTypes` that the operator will accept (the cast
 *    target when a cast is needed). Equals `sourceDataType` when the
 *    declared dtype is directly input-creatable per
 *    `context.opSupportLimits().input.dataTypes`; otherwise `sourceDataType`
 *    is a widening-compatible substitute (via `findCompatibleType`) that
 *    must be cast to `targetDataType` via `MLGraphBuilder.cast()` before
 *    use.
 *
 * Callers should build the operand via `buildInputOperand(builder, name,
 * dataTypeInfo, shape)`, which handles the optional cast uniformly.
 *
 * Returns `undefined` when neither a directly-creatable nor a
 * widening-compatible dtype is available for any of the declared dtypes.
 *
 * @param {object} operandLimits - The operand's entry in
 *   `context.opSupportLimits()[operatorName][operandKey]`.
 * @return {{sourceDataType: string, targetDataType: string}|undefined}
 */
function pickSupportedDataTypeInfo(operandLimits) {
  const globalInputDataTypes = context.opSupportLimits().input.dataTypes;
  const dataType =
      operandLimits.dataTypes.find(dt => globalInputDataTypes.includes(dt));
  if (dataType !== undefined) {
    return {sourceDataType: dataType, targetDataType: dataType};
  }
  // Fallback: iterate declared dtypes; return the first that has a
  // widening-compatible globally-creatable substitute (per
  // `findCompatibleType`). The substitute becomes `sourceDataType` (for
  // `builder.input()`) and is cast to the operand's declared
  // `targetDataType` at use.
  const castOpSupportLimits = context.opSupportLimits().cast;
  for (let dt of operandLimits.dataTypes) {
    const compatibleDataType =
        findCompatibleType(dt, globalInputDataTypes, castOpSupportLimits);
    if (compatibleDataType) {
      return {sourceDataType: compatibleDataType, targetDataType: dt};
    }
  }
  return undefined;
}

/**
 * Materialise an operand of `dataTypeInfo.targetDataType` on the given
 * `builder`. Calls `MLGraphBuilder.input()` with
 * `dataTypeInfo.sourceDataType`, then casts to `dataTypeInfo.targetDataType`
 * if the two differ. `dataTypeInfo` is a value returned from
 * `pickSupportedDataTypeInfo`.
 *
 * @param {MLGraphBuilder} builder
 * @param {string} operandName
 * @param {{sourceDataType: string, targetDataType: string}} dataTypeInfo
 * @param {number[]} shape
 * @return {MLOperand}
 */
function buildInputOperand(builder, operandName, dataTypeInfo, shape) {
  const rawOperand = builder.input(
      operandName, {dataType: dataTypeInfo.sourceDataType, shape});
  return dataTypeInfo.targetDataType === dataTypeInfo.sourceDataType ?
      rawOperand :
      builder.cast(rawOperand, dataTypeInfo.targetDataType);
}

/**
 * Validate that an operator throws TypeError for EVERY rank strictly greater
 * than the maximum allowed by context.opSupportLimits() for the specified
 * operand, up to and including the global input rank max.
 *
 * Iterating the full over-max range (rankMax+1 .. globalRankMax) — rather
 * than testing only rankMax+1 — catches operators that partially reject
 * out-of-range ranks (e.g. reject rankMax+1 but silently accept rankMax+2),
 * which matters because backends (e.g. XNNPACK with XNN_MAX_TENSOR_DIMS=6)
 * may have smaller internal rank limits than the WebNN global max of 8, and
 * every accepted-but-unsupported rank is a potential native crash surface.
 *
 * The test is skipped (returns early) when:
 *  - the operator is absent from opSupportLimits, or
 *  - the operator's max rank already equals (or exceeds) the global input
 *    max rank (no rank above max can be created), or
 *  - none of the operand's supported data types are globally creatable.
 *
 * @param {string} operatorName - The operator name (key in opSupportLimits).
 * @param {string} operandKey - The operand whose rank is under test
 *   (e.g. 'input', 'a').
 * @param {function(MLGraphBuilder, MLOperand): void} buildFn - Receives a
 *   fresh builder and the already-created out-of-range input. Should call the
 *   operator and return its output (or throw TypeError).
 * @param {function(number): number[]} buildShape - Returns the shape to
 *   create the operand with for the rank under test. Defaults to
 *   `buildExampleShape`.
 */
function validateOperandRankTooLarge(operatorName, operandKey, buildFn,
                                     buildShape = buildExampleShape) {
  promise_test(
      async t => {
        const opLimits = context.opSupportLimits()[operatorName];
        if (!opLimits)
          return;
        const {max: rankMax} = opLimits[operandKey].rankRange;
        const globalRankMax = getGlobalRankMax();
        const dataTypeInfo = pickSupportedDataTypeInfo(opLimits[operandKey]);
        if (!dataTypeInfo)
          return;
        // Test every rank in (rankMax, globalRankMax]. Iterating the full range
        // (rather than only rankMax+1) surfaces any rank the operator wrongly
        // accepts between its declared max and the global max — such ranks
        // would otherwise reach the backend and can trigger e.g. XNNPACK
        // stack-buffer overflows (XNN_MAX_TENSOR_DIMS=6 vs. WebNN global max
        // 8).
        for (let rank = rankMax + 1; rank <= globalRankMax; rank++) {
          const shape = buildShape(rank);
          const builder = new MLGraphBuilder(context);
          const input =
              buildInputOperand(builder, operandKey, dataTypeInfo, shape);
          // Allow buildFn to signal "cannot construct on this backend" by
          // returning null (e.g. a companion operand's data type is not
          // globally creatable). In that case, skip this rank.
          let result;
          let thrown;
          try {
            result = buildFn(builder, input);
          } catch (e) {
            thrown = e;
          }
          assert_true(thrown instanceof TypeError,
                      `${operandKey} rank ${rank} (> max ${
                          rankMax}) should be rejected`);
        }
      },
      `[${operatorName}] throw if ${
          operandKey} rank exceeds max rank allowed by opSupportLimits`);
}

/**
 * Validate that an operator throws TypeError for EVERY rank strictly less
 * than the minimum allowed by context.opSupportLimits() for the specified
 * operand, down to and including rank 0 (scalar).
 *
 * Iterating the full under-min range ([0, rankMin)) — rather than testing
 * only rankMin-1 — mirrors validateOperandRankTooLarge and catches
 * operators that partially reject out-of-range ranks (e.g. reject rankMin-1
 * but silently accept rankMin-2). Every accepted-but-unsupported rank is a
 * potential native crash surface.
 *
 * The test is skipped (returns early) when:
 *  - the operator is absent from opSupportLimits, or
 *  - none of the operand's supported data types are globally creatable.
 *
 * When rankMin is 0 the loop below is empty, so no assertion runs.
 *
 * @param {string} operatorName - The operator name (key in opSupportLimits).
 * @param {string} operandKey - The operand whose rank is under test.
 * @param {function(MLGraphBuilder, MLOperand): void} buildFn - Receives a
 *   fresh builder and the already-created out-of-range input. Should call the
 *   operator and return its output (or throw TypeError).
 * @param {function(number): number[]} buildShape - Returns the shape to
 *   create the operand with for the rank under test. Defaults to
 *   `buildExampleShape`.
 */
function validateOperandRankTooSmall(operatorName, operandKey, buildFn,
                                     buildShape = buildExampleShape) {
  promise_test(
      async t => {
        const opLimits = context.opSupportLimits()[operatorName];
        if (!opLimits)
          return;
        const {min: rankMin} = opLimits[operandKey].rankRange;
        const dataTypeInfo = pickSupportedDataTypeInfo(opLimits[operandKey]);
        if (!dataTypeInfo)
          return;
        // Test every rank in [0, rankMin). Iterating the full range (rather
        // than only rankMin-1) surfaces any rank the operator wrongly accepts
        // between 0 and its declared min — mirroring
        // validateOperandRankTooLarge.
        for (let rank = 0; rank < rankMin; rank++) {
          const shape = buildShape(rank);
          const builder = new MLGraphBuilder(context);
          const input =
              buildInputOperand(builder, operandKey, dataTypeInfo, shape);
          let thrown;
          try {
            buildFn(builder, input);
          } catch (e) {
            thrown = e;
          }
          assert_true(thrown instanceof TypeError,
                      `${operandKey} rank ${rank} (< min ${
                          rankMin}) should be rejected`);
        }
      },
      `[${operatorName}] throw if ${
          operandKey} rank is below min rank allowed by opSupportLimits`);
}

/**
 * Validate that an operator accepts every rank within the inclusive range
 * [min, max] allowed by context.opSupportLimits() for the specified operand.
 *
 * This complements validateOperandRankTooLarge / validateOperandRankTooSmall
 * by exercising all of the supported ranks in between (and including) the
 * boundaries, ensuring no valid rank is spuriously rejected.
 *
 * The test is skipped (returns early) when:
 *  - the operator is absent from opSupportLimits, or
 *  - none of the operand's supported data types are globally creatable, or
 *  - the operand's max rank exceeds the global input max rank (such a rank
 *    cannot be created).
 *
 * @param {string} operatorName - The operator name (key in opSupportLimits).
 * @param {string} operandKey - The operand whose rank is under test
 *   (e.g. 'input', 'a').
 * @param {function(MLGraphBuilder, MLOperand): MLOperand} buildFn - Receives a
 *   fresh builder and an in-range input. Should call the operator with valid
 *   remaining operands and return its output.
 * @param {function(number): number[]} buildShape - Returns the shape to
 *   create the operand with for the rank under test. Defaults to
 *   `buildExampleShape`.
 */
function validateOperandRankInRange(operatorName, operandKey, buildFn,
                                    buildShape = buildExampleShape) {
  promise_test(
      async t => {
        const opLimits = context.opSupportLimits()[operatorName];
        if (!opLimits)
          return;
        const {min: rankMin, max: rankMax} = opLimits[operandKey].rankRange;
        const dataTypeInfo = pickSupportedDataTypeInfo(opLimits[operandKey]);
        if (!dataTypeInfo)
          return;
        for (let rank = rankMin; rank <= rankMax; rank++) {
          const shape = buildShape(rank);
          const builder = new MLGraphBuilder(context);
          const input =
              buildInputOperand(builder, operandKey, dataTypeInfo, shape);
          const output = buildFn(builder, input);
          assert_true(output instanceof MLOperand,
                      `${operandKey} rank ${rank} should be accepted`);
        }
      },
      `[${operatorName}] accept all ${
          operandKey} ranks allowed by opSupportLimits`);
}

/**
 * Emit the full trio of rank-validation tests — below-min, in-range, and
 * above-max — for a single-operand rank check. Prefer this wrapper over
 * calling `validateOperandRankTooSmall` / `validateOperandRankInRange` /
 * `validateOperandRankTooLarge` individually: it guarantees all three regions
 * are covered so a later spec change to `rankRange.min` doesn't silently drop
 * the below-min check.
 *
 * `buildFn` must satisfy the union of the three contracts (see the individual
 * helpers above). In practice this means:
 *  - When invoked with an in-range input, it should call the operator and
 *    return its output (an MLOperand), or return `null` to signal
 *    "not creatable on this backend".
 *  - When invoked with an out-of-range input, it should call the operator;
 *    the operator throws TypeError, which the helpers catch and assert.
 *
 * The individual helpers remain available (and are still needed) for
 * asymmetric multi-operand ops (e.g. `matmul`, `where`) whose different
 * operands need different rank checks or share a Cartesian in-range test.
 *
 * @param {string} operatorName - The operator name (key in opSupportLimits).
 * @param {string} operandKey - The operand whose rank is under test.
 * @param {function(MLGraphBuilder, MLOperand): (MLOperand|null)} buildFn -
 *   Receives a fresh builder and the input at the rank under test.
 * @param {function(number): number[]} buildShape - Returns the shape to
 *   create the operand with for the rank under test. Defaults to
 *   `buildExampleShape`; override it when the uniform [kExampleDimSize, ...]
 *   shape cannot exercise the operator (e.g. `expand`, which only broadcasts
 *   dimensions of size 1).
 */
function validateOperandRank(operatorName, operandKey, buildFn,
                             buildShape = buildExampleShape) {
  validateOperandRankOutOfRange(operatorName, operandKey, buildFn, buildShape);
  validateOperandRankInRange(operatorName, operandKey, buildFn, buildShape);
}

/**
 * Emit both out-of-range rank checks — below-min and above-max — for a
 * single-operand rank check. Prefer this wrapper over calling
 * `validateOperandRankTooSmall` / `validateOperandRankTooLarge` individually
 * for asymmetric multi-operand ops (e.g. `where`, `matmul`) whose in-range
 * coverage is provided by a separate Cartesian `promise_test` and would be
 * duplicated by the full `validateOperandRank` wrapper.
 *
 * @param {string} operatorName - The operator name (key in opSupportLimits).
 * @param {string} operandKey - The operand whose rank is under test.
 * @param {function(MLGraphBuilder, MLOperand): (MLOperand|null)} buildFn -
 *   Receives a fresh builder and the out-of-range input. Should call the
 *   operator; the operator throws TypeError, which the helpers catch and
 *   assert.
 * @param {function(number): number[]} buildShape - Returns the shape to
 *   create the operand with for the rank under test. Defaults to
 *   `buildExampleShape`.
 */
function validateOperandRankOutOfRange(operatorName, operandKey, buildFn,
                                       buildShape = buildExampleShape) {
  validateOperandRankTooSmall(operatorName, operandKey, buildFn, buildShape);
  validateOperandRankTooLarge(operatorName, operandKey, buildFn, buildShape);
}
