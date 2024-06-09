'use strict';

// https://webmachinelearning.github.io/webnn/#enumdef-mloperanddatatype
const TypedArrayDict = {
  // workaround use Uint16 for Float16
  float16: Uint16Array,

  float32: Float32Array,
  int32: Int32Array,
  uint32: Uint32Array,
  int8: Int8Array,
  uint8: Uint8Array,
  int64: BigInt64Array,
};

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

// The maximum index to validate for the output's expected value.
const kMaximumIndexToValidate = 1000;

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
 * Get tests resources from test data JSON file of specified operation name.
 * @param {String} operationName - An operation name
 * @returns {Object} Tests resources
 */
const loadTests = (operationName) => {
  const loadJSON = (file) => {
    let xmlhttp = new XMLHttpRequest();
    xmlhttp.open("GET", file, false);
    xmlhttp.overrideMimeType("application/json");
    xmlhttp.send();
    if (xmlhttp.status == 200 && xmlhttp.readyState == 4) {
      return xmlhttp.responseText;
    } else {
      throw new Error(`Failed to load ${file}`);
    }
  };

  const capitalLetterMatches = operationName.match(/[A-Z]/g);
  if (capitalLetterMatches !== null) {
    // for example: the test data JSON file for leakyRelu is leaky_relu.json and for reduceLogSum is reduce_log_sum.json
    capitalLetterMatches.forEach(
      capitalLetter => operationName = operationName.replace(capitalLetter, `_${capitalLetter.toLowerCase()}`)
    )
  }
  const json = loadJSON(`/webnn/resources/test_data/${operationName}.json`);
  const resources = JSON.parse(json.replace(/\\"|"(?:\\"|[^"])*"|(\/\/.*|\/\*[\s\S]*?\*\/)/g, (m, g) => g ? "" : m));
  return resources.tests;
};

/**
 * Get expected resource from given resources with output name.
 * @param {Array} resources - An array of given resources
 * @param {String} outputName - An output name
 * @returns {Object} An object of expected resource
 */
const getNamedResource = (resources, outputName) => {
  let ret;
  for (let resource of resources) {
    if (resource.name === outputName) {
      ret = resource;
      break;
    }
  }
  if (ret === undefined) {
    throw new Error(`Failed to get expected resource by ${outputName}`);
  }
  return ret;
};


/**
 * Get ULP tolerance of conv2d/convTranspose2d operation.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getConv2dPrecisionTolerance = (resources, operationName) => {
  // number of reduced input elements multiplied by filter and summed (a sliding dot product like pooling)
  const inputNameArray = Object.keys(resources.inputs);
  const inputShape = resources.inputs[inputNameArray[0]].shape;
  const filterShape = resources.inputs[inputNameArray[1]].shape;
  const options = resources.options;
  let groups = 1;
  let inputChannels = inputShape[1]; // default nchw inputLayout
  // default oihw filterLayout for conv2d or default iohw filterLayout for convTranspose2d
  let filterWidth = filterShape[3];
  let filterHeight = filterShape[2];
  if (options) {
    if (options.groups) {
      groups = options.groups;
    }
    if (options.inputLayout) {
      if (!['nchw', 'nhwc'].includes(options.inputLayout)) {
        throw new Error(`Unsupported inputLayout ${options.inputLayout}`);
      }
      inputChannels = options.inputLayout === 'nchw' ? inputChannels : inputShape[3];
    }
    if (options.filterLayout) {
      let filterLayouts = ['oihw', 'hwio', 'ohwi', 'ihwo']; // default for conv2d
      if (operationName === 'convTranspose2d') {
        filterLayouts = ['iohw', 'hwoi', 'ohwi'];
      }
      if (!filterLayouts.includes(options.filterLayout)) {
        throw new Error(`Unsupported filterLayout ${options.filterLayout}`);
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
  }
  const tolerance = filterWidth * filterHeight * (inputChannels / groups) * 2;
  return tolerance;
};

/**
 * Get ULP tolerance of gemm operation.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getGemmPrecisionTolerance = (resources, operationName) => {
  // GEMM : alpha * (A x B) + beta * C
  // An upper bound for the worst serial ordering is bounded by
  // the number of lossy operations, where matrix multiplication
  // is a dot product (mul and add times the number of elements)
  // plus bias operations.
  const shapeA = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  const options = {...resources.options};
  const width = options.aTranspose ? shapeA[0] : shapeA[1];
  let tolerance = width * 2;
  // default options.alpha is 1.0
  if (options.alpha !== undefined && options.alpha !== 1.0) {
    tolerance++;
  }
  if (options.c && options.beta !== 0.0) {
    // default options.beta is 1.0
    if (options.beta !== undefined && options.beta !== 1.0) {
      tolerance++;
    }
    tolerance++;
  }
  return tolerance;
};

/**
 * Get ULP tolerance of matmul operation.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getMatmulPrecisionTolerance = (resources, operationName) => {
  // Matmul : Compute the matrix product of two input tensors.
  // If a is 1-D, WebNN converts it to a 2-D tensor by prepending a 1 to its dimensions, [n] -> [1, n].
  // So we can just always check the last dimension here.
  const shapeA = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  const tolerance = shapeA[shapeA.length - 1] * 2;
  return tolerance;
};

/**
 * Get ULP tolerance of averagePool2d or l2Pool2d operation.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getAveragePool2dPrecisionTolerance = (resources, operationName) => {
  const inputShape = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  let height;
  let width;
  const options = {...resources.options};
  if (options.windowDimensions) {
    height = options.windowDimensions[0];
    width = options.windowDimensions[1];
  } else {
    // If not present, the window dimensions are assumed to be the height and width dimensions of the input shape
    if (options.layout && options.layout === 'nhwc') {
      height = inputShape[1];
      width = inputShape[2];
    } else {
      // nhwc layout of input
      height = inputShape[2];
      width = inputShape[3];
    }
  }

  const tolerance = height * width + 2;
  return tolerance;
};

/**
 * Get ULP tolerance of softmax operation.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getSoftmaxPrecisionTolerance = (resources, operationName) => {
  // Compute the softmax values of the 2-D input tensor along axis 1.
  const inputShape = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  const tolerance = inputShape[1] * 3 + 3;
  return tolerance;
};

/**
 * Get ULP tolerance of reduction operations.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getReductionPrecisionTolerance = (resources, operationName) => {
  const inputShape = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  const rank = inputShape.length;
  const options = {...resources.options};
  let sizes;
  if (options && options.axes) {
    sizes = options.axes.map(
                (axis) => axis < 0 ? inputShape[axis + rank] : inputShape[axis]
    );
  } else {
    sizes = inputShape;
  }
  const reducedElementCount = sizes.length ?
      sizes.reduce((accumulator, currentValue) => accumulator * currentValue) :
      1;
  let tolerance;
  switch (operationName) {
    case 'reduceL1':
    case 'reduceProduct':
    case 'reduceSum':
      tolerance = reducedElementCount;
      break;
    case 'reduceL2':
      tolerance = reducedElementCount * 2 + 1;
      break;
    case 'reduceMean':
      tolerance = reducedElementCount + 2;
      break;
    case 'reduceLogSum':
      tolerance = reducedElementCount + 18;
      break;
    case 'reduceLogSumExp':
      tolerance = reducedElementCount * 2 + 18;
      break;
    case 'reduceSumSquare':
      tolerance = reducedElementCount * 2;
      break;
    default:
      break;
  }
  return tolerance;
};

/**
 * Get ULP tolerance of resample2d operations.
 * @param {Object} resources - Resources used for building a graph
 * @param {String} operationName - An operation name
 * @returns {Number} A tolerance number
 */
const getResample2dPrecisionTolerance = (resources, operationName) => {
  const options = {...resources.options};
  let tolerance;
  if (options.mode && options.mode === 'linear') {
    // interpolation mode is linear
    const precisionType = resources.expected.type;
    if (precisionType === 'float32') {
      tolerance = 84;
    } else if (precisionType === 'float16') {
      tolerance = 10;
    } else {
      tolerance = 1;
    }
  } else {
    // interpolation mode is nearest-neighbor
    tolerance = 0;
  }
  return tolerance;
};

// Refer to precision metrics on https://github.com/webmachinelearning/webnn/issues/265#issuecomment-1256242643
const PrecisionMetrics = {
  argMax: {ULP: {int64: 0}},
  argMin: {ULP: {int64: 0}},
  batchNormalization: {ULP: {float32: 6, float16: 6}},
  cast: {ULP: {float32: 1, float16: 1, int32: 0, uint32: 0, int64: 0, int8: 0, uint8: 0}},
  clamp: {ULP: {float32: 0, float16: 0}},
  concat: {ULP: {float32: 0, float16: 0}},
  constant: {ULP: {float32: 2, float16: 2, int32: 0, uint32: 0, int64: 0, int8: 0, uint8: 0}},
  conv2d: {ULP: {float32: getConv2dPrecisionTolerance, float16: getConv2dPrecisionTolerance}},
  convTranspose2d: {ULP: {float32: getConv2dPrecisionTolerance, float16: getConv2dPrecisionTolerance}},
  // Begin Element-wise binary operations
  add: {ULP: {float32: 1, float16: 1}},
  sub: {ULP: {float32: 1, float16: 1}},
  mul: {ULP: {float32: 1, float16: 1}},
  div: {ULP: {float32: 2, float16: 2}},
  max: {ULP: {float32: 0, float16: 0}},
  min: {ULP: {float32: 0, float16: 0}},
  pow: {ULP: {float32: 32, float16: 2}},
  // End Element-wise binary operations
  // Begin Element-wise logical operations
  equal: {ULP: {uint8: 0}},
  greater: {ULP: {uint8: 0}},
  greaterOrEqual: {ULP: {uint8: 0}},
  lesser: {ULP: {uint8: 0}},
  lesserOrEqual: {ULP: {uint8: 0}},
  logicalNot: {ULP: {uint8: 0}},
  // End Element-wise logical operations
  // Begin Element-wise unary operations
  abs: {ULP: {float32: 0, float16: 0}},
  ceil: {ULP: {float32: 0, float16: 0}},
  cos: {ATOL: {float32: 1/1024, float16: 1/512}},
  erf: {ATOL: {float32: 1/1024, float16: 1/512}},
  exp: {ULP: {float32: 32, float16: 1}},
  floor: {ULP: {float32: 0, float16: 0}},
  identity: {ULP: {float32: 0, float16: 0}},
  log: {ATOL: {float32: 1/1024, float16:  1/1024}},
  neg: {ULP: {float32: 0, float16: 0}},
  reciprocal: {ULP: {float32: 2, float16: 2}},
  sin: {ATOL: {float32: 1/1024, float16: 1/512}},
  sqrt: {ULP: {float32: 1, float16: 1}},
  tan: {ATOL: {float32: 1/1024, float16: 1/512}},
  // End Element-wise unary operations
  elu: {ULP: {float32: 18, float16: 18}},
  expand: {ULP: {float32: 0, float16: 0}},
  gather: {ULP: {float32: 0, float16: 0}},
  gemm: {ULP: {float32: getGemmPrecisionTolerance, float16: getGemmPrecisionTolerance}},
  instanceNormalization: {ULP: {float32: 840, float16: 8400}},
  hardSigmoid: {ULP: {float32: 2, float16: 2}},
  hardSwish: {ULP: {float32: 4, float16: 4}},
  layerNormalization: {ATOL: {float32: 1/1024, float16: 1/512}},
  leakyRelu: {ULP: {float32: 1, float16: 1}},
  linear: {ULP: {float32: 2, float16: 2}},
  matmul: {ULP: {float32: getMatmulPrecisionTolerance, float16: getMatmulPrecisionTolerance}},
  pad: {ULP: {float32: 0, float16: 0}},
  // Begin Pooling operations
  averagePool2d: {ULP: {float32: getAveragePool2dPrecisionTolerance, float16: getAveragePool2dPrecisionTolerance}},
  l2Pool2d: {ULP: {float32: getAveragePool2dPrecisionTolerance, float16: getAveragePool2dPrecisionTolerance}},
  maxPool2d: {ULP: {float32: 0, float16: 0}},
  // End Pooling operations
  prelu: {ULP: {float32: 1, float16: 1}},
  // Begin Reduction operations
  reduceL1: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceL2: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceLogSum: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceLogSumExp: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceMax: {ULP: {float32: 0, float16: 0}},
  reduceMean: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceMin: {ULP: {float32: 0, float16: 0}},
  reduceProduct: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceSum: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  reduceSumSquare: {ULP: {float32: getReductionPrecisionTolerance, float16: getReductionPrecisionTolerance}},
  // End Reduction operations
  relu: {ULP: {float32: 0, float16: 0}},
  resample2d: {ULP: {float32: getResample2dPrecisionTolerance, float16: getResample2dPrecisionTolerance}},
  reshape: {ULP: {float32: 0, float16: 0}},
  sigmoid: {ULP: {float32: 32+2, float16: 3}}, // float32 (leaving a few ULP for roundoff)
  slice: {ULP: {float32: 0, float16: 0}},
  softmax: {ULP: {float32: getSoftmaxPrecisionTolerance, float16: getSoftmaxPrecisionTolerance}},
  softplus: {ULP: {float32: 18, float16: 18}},
  softsign: {ULP: {float32: 3, float16: 3}},
  split: {ULP: {float32: 0, float16: 0}},
  tanh: {ATOL: {float32: 1/1024, float16: 1/512}},
  transpose: {ULP: {float32: 0, float16: 0}},
  triangular: {ULP: {float32: 0, float16: 0}},
  where: {ULP: {float32: 0, float16: 0}},
};

/**
 * Get precison tolerance value.
 * @param {String} operationName - An operation name
 * @param {String} metricType - Value: 'ULP', 'ATOL'
 * @param {Object} resources - Resources used for building a graph
 * @returns {Number} A tolerance number
 */
const getPrecisonTolerance = (operationName, metricType, resources) => {
  // the outputs by split or gru is a sequence
  const precisionType = Array.isArray(resources.expected) ? resources.expected[0].type : resources.expected.type;
  let tolerance = PrecisionMetrics[operationName][metricType][precisionType];
  // If the tolerance is dynamic, then evaluate the function to get the value.
  if (tolerance instanceof Function) {
    tolerance = tolerance(resources, operationName);
  }
  return tolerance;
};

/**
 * Get bitwise of the given value.
 * @param {Number} value
 * @param {String} dataType - A data type string, like "float32", "float16",
 *     more types, please see:
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperanddatatype
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
 * Assert that each array property in ``actual`` is a number being close enough to the corresponding
 * property in ``expected`` by the acceptable ULP distance ``nulp`` with given ``dataType`` data type.
 *
 * @param {Array} actual - Array of test values.
 * @param {Array} expected - Array of values expected to be close to the values in ``actual``.
 * @param {Number} nulp - A BigInt value indicates acceptable ULP distance.
 * @param {String} dataType - A data type string, value: "float32",
 *     more types, please see:
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperanddatatype
 * @param {String} description - Description of the condition being tested.
 */
const assert_array_approx_equals_ulp = (actual, expected, nulp, dataType, description) => {
  /*
    * Test if two primitive arrays are equal within acceptable ULP distance
    */
  assert_true(actual.length === expected.length,
              `assert_array_approx_equals_ulp: ${description} lengths differ, expected ${expected.length} but got ${actual.length}`);
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
      }
      distance = actualBitwise - expectedBitwise;
      distance = distance >= 0 ? distance : -distance;
      assert_true(distance <= nulp,
                  `assert_array_approx_equals_ulp: ${description} actual ${actual[i]} should be close enough to expected ${expected[i]} by the acceptable ${nulp} ULP distance, but they have ${distance} ULP distance`);
    }
  }
};

/**
 * Assert actual results with expected results.
 * @param {String} operationName - An operation name
 * @param {(Number[]|Number)} actual
 * @param {(Number[]|Number)} expected
 * @param {Number} tolerance
 * @param {String} operandType  - An operand type string, value: "float32",
 *     more types, please see:
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperanddatatype
 * @param {String} metricType - Value: 'ULP', 'ATOL'
 */
const doAssert = (operationName, actual, expected, tolerance, operandType, metricType) => {
  const description = `test ${operationName} ${operandType}`;
  if (typeof expected === 'number') {
    // for checking a scalar output by matmul 1D x 1D
    expected = [expected];
    actual = [actual];
  }
  if (metricType === 'ULP') {
    assert_array_approx_equals_ulp(actual, expected, tolerance, operandType, description);
  } else if (metricType === 'ATOL') {
    assert_array_approx_equals(actual, expected, tolerance, description);
  }
};

/**
 * Check computed results with expected data.
 * @param {String} operationName - An operation name
 * @param {Object.<String, MLOperand>} namedOutputOperands
 * @param {Object.<MLNamedArrayBufferViews>} outputs - The resources of required outputs
 * @param {Object} resources - Resources used for building a graph
 */
const checkResults = (operationName, namedOutputOperands, outputs, resources) => {
  const metricType = Object.keys(PrecisionMetrics[operationName])[0];
  const expected = resources.expected;
  let tolerance;
  let operandType;
  let outputData;
  let expectedData;
  if (Array.isArray(expected)) {
    // the outputs of split() or gru() is a sequence
    for (let operandName in namedOutputOperands) {
      const suboutputResource = getNamedResource(expected, operandName);
      assert_array_equals(namedOutputOperands[operandName].shape(), suboutputResource.shape ?? []);
      outputData = outputs[operandName];
      tolerance = getPrecisonTolerance(operationName, metricType, resources);
      doAssert(operationName, outputData, suboutputResource.data, tolerance, suboutputResource.type, metricType)
    }
  } else {
    assert_array_equals(namedOutputOperands[expected.name].shape(), expected.shape ?? []);
    outputData = outputs[expected.name];
    expectedData = expected.data;
    operandType = expected.type;
    tolerance = getPrecisonTolerance(operationName, metricType, resources);
    doAssert(operationName, outputData, expectedData, tolerance, operandType, metricType)
  }
};

/**
 * Create a constant operand
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for constant operand
 * @returns {MLOperand} A constant operand
 */
const createConstantOperand = (builder, resources) => {
  const bufferView = (typeof (resources.data) === 'number' &&
                      sizeOfShape(resources.shape) > 1) ?
      new TypedArrayDict[resources.type](sizeOfShape(resources.shape))
          .fill(resources.data) :
      new TypedArrayDict[resources.type](resources.data);
  return builder.constant({dataType: resources.type, type: resources.type, dimensions: resources.shape}, bufferView);
};

/**
 * Create single input operands for a graph.
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @param {String} [inputOperandName] - An inputOperand name
 * @returns {MLOperand} An input operand
 */
const createSingleInputOperand = (builder, resources, inputOperandName) => {
  inputOperandName = inputOperandName ? inputOperandName : Object.keys(resources.inputs)[0];
  const inputResources = resources.inputs[inputOperandName];
  let operand;
  if (resources.inputs[inputOperandName].hasOwnProperty('constant') && resources.inputs[inputOperandName]['constant']) {
    operand = createConstantOperand(builder, resources.inputs[inputOperandName]);
  } else {
    operand = builder.input(inputOperandName, {dataType: inputResources.type, type: inputResources.type, dimensions: inputResources.shape});
  }
  return operand;
};

/**
 * Create multi input operands for a graph.
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @returns {MLOperand[]} Input operands array
 */
const createMultiInputOperands = (builder, resources) => {
  let inputOperands = [];
  const inputOperandNameArray = Object.keys(resources.inputs);
  inputOperandNameArray.forEach(inputOperandName => {
    const operand = createSingleInputOperand(builder, resources, inputOperandName);
    inputOperands.push(operand);
  });
  return inputOperands;
};

/**
 * Build an operation which has a single input.
 * @param {String} operationName - An operation name
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @returns {MLNamedOperands}
 */
const buildOperationWithSingleInput = (operationName, builder, resources) => {
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  const outputOperand = resources.options ?
      builder[operationName](inputOperand, resources.options) : builder[operationName](inputOperand);
  namedOutputOperand[resources.expected.name] = outputOperand;
  return namedOutputOperand;
};

/**
 * Build an operation which has two inputs.
 * @param {String} operationName - An operation name
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @returns {MLNamedOperands}
 */
const buildOperationWithTwoInputs = (operationName, builder, resources) => {
  // For example: MLOperand matmul(MLOperand a, MLOperand b);
  const namedOutputOperand = {};
  const [inputOperandA, inputOperandB] = createMultiInputOperands(builder, resources);
  const outputOperand = resources.options ?
      builder[operationName](inputOperandA, inputOperandB, resources.options) : builder[operationName](inputOperandA, inputOperandB);
  namedOutputOperand[resources.expected.name] = outputOperand;
  return namedOutputOperand;
};

const buildBatchNorm = (operationName, builder, resources) => {
  // MLOperand batchNormalization(MLOperand input, MLOperand mean, MLOperand variance,
  //                              optional MLBatchNormalizationOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, meanOperand, varianceOperand] = createMultiInputOperands(builder, resources);
  const batchNormOptions = {...resources.options};
  if (batchNormOptions.scale) {
    batchNormOptions.scale = createConstantOperand(builder, batchNormOptions.scale);
  }
  if (batchNormOptions.bias) {
    batchNormOptions.bias = createConstantOperand(builder, batchNormOptions.bias);
  }
  // invoke builder.batchNormalization()
  namedOutputOperand[resources.expected.name] =
      builder[operationName](inputOperand, meanOperand, varianceOperand, batchNormOptions);
  return namedOutputOperand;
};

const buildCast = (operationName, builder, resources) => {
  // MLOperand cast(MLOperand input, MLOperandDataType type);
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.cast()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.type);
  return namedOutputOperand;
};

const buildConcat = (operationName, builder, resources) => {
  // MLOperand concat(sequence<MLOperand> inputs, unsigned long axis);
  const namedOutputOperand = {};
  const inputOperands = [];
  let operand;
  for (let input of resources.inputs) {
    if (input.hasOwnProperty('constant') && input['constant']) {
      operand = createConstantOperand(builder, input);
    } else {
      operand = builder.input(input.name, {dataType: input.type, type: input.type, dimensions: input.shape});
    }
    inputOperands.push(operand);
  }
  // invoke builder.concat()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperands, resources.axis);
  return namedOutputOperand;
};

const buildConstantRange = (operationName, builder, resources) => {
  const namedOutputOperand = {};
  // invoke builder.constant(start, step, outputShape, type)
  namedOutputOperand[resources.expected.name] = builder[operationName](resources.inputs.start, resources.inputs.step, resources.outputShape, resources.type);
  return namedOutputOperand;
};

const buildConvTranspose2d = (operationName, builder, resources) => {
  // MLOperand convTranspose2d(MLOperand input, MLOperand filter, optional MLConvTranspose2dOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, filterOperand] = createMultiInputOperands(builder, resources);
  let convTranspose2dOptions = {...resources.options};
  if (convTranspose2dOptions.bias) {
    convTranspose2dOptions.bias = createConstantOperand(builder, convTranspose2dOptions.bias);
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, filterOperand, convTranspose2dOptions);
  return namedOutputOperand;
};

const buildConv2d = (operationName, builder, resources) => {
  // MLOperand conv2d(MLOperand input, MLOperand filter, optional MLConv2dOptions options = {});
  const namedOutputOperand = {};
  const [inputOperand, filterOperand] = createMultiInputOperands(builder, resources);
  let conv2dOptions = {...resources.options};
  if (conv2dOptions.bias) {
    conv2dOptions.bias = createConstantOperand(builder, conv2dOptions.bias);
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, filterOperand, conv2dOptions);
  return namedOutputOperand;
};

const buildGemm = (operationName, builder, resources) => {
  // MLOperand gemm(MLOperand a, MLOperand b, optional MLGemmOptions options = {});
  const namedOutputOperand = {};
  const [inputOperandA, inputOperandB] = createMultiInputOperands(builder, resources);
  let gemmOptions = {...resources.options};
  if (gemmOptions.c) {
    if (gemmOptions.c.shape) {
      gemmOptions.c = createConstantOperand(builder, gemmOptions.c);
    } else {
      // MLOperand c;
      // Create a single-value operand when c is a scalar
      gemmOptions.c = builder.constant({dataType: 'float32', type: 'float32', dimensions: [1]}, new Float32Array([gemmOptions.c]));
    }
  }
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperandA, inputOperandB, gemmOptions);
  return namedOutputOperand;
};

const buildLayerNorm = (operationName, builder, resources) => {
  // MLOperand layerNormalization(MLOperand input, optional MLLayerNormalizationOptions options = {});
  // MLOperand instanceNormalization(MLOperand input, optional MLInstanceNormalizationOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  const layerNormOptions = {...resources.options};
  if (layerNormOptions.scale) {
    layerNormOptions.scale = createConstantOperand(builder, layerNormOptions.scale);
  }
  if (layerNormOptions.bias) {
    layerNormOptions.bias = createConstantOperand(builder, layerNormOptions.bias);
  }
  // invoke builder.layerNormalization() or builder.instanceNormalization()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, layerNormOptions);
  return namedOutputOperand;
};

const buildPad = (operationName, builder, resources) => {
  // MLOperand pad(MLOperand input, sequence<unsigned long> beginningPadding, sequence<unsigned long> endingPadding, optional MLPadOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.pad()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.beginningPadding, resources.endingPadding, resources.options);
  return namedOutputOperand;
};

const buildReshape = (operationName, builder, resources) => {
  // MLOperand reshape(MLOperand input, sequence<unsigned long> newShape);
  // MLOperand expand(MLOperand input, sequence<unsigned long> newShape);
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.reshape() or builder.expand()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.newShape);
  return namedOutputOperand;
};

const buildSlice = (operationName, builder, resources) => {
  // MLOperand slice(MLOperand input, sequence<unsigned long> starts, sequence<unsigned long> sizes);
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.slice()
  namedOutputOperand[resources.expected.name] = builder[operationName](inputOperand, resources.starts, resources.sizes);
  return namedOutputOperand;
};

const buildSplit = (operationName, builder, resources) => {
  // sequence<MLOperand> split(MLOperand input,
  //                           (unsigned long or sequence<unsigned long>) splits,
  //                           optional MLSplitOptions options = {});
  const namedOutputOperand = {};
  const inputOperand = createSingleInputOperand(builder, resources);
  // invoke builder.split()
  const outputOperands = builder[operationName](inputOperand, resources.splits, resources.options);
  resources.expected.forEach((resourceDict, index) => {
    namedOutputOperand[resourceDict.name] = outputOperands[index];
  });
  return namedOutputOperand;
};

const buildWhere = (operationName, builder, resources) => {
  // MLOperand where(MLOperand condition, MLOperand trueValues, MLOperand falseValues);
  const namedOutputOperand = {};
  const [conditionOperand, trueValuesOperand, falseValuesOperand] = createMultiInputOperands(builder, resources);
  // invoke builder.where()
  namedOutputOperand[resources.expected.name] = builder[operationName](conditionOperand, trueValuesOperand, falseValuesOperand);
  return namedOutputOperand;
};

/**
 * Build a graph.
 * @param {String} operationName - An operation name
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @param {Function} buildFunc - A build function for an operation
 * @returns [namedOperands, inputs, outputs]
 */
const buildGraph = (operationName, builder, resources, buildFunc) => {
  const namedOperands = buildFunc(operationName, builder, resources);
  let inputs = {};
  if (Array.isArray(resources.inputs)) {
    // the inputs of concat() is a sequence
    for (let subInput of resources.inputs) {
      if (!subInput.hasOwnProperty('constant') || !subInput.constant) {
        inputs[subInput.name] = getTypedArrayData(
            subInput.type, sizeOfShape(subInput.shape), subInput.data);
      }
    }
  } else {
    for (let inputName in resources.inputs) {
      const subTestByName = resources.inputs[inputName];
      if (!subTestByName.hasOwnProperty('constant') || !subTestByName.constant) {
        inputs[inputName] = getTypedArrayData(
            subTestByName.type, sizeOfShape(subTestByName.shape),
            subTestByName.data);
      }
    }
  }
  let outputs = {};
  if (Array.isArray(resources.expected)) {
    // the outputs of split() or gru() is a sequence
    for (let i = 0; i < resources.expected.length; i++) {
      const subExpected = resources.expected[i];
      outputs[subExpected.name] = new TypedArrayDict[subExpected.type](sizeOfShape(subExpected.shape));
    }
  } else {
    // matmul 1D with 1D produces a scalar which doesn't have its shape
    const shape = resources.expected.shape ? resources.expected.shape : [1];
    outputs[resources.expected.name] = new TypedArrayDict[resources.expected.type](sizeOfShape(shape));
  }
  return [namedOperands, inputs, outputs];
};

/**
 * Build a graph, compile graph and execute, then check computed results.
 * @param {String} operationName - An operation name
 * @param {MLContext} context - A ML context
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @param {Function} buildFunc - A build function for an operation
 */
const run = async (operationName, context, builder, resources, buildFunc) => {
  // build a graph
  const [namedOutputOperands, inputs, outputs] = buildGraph(operationName, builder, resources, buildFunc);
  // compile the graph up to the output operand
  const graph = await builder.build(namedOutputOperands);
  // execute the compiled graph
  const result = await context.compute(graph, inputs, outputs);
  checkResults(operationName, namedOutputOperands, result.outputs, resources);
};

const variant = location.search.substring(1);
const contextOptions = kContextOptionsForVariant[variant];

/**
 * Checks if MLBuffer is implemented or not.
 * @param {MLContext} ml_context - A ML context to test for MLBuffer support.
 * @returns {Boolean} True if MLBuffer is supported; otherwise, False.
 */
const isMLBufferSupported =
    (ml_context) => {
      return (createBuffer(ml_context, 4) !== undefined);
    }

/**
 * Run WebNN operation tests.
 * @param {(String[]|String)} operationName - An operation name array or an
 *     operation name
 * @param {Function} buildFunc - A build function for an operation
 */
const testWebNNOperation = (operationName, buildFunc) => {
  let operationNameArray;
  if (typeof operationName === 'string') {
    operationNameArray = [operationName];
  } else if (Array.isArray(operationName)) {
    operationNameArray = operationName;
  }

  let context;
  let builder;
  operationNameArray.forEach((subOperationName) => {
    const tests = loadTests(subOperationName);
    promise_setup(async () => {
      let supported = false;
      try {
        context = await navigator.ml.createContext(contextOptions);
        supported = true;
      } catch (e) {
      }
      assert_implements(
          supported, `Unable to create context for ${variant} variant`);
      builder = new MLGraphBuilder(context);
    });
    for (const subTest of tests) {
      promise_test(async () => {
        await run(subOperationName, context, builder, subTest, buildFunc);
      }, `${subTest.name}`);
    }
  });
};

/**
 * WebNN parallel compute operation test.
 */
const testParallelCompute = () => {
  let ml_context;
  let ml_graph;

  promise_setup(async () => {
    let supported = false;
    try {
      ml_context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
    // Construct a simple graph: A = B * 2.
    const builder = new MLGraphBuilder(ml_context);
    const operandType = {dataType: 'float32', dimensions: [1]};
    const input_operand = builder.input('input', operandType);
    const const_operand = builder.constant(operandType, Float32Array.from([2]));
    const output_operand = builder.mul(input_operand, const_operand);
    ml_graph = await builder.build({'output': output_operand});
  });

  promise_test(async () => {
    const test_inputs = [1, 2, 3, 4];

    const actual_outputs = await Promise.all(test_inputs.map(async (input) => {
      let inputs = {'input': Float32Array.from([input])};
      let outputs = {'output': new Float32Array(1)};
      ({inputs, outputs} = await ml_context.compute(ml_graph, inputs, outputs));
      return outputs.output[0];
    }));

    const expected_outputs = [2, 4, 6, 8];
    assert_array_equals(actual_outputs, expected_outputs);
  });
};

/**
 * Run WebNN conformance tests by specified operation.
 * @param {(String[]|String)} operationName - An operation name array or an
 *     operation name
 * @param {Function} buildFunc - A build function for an operation
 */
const runWebNNConformanceTests = (operationName, buildFunc) => {
  // Link to https://github.com/web-platform-tests/wpt/pull/44883
  // Check navigator.ml is defined before trying to run WebNN tests
  if (navigator.ml) {
    testWebNNOperation(operationName, buildFunc);
  } else {
    // Show indication to users why the test failed
    test(
        () => assert_not_equals(
            navigator.ml, undefined, 'ml property is defined on navigator'));
  }
};

// ref: http://stackoverflow.com/questions/32633585/how-do-you-convert-to-half-floats-in-javascript
const toHalf = (value) => {
  let floatView = new Float32Array(1);
  let int32View = new Int32Array(floatView.buffer);

  /* This method is faster than the OpenEXR implementation (very often
   * used, eg. in Ogre), with the additional benefit of rounding, inspired
   * by James Tursa's half-precision code. */

  floatView[0] = value;
  let x = int32View[0];

  let bits = (x >> 16) & 0x8000; /* Get the sign */
  let m = (x >> 12) & 0x07ff; /* Keep one extra bit for rounding */
  let e = (x >> 23) & 0xff; /* Using int is faster here */

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


/**
 * WebNN buffer creation.
 * @param {MLContext} context - the context used to create the buffer.
 * @param {Number} bufferSize - Size of the buffer to create, in bytes.
 * @returns {MLBuffer} the created buffer.
 */
const createBuffer = (context, bufferSize) => {
  let buffer;
  try {
    buffer = context.createBuffer({size: bufferSize});
    assert_equals(buffer.size, bufferSize);
  } catch (e) {
    assert_true(e instanceof DOMException);
    assert_equals(e.name, "NotSupportedError");
  }
  return buffer;
};

/**
 * WebNN destroy buffer twice test.
 * @param {String} testName - The name of the test operation.
 */
const testDestroyWebNNBuffer = (testName) => {
  let context;
  let buffer;
  promise_setup(async () => {
    let supported = false;
    try {
      context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
    buffer = createBuffer(context, 4);
  });
  promise_test(async () => {
    // MLBuffer is not supported for this deviceType.
    if (buffer === undefined) {
      return;
    }
    buffer.destroy();
    buffer.destroy();
  }, `${testName}`);
};

/**
 * WebNN create buffer test.
 * @param {String} testName - The name of the test operation.
 * @param {Number} bufferSize - Size of the buffer to create, in bytes.
 */
const testCreateWebNNBuffer = (testName, bufferSize) => {
  let context;

  promise_setup(async () => {
    let supported = false;
    try {
      context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
  });
  promise_test(async () => {
    createBuffer(context, bufferSize);
  }, `${testName} / ${bufferSize}`);
};

/**
 * Asserts the buffer data in MLBuffer matches expected.
 * @param {MLContext} ml_context - The context used to create the buffer.
 * @param {MLBuffer} ml_buffer - The buffer to read and compare data.
 * @param {Array} expected - Array of the expected data in the buffer.
 */
const assert_buffer_data_equals = async (ml_context, ml_buffer, expected) => {
  const actual = await ml_context.readBuffer(ml_buffer);
  assert_array_equals(
      new expected.constructor(actual), expected,
      'Read buffer data equals expected data.');
};

/**
 * WebNN write buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testWriteWebNNBuffer = (testName) => {
  let ml_context;
  promise_setup(async () => {
    let supported = false;
    try {
      ml_context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
  });

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    let array_buffer = new ArrayBuffer(ml_buffer.size);

    // Writing with a size that goes past that source buffer length.
    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, new Uint8Array(array_buffer), /*srcOffset=*/ 0,
            /*srcSize=*/ ml_buffer.size + 1));
    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, new Uint8Array(array_buffer), /*srcOffset=*/ 3,
            /*srcSize=*/ 4));

    // Writing with a source offset that is out of range of the source size.
    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, new Uint8Array(array_buffer),
            /*srcOffset=*/ ml_buffer.size + 1));

    // Writing with a source offset that is out of range of implicit copy size.
    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, new Uint8Array(array_buffer),
            /*srcOffset=*/ ml_buffer.size + 1, /*srcSize=*/ undefined));

    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, new Uint8Array(array_buffer), /*srcOffset=*/ undefined,
            /*srcSize=*/ ml_buffer.size + 1));

    assert_throws_js(
        TypeError,
        () => ml_context.writeBuffer(
            ml_buffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE, 0xEE])));
  }, `${testName} / error`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Writing data to a destroyed MLBuffer should throw.
    ml_buffer.destroy();

    assert_throws_dom(
        'InvalidStateError',
        () =>
            ml_context.writeBuffer(ml_buffer, new Uint8Array(ml_buffer.size)));
  }, `${testName} / destroy`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    const array_buffer = new ArrayBuffer(ml_buffer.size);
    const detached_buffer = array_buffer.transfer();
    assert_true(array_buffer.detached, 'array buffer should be detached.');

    ml_context.writeBuffer(ml_buffer, array_buffer);
  }, `${testName} / detached`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    let another_ml_context = await navigator.ml.createContext(contextOptions);
    let another_ml_buffer = createBuffer(another_ml_context, ml_buffer.size);

    let input_data = new Uint8Array(ml_buffer.size).fill(0xAA);
    assert_throws_js(
        TypeError, () => ml_context.writeBuffer(another_ml_buffer, input_data));
    assert_throws_js(
        TypeError, () => another_ml_context.writeBuffer(ml_buffer, input_data));
  }, `${testName} / context_mismatch`);
};

/**
 * WebNN read buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testReadWebNNBuffer = (testName) => {
  let ml_context;
  promise_setup(async () => {
    let supported = false;
    try {
      ml_context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
  });

  promise_test(async t => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Reading a destroyed MLBuffer should reject.
    ml_buffer.destroy();

    await promise_rejects_dom(
        t, 'InvalidStateError', ml_context.readBuffer(ml_buffer));
  }, `${testName} / destroy`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Initialize the buffer.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    ml_context.writeBuffer(ml_buffer, Uint32Array.from([0xBBBBBBBB]));
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint32Array.from([0xBBBBBBBB]));
    ;
  }, `${testName} / full_size`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Initialize the buffer.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Writing to the remainder of the buffer from source offset.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xCC, 0xCC, 0xBB, 0xBB]),
        /*srcOffset=*/ 2);
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint8Array.from([0xBB, 0xBB, 0xAA, 0xAA]));
  }, `${testName} / src_offset_only`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Initialize the buffer.
    const input_data = [0xAA, 0xAA, 0xAA, 0xAA];
    ml_context.writeBuffer(ml_buffer, Uint8Array.from(input_data));

    // Writing zero bytes at the end of the buffer.
    ml_context.writeBuffer(
        ml_buffer, Uint32Array.from([0xBBBBBBBB]), /*srcOffset=*/ 1);
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint8Array.from(input_data));
  }, `${testName} / zero_write`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Initialize the buffer.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Writing with both a source offset and size.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xDD, 0xDD, 0xCC, 0xDD]),
        /*srcOffset=*/ 2, /*srcSize=*/ 1);
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint8Array.from([0xCC, 0xAA, 0xAA, 0xAA]));
  }, `${testName} / src_offset_and_size`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    // Initialize the buffer.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Using an offset allows a larger source buffer to fit.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE, 0xEE]),
        /*srcOffset=*/ 1);
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE]));
  }, `${testName} / larger_src_data`);

  promise_test(async () => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    const input_data = [0xAA, 0xAA, 0xAA, 0xAA];

    // Writing with a source offset of undefined should be treated as 0.
    ml_context.writeBuffer(
        ml_buffer, Uint8Array.from(input_data), /*srcOffset=*/ undefined,
        /*srcSize=*/ input_data.length);
    await assert_buffer_data_equals(
        ml_context, ml_buffer, Uint8Array.from(input_data));
  }, `${testName} / no_src_offset`);

  promise_test(async t => {
    let ml_buffer = createBuffer(ml_context, 4);

    // MLBuffer was unsupported for the deviceType.
    if (ml_buffer === undefined) {
      return;
    }

    let another_ml_context = await navigator.ml.createContext(contextOptions);
    let another_ml_buffer = createBuffer(another_ml_context, ml_buffer.size);

    await promise_rejects_js(
        t, TypeError, ml_context.readBuffer(another_ml_buffer));
    await promise_rejects_js(
        t, TypeError, another_ml_context.readBuffer(ml_buffer));
  }, `${testName} / context_mismatch`);
};

/**
 * WebNN dispatch buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testDispatchWebNNBuffer = (testName) => {
  let ml_context;
  let ml_graph;
  const shape = [3, 5];
  let inputs = {};
  let outputs = {};
  promise_setup(async () => {
    let supported = false;
    try {
      ml_context = await navigator.ml.createContext(contextOptions);
      supported = true;
    } catch (e) {
    }
    assert_implements(
        supported, `Unable to create context for ${variant} variant`);
    // Construct a simple graph: A = B + C, with two outputs.
    const builder = new MLGraphBuilder(ml_context);
    const operandType = {dataType: 'float32', dimensions: shape};
    const lhs_operand = builder.input('lhs', operandType);
    const rhs_operand = builder.input('rhs', operandType);
    const output_1_operand = builder.add(lhs_operand, rhs_operand);
    const output_2_operand = builder.add(lhs_operand, rhs_operand);
    ml_graph = await builder.build(
        {'output1': output_1_operand, 'output2': output_2_operand});
    const ml_buffer_size =
        TypedArrayDict['float32'].BYTES_PER_ELEMENT * sizeOfShape(shape);
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }
    inputs = {
      'lhs': ml_context.createBuffer({size: ml_buffer_size}),
      'rhs': ml_context.createBuffer({size: ml_buffer_size}),
    };
    outputs = {
      'output1': ml_context.createBuffer({size: ml_buffer_size}),
      'output2': ml_context.createBuffer({size: ml_buffer_size}),
    };
  });

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    let another_ml_context = await navigator.ml.createContext(contextOptions);

    // Control case, same context.
    ml_context.dispatch(ml_graph, inputs, outputs);

    // Test the wrong context being used for inputs.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs':
                  another_ml_context.createBuffer({size: inputs['lhs'].size()}),
              'rhs': inputs['rhs'],
            },
            outputs));

    // Test the wrong context being used for outputs.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1':
          another_ml_context.createBuffer({size: outputs['output1'].size()}),
      'output2': outputs['output2'],
    }));
  }, `${testName} / context_mismatch`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    // Control case, valid size.
    ml_context.dispatch(ml_graph, inputs, outputs);

    // Input is too large.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': ml_context.createBuffer({size: inputs['lhs'].size() + 1}),
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': inputs['lhs'],
              'rhs': ml_context.createBuffer({size: inputs['rhs'].size() + 1}),
            },
            outputs));

    // Output is too large.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': ml_context.createBuffer({size: outputs['output1'].size() + 1}),
      'output2': outputs['output2'],
    }));

    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': outputs['output1'],
      'output2': ml_context.createBuffer({size: outputs['output2'].size() + 1}),
    }));
  }, `${testName} / invalid_size`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    // Control case, valid names.
    ml_context.dispatch(ml_graph, inputs, outputs);

    // No names is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, {}, {}));

    // Input name is invalid.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'a_different_input_name': inputs['lhs'],
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': inputs['lhs'],
              'a_different_input_name': inputs['rhs'],
            },
            outputs));

    // Output name is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'a_different_output_name': outputs['output1'],
      'output2': outputs['output2'],
    }));

    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': outputs['output1'],
      'a_different_output_name': outputs['output2'],
    }));

    // Too few named inputs is invalid.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': inputs['lhs'],
            },
            outputs));

    // Too many named inputs is invalid.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': inputs['lhs'],
              'rhs': inputs['rhs'],
              'a_different_input_name':
                  ml_context.createBuffer({size: inputs['rhs'].size()}),
            },
            outputs));

    // Too few named outputs is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': outputs['output1']
    }));

    // Too many named outputs is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output2'],
      'a_different_output_name':
          ml_context.createBuffer({size: outputs['output2'].size()}),
    }));
  }, `${testName} / invalid_name`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    // Control case, valid buffers.
    ml_context.dispatch(ml_graph, inputs, outputs);

    // Same buffer used as outputs more than once is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output1'],
    }));

    // Same buffer used as input and output is invalid.
    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': inputs['lhs'],
      'output2': outputs['output2'],
    }));

    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': outputs['output1'],
              'rhs': inputs['rhs'],
            },
            outputs));

    // Buffer that does not exist is invalid.
    assert_throws_js(
        TypeError,
        () => ml_context.dispatch(
            ml_graph, {
              'lhs': undefined,
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(TypeError, () => ml_context.dispatch(ml_graph, inputs, {
      'output1': undefined,
      'output2': outputs['output2'],
    }));
  }, `${testName} / invalid_buffer`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatch_inputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatch_1_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    const dispatch_2_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const input_data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatch_inputs['lhs'], input_data);
    ml_context.writeBuffer(dispatch_inputs['rhs'], input_data);

    // Output_1 = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatch_inputs, dispatch_1_outputs);

    // Output_2 = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatch_inputs, dispatch_2_outputs);

    await assert_buffer_data_equals(
        ml_context, dispatch_1_outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        ml_context, dispatch_1_outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        ml_context, dispatch_2_outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        ml_context, dispatch_2_outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatch_1_inputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatch_2_inputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatch_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const input_1_data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatch_1_inputs['lhs'], input_1_data);
    ml_context.writeBuffer(dispatch_1_inputs['rhs'], input_1_data);

    const input_2_data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    ml_context.writeBuffer(dispatch_2_inputs['lhs'], input_2_data);
    ml_context.writeBuffer(dispatch_2_inputs['rhs'], input_2_data);

    // Output = LHS_1 + RHS_1 = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatch_1_inputs, dispatch_outputs);

    // Output = LHS_2 + RHS_2 = 2 + 2 = 4
    ml_context.dispatch(ml_graph, dispatch_2_inputs, dispatch_outputs);

    await assert_buffer_data_equals(
        ml_context, dispatch_outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));

    await assert_buffer_data_equals(
        ml_context, dispatch_outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));
  }, `${testName} / same_outputs`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatch_inputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatch_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const input_data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatch_inputs['lhs'], input_data);
    ml_context.writeBuffer(dispatch_inputs['rhs'], input_data);

    // Output = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatch_inputs, dispatch_outputs);
    ml_context.dispatch(ml_graph, dispatch_inputs, dispatch_outputs);

    await assert_buffer_data_equals(
        ml_context, dispatch_outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        ml_context, dispatch_outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs_and_outputs`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatch_inputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatch_1_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    const dispatch_2_outputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const input_data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatch_inputs['lhs'], input_data);
    ml_context.writeBuffer(dispatch_inputs['rhs'], input_data);

    // Output_1 = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatch_inputs, dispatch_1_outputs);

    // Output_2 = Output_1_LHS + Output_1_RHS = 2 + 2 = 4
    ml_context.dispatch(
        ml_graph, {
          'lhs': dispatch_1_outputs['output1'],
          'rhs': dispatch_1_outputs['output2'],
        },
        dispatch_2_outputs);

    // Output_1 = Output_2_LHS + Output_2_RHS = 4 + 4 = 8
    ml_context.dispatch(
        ml_graph, {
          'lhs': dispatch_2_outputs['output1'],
          'rhs': dispatch_2_outputs['output2'],
        },
        dispatch_1_outputs);

    await assert_buffer_data_equals(
        ml_context, dispatch_1_outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(8));

    await assert_buffer_data_equals(
        ml_context, dispatch_1_outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(8));
  }, `${testName} / outputs_as_inputs`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    // Construct a simple graph: OUTPUT = LHS - RHS.
    const builder = new MLGraphBuilder(ml_context);
    const operandType = {dataType: 'float32', dimensions: shape};
    const lhsOperand = builder.input('lhs', operandType);
    const rhsOperand = builder.input('rhs', operandType);
    const graph =
        await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

    const lhsBuffer = ml_context.createBuffer({size: inputs['lhs'].size});
    const rhsBuffer = ml_context.createBuffer({size: inputs['rhs'].size});

    const dispatchOutputs = {
      'output': ml_context.createBuffer({size: outputs['output1'].size})
    };

    // Initialize inputs
    ml_context.writeBuffer(
        lhsBuffer, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(5.0));
    ml_context.writeBuffer(
        rhsBuffer, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(3.0));

    // Output = LHS - RHS = 5 - 3 = 2
    ml_context.dispatch(
        graph, {
          'lhs': lhsBuffer,
          'rhs': rhsBuffer,
        },
        dispatchOutputs);

    await assert_buffer_data_equals(
        ml_context, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(2));

    // Output = RHS - LHS = 3 - 5 = -2
    ml_context.dispatch(
        graph, {
          'lhs': rhsBuffer,
          'rhs': lhsBuffer,
        },
        dispatchOutputs);

    await assert_buffer_data_equals(
        ml_context, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(-2));
  }, `${testName} / same name diff input buffers`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatchInputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const outputBuffer1 =
        ml_context.createBuffer({size: outputs['output1'].size});
    const outputBuffer2 =
        ml_context.createBuffer({size: outputs['output2'].size});

    // Initialize inputs
    const inputData1 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], inputData1);
    ml_context.writeBuffer(dispatchInputs['rhs'], inputData1);

    // Output = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatchInputs, {
      'output1': outputBuffer1,
      'output2': outputBuffer2,
    });

    // Output = LHS + RHS = 2 + 2 = 4
    const inputData2 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], inputData2);
    ml_context.writeBuffer(dispatchInputs['rhs'], inputData2);

    ml_context.dispatch(ml_graph, dispatchInputs, {
      'output1': outputBuffer1,
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    });

    // Ensure the last dispatch() did not modify the original second output
    // buffer.
    await assert_buffer_data_equals(
        ml_context, outputBuffer2,
        new Float32Array(sizeOfShape(shape)).fill(2));
  }, `${testName} / same name diff outputs buffers`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatchInputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatchOutputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], inputData);
    ml_context.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatchInputs, dispatchOutputs);

    // Check destroyed input buffers cannot be re-used in subsequent dispatches.
    dispatchInputs['lhs'].destroy();
    dispatchInputs['lhs'] = ml_context.createBuffer({size: inputs['lhs'].size});

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    ml_context.dispatch(ml_graph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        ml_context, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));

    dispatchInputs['rhs'].destroy();
    dispatchInputs['rhs'] = ml_context.createBuffer({size: inputs['rhs'].size});
    ml_context.writeBuffer(dispatchInputs['rhs'], newInputData);

    // Output = LHS + RHS = 2 + 2 = 4
    ml_context.dispatch(ml_graph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        ml_context, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4));
  }, `${testName} / same name diff inputs buffers destroy`);

  promise_test(async () => {
    // MLBuffer was unsupported for the deviceType.
    if (!isMLBufferSupported(ml_context)) {
      return;
    }

    const dispatchInputs = {
      'lhs': ml_context.createBuffer({size: inputs['lhs'].size}),
      'rhs': ml_context.createBuffer({size: inputs['rhs'].size}),
    };

    const dispatchOutputs = {
      'output1': ml_context.createBuffer({size: outputs['output1'].size}),
      'output2': ml_context.createBuffer({size: outputs['output2'].size}),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], inputData);
    ml_context.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    ml_context.dispatch(ml_graph, dispatchInputs, dispatchOutputs);

    // Check destroyed output buffers cannot be re-used in subsequent
    // dispatches.
    dispatchOutputs['output1'].destroy();
    dispatchOutputs['output1'] =
        ml_context.createBuffer({size: outputs['output1'].size});

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    ml_context.writeBuffer(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    ml_context.dispatch(ml_graph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        ml_context, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));
  }, `${testName} / same name diff outputs buffers destroy`);
};
