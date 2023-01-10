'use strict';

const ExecutionArray = ['sync', 'async'];

// https://webmachinelearning.github.io/webnn/#enumdef-mldevicetype
const DeviceTypeArray = ['cpu', 'gpu'];

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
 * Get JSON resources from specified test resources file.
 * @param {String} file - A test resources file path
 * @returns {Object} Test resources
 */
const loadResources = (file) => {
  const loadJSON = () => {
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

  const json = loadJSON();
  return JSON.parse(json.replace(/\\"|"(?:\\"|[^"])*"|(\/\/.*|\/\*[\s\S]*?\*\/)/g, (m, g) => g ? "" : m));
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
 * Get ULP tolerance of softmax operation.
 * @param {Object} resources - Resources used for building a graph
 * @returns {Number} A tolerance number
 */
const getSoftmaxPrecisionTolerance = (resources) => {
  // Compute the softmax values of the 2-D input tensor along axis 1.
  const inputShape = resources.inputs[Object.keys(resources.inputs)[0]].shape;
  const tolerance = inputShape[1] * 3 + 3;
  return tolerance;
};

// Refer to precision metrics on https://github.com/webmachinelearning/webnn/issues/265#issuecomment-1256242643
const PrecisionMetrics = {
  clamp: {ULP: {float32: 0, float16: 0}},
  concat: {ULP: {float32: 0, float16: 0}},
  leakyRelu: {ULP: {float32: 1, float16: 1}},
  relu: {ULP: {float32: 0, float16: 0}},
  reshape: {ULP: {float32: 0, float16: 0}},
  sigmoid: {ULP: {float32: 32+2, float16: 3}}, // float32 (leaving a few ULP for roundoff)
  slice: {ULP: {float32: 0, float16: 0}},
  softmax: {ULP: {float32: getSoftmaxPrecisionTolerance, float16: getSoftmaxPrecisionTolerance}},
  split: {ULP: {float32: 0, float16: 0}},
  squeeze: {ULP: {float32: 0, float16: 0}},
  tanh: {ATOL: {float32: 1/1024, float16: 1/512}},
  transpose: {ULP: {float32: 0, float16: 0}},
};

/**
 * Get precison tolerance value.
 * @param {String} operationName - An operation name
 * @param {String} metricType - Value: 'ULP', 'ATOL'
 * @param {String} precisionType - A precision type string, like "float32", "float16",
 *     more types, please see:
 *     https://webmachinelearning.github.io/webnn/#enumdef-mloperandtype
 * @returns {Number} A tolerance number
 */
const getPrecisonTolerance = (operationName, metricType, precisionType) => {
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
    actualBitwise = getBitwise(actual[i], dataType);
    expectedBitwise = getBitwise(expected[i], dataType);
    distance = actualBitwise - expectedBitwise;
    distance = distance >= 0 ? distance : -distance;
    assert_true(distance <= nulp,
                `assert_array_approx_equals_ulp: ${description} actual ${actual[i]} should be close enough to expected ${expected[i]} by the acceptable ${nulp} ULP distance, but they have ${distance} ULP distance`);
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
      tolerance = getPrecisonTolerance(operationName, metricType, operandType);
      doAssert(operationName, outputData, expectedData, tolerance, operandType, metricType)
    }
  } else {
    outputData = outputs[expected.name];
    expectedData = expected.data;
    operandType = expected.type;
    tolerance = getPrecisonTolerance(operationName, metricType, operandType);
    doAssert(operationName, outputData, expectedData, tolerance, operandType, metricType)
  }
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
      inputs[subInput.name] = new TypedArrayDict[subInput.type](subInput.data);
    }
  } else {
    for (let inputName in resources.inputs) {
      const subTestByName = resources.inputs[inputName];
      inputs[inputName] = new TypedArrayDict[subTestByName.type](subTestByName.data);
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
  await context.compute(graph, inputs, outputs);
  checkResults(operationName, namedOutputOperands, outputs, resources);
};

/**
 * Run WebNN operation tests.
 * @param {String} operationName - An operation name
 * @param {String} file - A test resources file path
 * @param {Function} buildFunc - A build function for an operation
 */
const testWebNNOperation = (operationName, file, buildFunc) => {
  const resources = loadResources(file);
  const tests = resources.tests;
  ExecutionArray.forEach(executionType => {
    const isSync = executionType === 'sync';
    if (self.GLOBAL.isWindow() && isSync) {
      return;
    }
    let context;
    let builder;
    if (isSync) {
      // test sync
      DeviceTypeArray.forEach(deviceType => {
        setup(() => {
          context = navigator.ml.createContextSync({deviceType});
          builder = new MLGraphBuilder(context);
        });
        for (const subTest of tests) {
          test(() => {
            runSync(operationName, context, builder, subTest, buildFunc);
          }, `${subTest.name} / ${deviceType} / ${executionType}`);
        }
      });
    } else {
      // test async
      DeviceTypeArray.forEach(deviceType => {
        promise_setup(async () => {
          context = await navigator.ml.createContext({deviceType});
          builder = new MLGraphBuilder(context);
        });
        for (const subTest of tests) {
          promise_test(async () => {
            await run(operationName, context, builder, subTest, buildFunc);
          }, `${subTest.name} / ${deviceType} / ${executionType}`);
        }
      });
    }
  });
};