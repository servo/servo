'use strict';

const ExecutionArray = ['sync', 'async'];

// https://webmachinelearning.github.io/webnn/#enumdef-mloperandtype
const TypedArrayDict = {
  float32: Float32Array,
  int32: Int32Array,
  uint32: Uint32Array,
  int8: Int8Array,
  uint8: Uint8Array,
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
 * Get exptected data and data type from given resources with output name.
 * @param {Array} resources - An array of expected resources
 * @param {String} outputName - An output name
 * @returns {Array.<[Number[], String]>} An array of expected data array and data type
 */
const getExpectedDataAndType = (resources, outputName) => {
  let ret;
  for (let subResources of resources) {
    if (subResources.name === outputName) {
      ret = [subResources.data, subResources.type];
      break;
    }
  }
  if (ret === undefined) {
    throw new Error(`Failed to get expected data sources and type by ${outputName}`);
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
 * Get ULP tolerance of averagePool2d operation.
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
  const reducedElementCount = sizes.reduce(
                                  (accumulator, currentValue) => accumulator * currentValue
  );
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

// Refer to precision metrics on https://github.com/webmachinelearning/webnn/issues/265#issuecomment-1256242643
const PrecisionMetrics = {
  batchNormalization: {ULP: {float32: 6, float16: 6}},
  clamp: {ULP: {float32: 0, float16: 0}},
  concat: {ULP: {float32: 0, float16: 0}},
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
  // Begin Element-wise unary operations
  abs: {ULP: {float32: 0, float16: 0}},
  ceil: {ULP: {float32: 0, float16: 0}},
  cos: {ATOL: {float32: 1/1024, float16: 1/512}},
  exp: {ULP: {float32: 32, float16: 1}},
  floor: {ULP: {float32: 0, float16: 0}},
  log: {ATOL: {float32: 1/1024, float16:  1/1024}},
  neg: {ULP: {float32: 0, float16: 0}},
  sin: {ATOL: {float32: 1/1024, float16: 1/512}},
  tan: {ATOL: {float32: 1/1024, float16: 1/512}},
  // End Element-wise unary operations
  elu: {ULP: {float32: 18, float16: 18}},
  gemm: {ULP: {float32: getGemmPrecisionTolerance, float16: getGemmPrecisionTolerance}},
  hardSigmoid: {ULP: {float32: 2, float16: 2}},
  hardSwish: {ULP: {float32: 4, float16: 4}},
  leakyRelu: {ULP: {float32: 1, float16: 1}},
  linear: {ULP: {float32: 2, float16: 2}},
  matmul: {ULP: {float32: getMatmulPrecisionTolerance, float16: getMatmulPrecisionTolerance}},
  pad: {ULP: {float32: 0, float16: 0}},
  // Begin Pooling operations
  averagePool2d: {ULP: {float32: getAveragePool2dPrecisionTolerance, float16: getAveragePool2dPrecisionTolerance}},
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
  reshape: {ULP: {float32: 0, float16: 0}},
  sigmoid: {ULP: {float32: 32+2, float16: 3}}, // float32 (leaving a few ULP for roundoff)
  slice: {ULP: {float32: 0, float16: 0}},
  softmax: {ULP: {float32: getSoftmaxPrecisionTolerance, float16: getSoftmaxPrecisionTolerance}},
  softplus: {ULP: {float32: 18, float16: 18}},
  softsign: {ULP: {float32: 3, float16: 3}},
  split: {ULP: {float32: 0, float16: 0}},
  squeeze: {ULP: {float32: 0, float16: 0}},
  tanh: {ATOL: {float32: 1/1024, float16: 1/512}},
  transpose: {ULP: {float32: 0, float16: 0}},
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
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperandtype
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
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperandtype
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
      actualBitwise = getBitwise(actual[i], dataType);
      expectedBitwise = getBitwise(expected[i], dataType);
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
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperandtype
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
      outputData = outputs[operandName];
      // for some operations which may have multi outputs of different types
      [expectedData, operandType] = getExpectedDataAndType(expected, operandName);
      tolerance = getPrecisonTolerance(operationName, metricType, resources);
      doAssert(operationName, outputData, expectedData, tolerance, operandType, metricType)
    }
  } else {
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
  const bufferView = new TypedArrayDict[resources.type](resources.data);
  return builder.constant({type: resources.type, dimensions: resources.shape}, bufferView);
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
  return builder.input(inputOperandName, {type: inputResources.type, dimensions: inputResources.shape});
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
    let operand;
    if (resources.inputs[inputOperandName].hasOwnProperty('constant') && resources.inputs[inputOperandName]['constant']) {
      operand = createConstantOperand(builder, resources.inputs[inputOperandName]);
    } else {
      operand = createSingleInputOperand(builder, resources, inputOperandName);
    }
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
const buildOperationWithTwoInputs= (operationName, builder, resources) => {
  // For example: MLOperand matmul(MLOperand a, MLOperand b);
  const namedOutputOperand = {};
  const [inputOperandA, inputOperandB] = createMultiInputOperands(builder, resources);
  const outputOperand = resources.options ?
      builder[operationName](inputOperandA, inputOperandB, resources.options) : builder[operationName](inputOperandA, inputOperandB);
  namedOutputOperand[resources.expected.name] = outputOperand;
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
        inputs[subInput.name] = new TypedArrayDict[subInput.type](subInput.data);
      }
    }
  } else {
    for (let inputName in resources.inputs) {
      const subTestByName = resources.inputs[inputName];
      if (!subTestByName.hasOwnProperty('constant') || !subTestByName.constant) {
        inputs[inputName] = new TypedArrayDict[subTestByName.type](subTestByName.data);
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
 * Build a graph, synchronously compile graph and execute, then check computed results.
 * @param {String} operationName - An operation name
 * @param {MLContext} context - A ML context
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @param {Function} buildFunc - A build function for an operation
 */
const runSync = (operationName, context, builder, resources, buildFunc) => {
  // build a graph
  const [namedOutputOperands, inputs, outputs] = buildGraph(operationName, builder, resources, buildFunc);
  // synchronously compile the graph up to the output operand
  const graph = builder.buildSync(namedOutputOperands);
  // synchronously execute the compiled graph.
  context.computeSync(graph, inputs, outputs);
  checkResults(operationName, namedOutputOperands, outputs, resources);
};

/**
 * Build a graph, asynchronously compile graph and execute, then check computed results.
 * @param {String} operationName - An operation name
 * @param {MLContext} context - A ML context
 * @param {MLGraphBuilder} builder - A ML graph builder
 * @param {Object} resources - Resources used for building a graph
 * @param {Function} buildFunc - A build function for an operation
 */
const run = async (operationName, context, builder, resources, buildFunc) => {
  // build a graph
  const [namedOutputOperands, inputs, outputs] = buildGraph(operationName, builder, resources, buildFunc);
  // asynchronously compile the graph up to the output operand
  const graph = await builder.build(namedOutputOperands);
  // asynchronously execute the compiled graph
  const result = await context.compute(graph, inputs, outputs);
  checkResults(operationName, namedOutputOperands, result.outputs, resources);
};

/**
 * Run WebNN operation tests.
 * @param {(String[]|String)} operationName - An operation name array or an operation name
 * @param {Function} buildFunc - A build function for an operation
 */
const testWebNNOperation = (operationName, buildFunc) => {
  let operationNameArray;
  if (typeof operationName === 'string') {
    operationNameArray = [operationName];
  } else if (Array.isArray(operationName)) {
    operationNameArray = operationName;
  }

  ExecutionArray.forEach(executionType => {
    const isSync = executionType === 'sync';
    if (self.GLOBAL.isWindow() && isSync) {
      return;
    }
    let context;
    let builder;
    if (isSync) {
      // test sync
      operationNameArray.forEach((subOperationName) => {
        const tests = loadTests(subOperationName);
        setup(() => {
          context = navigator.ml.createContextSync();
          builder = new MLGraphBuilder(context);
        });
        for (const subTest of tests) {
          test(() => {
            runSync(subOperationName, context, builder, subTest, buildFunc);
          }, `${subTest.name} / ${executionType}`);
        }
      });
    } else {
      // test async
      operationNameArray.forEach((subOperationName) => {
        const tests = loadTests(subOperationName);
        promise_setup(async () => {
          context = await navigator.ml.createContext();
          builder = new MLGraphBuilder(context);
        });
        for (const subTest of tests) {
          promise_test(async () => {
            await run(subOperationName, context, builder, subTest, buildFunc);
          }, `${subTest.name} / ${executionType}`);
        }
      });
    }
  });
};