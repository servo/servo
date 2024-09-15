'use strict';

// https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype
const TypedArrayDict = {
  float32: Float32Array,

  // Proposal to add float16 TypedArrays to JavaScript.
  // URL: https://tc39.es/proposal-float16array/
  // Use workaround Uint16 for Float16
  float16: Uint16Array,

  int64: BigInt64Array,
  uint64: BigUint64Array,
  int32: Int32Array,
  uint32: Uint32Array,
  int8: Int8Array,
  uint8: Uint8Array,
};

const kIntTypes = ['uint8', 'int8', 'uint32', 'int32', 'uint64', 'int64'];
const kFloatTypes = ['float16', 'float32'];

const findCompatibleType = (dataType, supportedTypes) => {
  for (let supportedType of supportedTypes) {
    if (kIntTypes.includes(dataType)) {
      if (kIntTypes.indexOf(supportedType) > kIntTypes.indexOf(dataType)) {
        return supportedType;
      }
    }

    if (kFloatTypes.includes(dataType)) {
      if (kFloatTypes.indexOf(supportedType) > kFloatTypes.indexOf(dataType)) {
        return supportedType;
      }
    }
  }
  return null;
};

// The maximum index to validate for the output's expected value.
const kMaximumIndexToValidate = 1000;

const kContextOptionsForVariant = {
  cpu: {
    deviceType: 'cpu',
  },
  gpu: {
    deviceType: 'gpu',
  },
  npu: {
    deviceType: 'npu',
  },
};

const variant = location.search.substring(1);
const contextOptions = kContextOptionsForVariant[variant];

const assertDescriptorsEquals = (outputOperand, expected) => {
  const dataType =
      expected.castedType ? expected.castedType : expected.dataType;
  assert_true(
      outputOperand.dataType() === dataType,
      'actual output dataType should be equal to expected output dataType');
  assert_array_equals(
      outputOperand.shape(), expected.dimensions,
      'actual output dimesnisons should be equal to expected output dimensions');
};

// ref:
// http://stackoverflow.com/questions/32633585/how-do-you-convert-to-half-floats-in-javascript
const toHalf = (value) => {
  let floatView = new Float32Array(1);
  let int32View = new Int32Array(floatView.buffer);

  /* This method is faster than the OpenEXR implementation (very often
   * used, eg. in Ogre), with the additional benefit of rounding, inspired
   * by James Tursa's half-precision code. */

  floatView[0] = value;
  let x = int32View[0];

  let bits = (x >> 16) & 0x8000; /* Get the sign */
  let m = (x >> 12) & 0x07ff;    /* Keep one extra bit for rounding */
  let e = (x >> 23) & 0xff;      /* Using int is faster here */

  /* If zero, or denormal, or exponent underflows too much for a denormal
   * half, return signed zero. */
  if (e < 103) {
    return bits;
  }

  /* If NaN, return NaN. If Inf or exponent overflow, return Inf. */
  if (e > 142) {
    bits |= 0x7c00;
    /* If exponent was 0xff and one mantissa bit was set, it means NaN,
     * not Inf, so make sure we set one mantissa bit too. */
    bits |= ((e == 255) ? 0 : 1) && (x & 0x007fffff);
    return bits;
  }

  /* If exponent underflows but not too much, return a denormal */
  if (e < 113) {
    m |= 0x0800;
    /* Extra rounding may overflow and set mantissa to 0 and exponent
     * to 1, which is OK. */
    bits |= (m >> (114 - e)) + ((m >> (113 - e)) & 1);
    return bits;
  }

  bits |= ((e - 112) << 10) | (m >> 1);
  /* Extra rounding. An overflow will set mantissa to 0 and increment
   * the exponent, which is OK. */
  bits += m & 1;
  return bits;
};

const getTypedArrayData = (type, size, data) => {
  let outData;

  if (type === 'float16') {
    if (typeof (data) === 'number' && size > 1) {
      return new TypedArrayDict[type](size).fill(toHalf(data));
    }
    // workaround to convert Float16 to Uint16
    outData = new TypedArrayDict[type](data.length);
    for (let i = 0; i < data.length; i++) {
      outData[i] = toHalf(data[i]);
    }
  } else if (type === 'int64') {
    if (typeof (data) === 'number' && size > 1) {
      return new TypedArrayDict[type](size).fill(BigInt(data));
    }
    outData = new TypedArrayDict[type](data.length);
    for (let i = 0; i < data.length; i++) {
      outData[i] = BigInt(data[i]);
    }
  } else {
    if (typeof (data) === 'number' && size > 1) {
      return new TypedArrayDict[type](size).fill(data);
    }
    outData = new TypedArrayDict[type](data);
  }
  return outData;
};

const sizeOfShape = (array) => {
  return array.reduce((accumulator, currentValue) => accumulator * currentValue, 1);
};

/**
 * Get bitwise of the given value.
 * @param {Number} value
 * @param {String} dataType - A data type string, like "float32", "float16",
 *     more types, please see:
 *     https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype
 * @return {Number} A 64-bit signed integer.
 */
const getBitwise = (value, dataType) => {
  const buffer = new ArrayBuffer(8);
  const int64Array = new BigInt64Array(buffer);
  int64Array[0] = value < 0 ? ~BigInt(0) : BigInt(0);
  let typedArray;
  if (dataType === "float32") {
    typedArray = new Float32Array(buffer);
  } else {
    throw new AssertionError(`Data type ${dataType} is not supported`);
  }
  typedArray[0] = value;
  return int64Array[0];
};

/**
 * Assert that each array property in ``actual`` is a number being close enough
 * to the corresponding property in ``expected`` by the acceptable ULP distance
 * ``nulp`` with given ``dataType`` data type.
 *
 * @param {Array} actual - Array of test values.
 * @param {Array} expected - Array of values expected to be close to the values
 *     in ``actual``.
 * @param {Number} nulp - A BigInt value indicates acceptable ULP distance.
 * @param {String} dataType - A data type string, value: "float32",
 *     more types, please see:
 *     https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype
 * @param {String} description - Description of the condition being tested.
 */
const assert_array_approx_equals_ulp = (actual, expected, nulp, dataType, description) => {
  /*
    * Test if two primitive arrays are equal within acceptable ULP distance
    */
  assert_true(
      actual.length === expected.length,
      `assert_array_approx_equals_ulp: ${description} lengths differ, ` +
          `expected ${expected.length} but got ${actual.length}`);
  let actualBitwise, expectedBitwise, distance;
  for (let i = 0; i < actual.length; i++) {
    if (actual[i] === expected[i]) {
      continue;
    } else {
      // measure the ULP distance
      if (dataType === 'float32') {
        actualBitwise = getBitwise(actual[i], dataType);
        expectedBitwise = getBitwise(expected[i], dataType);
      } else if (dataType === 'float16') {
        actualBitwise = actual[i];
        // convert expected data of Float16 to Uint16
        expectedBitwise = toHalf(expected[i]);
      } else if (dataType === 'int64') {
        actualBitwise = actual[i];
        expectedBitwise = BigInt(expected[i]);
      } else if (dataType === 'uint64') {
        actualBitwise = actual[i];
        expectedBitwise = BigUint64Array(expected[i]);
      } else if (
          dataType === 'int8' || dataType === 'uint8' || dataType === 'int32' ||
          dataType === 'uint32') {
        actualBitwise = actual[i];
        expectedBitwise = expected[i];
      }
      distance = actualBitwise - expectedBitwise;
      distance = distance >= 0 ? distance : -distance;

      // if true, invoke assert_true() in failure case
      // if false, it's expected, not invoke assert_true() in success case to
      // prevent spammy output
      if (distance > nulp) {
        assert_true(
            false,
            `assert_array_approx_equals_ulp: ${description} actual ` +
                `${actual[i]} should be close enough to expected ` +
                `${expected[i]} by the acceptable ${nulp} ULP distance, ` +
                `but they have ${distance} ULP distance`);
      }
    }
  }
};

/**
 * Assert actual results with expected results.
 * @param {String} operatorName
 * @param {(Number[]|Number)} actual
 * @param {(Number[]|Number)} expected
 * @param {String} metricType - Value: 'ULP', 'ATOL'
 * @param {Number} toleranceValue
 * @param {String} dataType  - An operand type string, value: "float32",
 *     more types, please see:
 *     https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype
 */
const doAssert =
    (operatorName, actual, expected, metricType, toleranceValue, dataType) => {
      const description = `test ${operatorName} ${dataType}`;
      if (typeof expected === 'number') {
        expected = [expected];
        actual = [actual];
      }
      if (metricType === 'ULP') {
        assert_array_approx_equals_ulp(
            actual, expected, toleranceValue, dataType, description);
      } else if (metricType === 'ATOL') {
        assert_array_approx_equals(
            actual, expected, toleranceValue, description);
      } else {
        throw new AssertionError(
            `Tolerance Metric type '${metricType}' is not supported`);
      }
    };

/**
 * Assert computed results be equal to expected data.
 * @param {Object} toleranceFunc
 * @param {Object.<MLNamedArrayBufferViews> |
 *     Array[Object.<MLNamedArrayBufferViews>]} actual
 * @param {Object} graphResources - Resources used for building a graph
 */
const assertResultsEquals = (toleranceFunc, actual, graphResources) => {
  const operatorName =
      graphResources.operators.map(operator => operator.name).join(' ');
  const expectedOutputs = graphResources.expectedOutputs;
  const toleranceInfo = toleranceFunc(graphResources);
  const metricType = toleranceInfo.metricType;
  const toleranceValue = toleranceInfo.value;
  let outputData;

  for (let operandName in actual) {
    const expectedSuboutput = expectedOutputs[operandName];
    const expectedDescriptor = expectedSuboutput.descriptor;
    let expectedData = expectedSuboutput.data;

    outputData = actual[operandName];
    // If data is scalar and shape is not, it means it's expecting to be
    // filled by the scalar value. Also limit the array size so it doesn't
    // timeout.
    if (typeof (expectedData) === 'number' && expectedDescriptor.dimensions &&
        sizeOfShape(expectedDescriptor.dimensions) > 1) {
      const size = Math.min(
          kMaximumIndexToValidate, sizeOfShape(expectedDescriptor.dimensions));
      expectedData = new Array(size).fill(expectedData);
      outputData = outputData.subarray(0, kMaximumIndexToValidate);
    }
    doAssert(
        operatorName, outputData, expectedData, metricType, toleranceValue,
        expectedDescriptor.dataType);
  }
};

const createOperand = (context, builder, operandName, resources) => {
  let operand;
  const descriptor = resources.descriptor;
  const dataType = descriptor.dataType;

  // If input data type is not supported on current platform, attempt to use
  // a supported type to pass the data, then cast back to original type.
  if (!context.opSupportLimits().input.dataTypes.includes(dataType)) {
    const compatibleType =
        findCompatibleType(dataType, context.opSupportLimits().input.dataTypes);
    if (compatibleType) {
      descriptor.castedType = compatibleType;
      descriptor.dataType = compatibleType;
    }
  }

  operand = resources.constant ?
      builder.constant(
          descriptor,
          getTypedArrayData(
              descriptor.dataType, sizeOfShape(descriptor.dimensions),
              resources.data)) :
      builder.input(operandName, descriptor);

  if (descriptor.castedType) {
    operand = builder.cast(operand, dataType);
  }

  return operand;
};

const prepareInputsForGraph = (inputs, resources) => {
  for (let operandName of Object.keys(resources)) {
    const inputOperandResources = resources[operandName];
    if (!inputOperandResources.constant) {
      inputs[operandName] = getTypedArrayData(
          inputOperandResources.descriptor.castedType ?
              inputOperandResources.descriptor.castedType :
              inputOperandResources.descriptor.dataType,
          sizeOfShape(inputOperandResources.descriptor.dimensions),
          inputOperandResources.data);
    }
  }
};

const prepareOutputsForGraph = (outputs, resources) => {
  for (let operandName of Object.keys(resources)) {
    const descriptor = resources[operandName].descriptor;
    const dataType =
        descriptor.castedType ? descriptor.castedType : descriptor.dataType;
    outputs[operandName] =
        new TypedArrayDict[dataType](sizeOfShape(descriptor.dimensions));
  }
};

/**
 * This function is to compile the constructed graph and compute.
 * @param {MLContext} context
 * @param {MLGraphBuilder} builder
 * @param {{
 *           inputs: Map<String, {
 *                                 data: Array.<Number>|Number,
 *                                 descriptor: MLOperandDescriptor,
 *                                 constant?: Boolean
 *                               }>,
 *           operators: Array.<{
 *                               name: String,
 *                               arguments: Array.<Map<String, Object>> ,
 *                               outputs: Array.<String>|String
 *                             }>,
 *           expectedOutputs: Map<String, {
 *                                          data: Array.<Number>|Number,
 *                                          descriptor: MLOperandDescriptor,
 *                                        }>
 *        }} graphResources - Resources used for building a graph
 * @returns A Promise of MLComputeResult.
 */
const buildGraphAndCompute = async (context, builder, graphResources) => {
  let outputOperands;
  const graphInputs = graphResources.inputs;
  const graphOperators = graphResources.operators;

  if (graphOperators.length === 1) {
    // For a test graph with a single operator
    const argumentArray = [];

    for (const argument of graphOperators[0].arguments) {
      for (const argumentName in argument) {
        if (argumentName !== 'options') {
          if (graphInputs.hasOwnProperty(argument[argumentName])) {
            const operandName = argument[argumentName];
            const operand = createOperand(
                context, builder, operandName, graphInputs[operandName]);
            argumentArray.push(operand);
          } else {
            argumentArray.push(argument[argumentName]);
          }
        } else {
          for (const [optionalArgumentName, value] of Object.entries(
                   argument['options'])) {
            if (typeof value === 'string' &&
                graphInputs.hasOwnProperty(value)) {
              const operandName = value;
              const operand = createOperand(
                  context, builder, operandName, graphInputs[operandName]);
              argument['options'][optionalArgumentName] = operand;
            }
          }
          argumentArray.push(argument['options']);
        }
      }
    }
    outputOperands = builder[graphOperators[0].name](...argumentArray);
  } else {
    // For a test graph with multiple operators
    // TODO: https://issues.chromium.org/issues/333756077
  }

  if (!Array.isArray(outputOperands)) {
    outputOperands = [outputOperands];
  }

  for (let i = 0; i < outputOperands.length; ++i) {
    const expectedDescriptor =
        graphResources
            .expectedOutputs[Object.keys(graphResources.expectedOutputs)[i]]
            .descriptor;
    if (!context.opSupportLimits().output.dataTypes.includes(
            expectedDescriptor.dataType)) {
      const compatibleType = findCompatibleType(
          expectedDescriptor.dataType,
          context.opSupportLimits().output.dataTypes);
      outputOperands[i] = builder.cast(outputOperands[i], compatibleType);
      expectedDescriptor.castedType = compatibleType;
    }
  }

  const outputNameArray = Object.keys(graphResources.expectedOutputs);
  for (let i = 0; i < outputOperands.length; ++i) {
    assertDescriptorsEquals(
        outputOperands[i],
        graphResources.expectedOutputs[outputNameArray[i]].descriptor);
  }

  const namedOutputOperand = {};
  outputNameArray.forEach(
      (name, index) => namedOutputOperand[name] = outputOperands[index]);

  // Compile the constructed graph.
  const graph = await builder.build(namedOutputOperand);

  const inputs = {};
  prepareInputsForGraph(inputs, graphInputs);

  const outputs = {};
  prepareOutputsForGraph(outputs, graphResources.expectedOutputs);

  // Execute the compiled graph.
  const result = await context.compute(graph, inputs, outputs);
  return result;
};

const getConv2dPrecisionTolerance = (graphResources) => {
  // number of reduced input elements multiplied by filter and summed (a sliding
  // dot product like pooling)
  const operatorResources = graphResources.operators[0];
  const operatorName = operatorResources.name;
  const args = operatorResources.arguments;
  const inputShape = graphResources.inputs[args[0][Object.keys(args[0])[0]]]
                         .descriptor.dimensions;
  const filterShape = graphResources.inputs[args[1][Object.keys(args[1])[0]]]
                          .descriptor.dimensions;
  const options =
      args.length === 3 ? {...args[2][Object.keys(args[2])[0]]} : {};
  let inputChannels = inputShape[1];  // default nchw inputLayout
  // default oihw filterLayout for conv2d or default iohw filterLayout for
  // convTranspose2d
  let filterWidth = filterShape[3];
  let filterHeight = filterShape[2];
  const groups = options.groups ? options.groups : 1;

  if (options.inputLayout) {
    if (!['nchw', 'nhwc'].includes(options.inputLayout)) {
      throw new Error(`Unknown inputLayout ${options.inputLayout}`);
    }
    inputChannels =
        options.inputLayout === 'nchw' ? inputChannels : inputShape[3];
  }
  if (options.filterLayout) {
    let filterLayouts = ['oihw', 'hwio', 'ohwi', 'ihwo'];  // default for conv2d
    if (operatorName === 'convTranspose2d') {
      filterLayouts = ['iohw', 'hwoi', 'ohwi'];
    }
    if (!filterLayouts.includes(options.filterLayout)) {
      throw new Error(`Unknown filterLayout ${options.filterLayout}`);
    }
    switch (options.filterLayout) {
      case 'oihw':
      case 'iohw':
        // Just use the existing filterWidth and filterHeight above.
        break;
      case 'hwio':
      case 'hwoi':
        filterWidth = filterShape[1];
        filterHeight = filterShape[0];
        break;
      case 'ohwi':
      case 'ihwo':
        filterWidth = filterShape[2];
        filterHeight = filterShape[1];
        break;
      default:
        break;
    }
  }

  const tolerance = filterWidth * filterHeight * (inputChannels / groups) * 2;
  const toleranceValueDict = {float32: tolerance, float16: tolerance};
  const expectedDataType =
      getExpectedDataTypeOfSingleOutput(graphResources.expectedOutputs);
  return {metricType: 'ULP', value: toleranceValueDict[expectedDataType]};
};

const getExpectedDataTypeOfSingleOutput = (expectedOutput) => {
  const expectedDescriptor =
      expectedOutput[Object.keys(expectedOutput)[0]].descriptor;
  const dataType = expectedDescriptor.castedType ?
      expectedDescriptor.castedType :
      expectedDescriptor.dataType;
  return dataType;
};

const getReducedElementCount =
    (graphResources) => {
      const args = graphResources.operators[0].arguments;
      const inputShape = graphResources.inputs[args[0][Object.keys(args[0])[0]]]
                             .descriptor.dimensions;
      const rank = inputShape.length;
      const options =
          args.length === 2 ? {...args[1][Object.keys(args[1])[0]]} : {};
      let sizes;

      if (options && options.axes) {
        sizes = options.axes.map(
            (axis) => axis < 0 ? inputShape[axis + rank] : inputShape[axis]);
      } else {
        sizes = inputShape;
      }

      return sizes.length ?
          sizes.reduce(
              (accumulator, currentValue) => accumulator * currentValue) :
          1;
    }

const webnn_conformance_test =
    (buildGraphAndComputeFunc, toleranceFunc, testResources) => {
      promise_test(async () => {
        let context;
        try {
          context = await navigator.ml.createContext(contextOptions);
        } catch (e) {
          throw new AssertionError(
              `Unable to create context for ${variant} variant. ${e}`);
        }
        const builder = new MLGraphBuilder(context);
        const result = await buildGraphAndComputeFunc(
            context, builder, testResources.graph);
        assertResultsEquals(toleranceFunc, result.outputs, testResources.graph);
      }, testResources.name);
    };
