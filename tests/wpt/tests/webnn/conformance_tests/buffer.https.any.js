// META: title=test WebNN API buffer operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlbuffer

const bytesPerDataType = (dataType) => {
  if (dataType === 'int8' || dataType === 'uint8') {
    return 1;
  } else if (dataType === 'float16') {
    return 2;
  } else if (
      dataType === 'float32' || dataType === 'int32' || dataType === 'uint32') {
    return 4;
  } else if (dataType === 'int64' || dataType === 'uint64') {
    return 8;
  } else {
    throw new AssertionError(`Data type '${dataType}' is not supported`);
  }
};

const sizeOfDescriptor = (descriptor) => {
  return descriptor.dimensions.reduce(
      (accumulator, currentValue) => accumulator * currentValue,
      bytesPerDataType(descriptor.dataType));
};

const getDescriptorFromBuffer = (buffer) => {
  return {
    dataType: buffer.dataType,
    dimensions: buffer.shape,
    usage: buffer.usage
  };
};


/**
 * WebNN destroy buffer twice test.
 * @param {String} testName - The name of the test operation.
 */
const testDestroyWebNNBuffer = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlBuffer =
          await mlContext.createBuffer({dataType: 'int32', dimensions: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create buffer for ${variant} variant. ${e}`);
    }
  });
  promise_test(async () => {
    let mlBuffer =
        await mlContext.createBuffer({dataType: 'int32', dimensions: [2, 3]});
    mlBuffer.destroy();
    mlBuffer.destroy();
  }, `${testName}`);
};

/**
 * WebNN create buffer test.
 * @param {String} testName - The name of the test operation.
 * @param {MLTensorDescriptor} bufferDescriptor - The intended buffer specs.
 */
const testCreateWebNNBuffer = (testName, bufferDescriptor) => {
  let mlContext;

  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }
  });
  promise_test(async t => {
    if (!mlContext.opSupportLimits().input.dataTypes.includes(
            bufferDescriptor.dataType)) {
      await promise_rejects_js(
          t, TypeError, mlContext.createBuffer(bufferDescriptor));
      return;
    }

    const mlBuffer = await mlContext.createBuffer(bufferDescriptor);
    assert_equals(
        mlBuffer.dataType, bufferDescriptor.dataType,
        'buffer data types do not match');
    assert_array_equals(
        mlBuffer.shape, bufferDescriptor.dimensions,
        'buffer shapes do not match');
  }, `${testName} / ${bufferDescriptor.dataType}`);
};

/**
 * Same as above, but expect creating the buffer to fail.
 * @param {String} testName - The name of the test operation.
 * @param {MLTensorDescriptor} bufferDescriptor - The intended buffer specs.
 */
const testCreateWebNNBufferFails = (testName, bufferDescriptor) => {
  let mlContext;

  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }
  });
  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError, mlContext.createBuffer(bufferDescriptor));
  }, `${testName} / ${bufferDescriptor.dataType}`);
};

/**
 * Asserts the buffer data in MLTensor matches expected.
 * @param {MLContext} mlContext - The context used to create the buffer.
 * @param {MLTensor} mlBuffer - The buffer to read and compare data.
 * @param {Array} expected - Array of the expected data in the buffer.
 */
const assert_buffer_data_equals = async (mlContext, mlBuffer, expected) => {
  const actual = await mlContext.readBuffer(mlBuffer);
  assert_array_equals(
      new expected.constructor(actual), expected,
      'Read buffer data equals expected data.');
};

/**
 * WebNN write buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testWriteWebNNBuffer = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlBuffer =
          await mlContext.createBuffer({dataType: 'int32', dimensions: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create buffer for ${variant} variant. ${e}`);
    }
  });

  promise_test(async () => {
    const bufferDescriptor = {
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO,
    };
    let mlBuffer = await mlContext.createBuffer(bufferDescriptor);

    const bufferByteLength = sizeOfDescriptor(bufferDescriptor);
    let arrayBuffer = new ArrayBuffer(bufferByteLength);

    // Writing with a size that goes past that source buffer length.
    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(arrayBuffer), /*srcOffset=*/ 0,
            /*srcSize=*/ bufferByteLength + 1));
    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(arrayBuffer), /*srcOffset=*/ 3,
            /*srcSize=*/ bufferByteLength));

    // Writing with a source offset that is out of range of the source size.
    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(arrayBuffer),
            /*srcOffset=*/ bufferByteLength + 1));

    // Writing with a source offset that is out of range of implicit copy size.
    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(arrayBuffer),
            /*srcOffset=*/ bufferByteLength + 1, /*srcSize=*/ undefined));

    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(arrayBuffer), /*srcOffset=*/ undefined,
            /*srcSize=*/ bufferByteLength + 1));

    assert_throws_js(
        TypeError,
        () => mlContext.writeBuffer(
            mlBuffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE, 0xEE])));
  }, `${testName} / error`);

  promise_test(async () => {
    const bufferDescriptor = {
      dataType: 'int32',
      dimensions: [2, 2],
      usage: MLTensorUsage.WRITE_TO,
    };
    let mlBuffer = await mlContext.createBuffer(bufferDescriptor);

    // Writing data to a destroyed MLTensor should throw.
    mlBuffer.destroy();

    assert_throws_dom(
        'InvalidStateError',
        () => mlContext.writeBuffer(
            mlBuffer, new Uint8Array(sizeOfDescriptor(bufferDescriptor))));
  }, `${testName} / destroy`);

  promise_test(async () => {
    const bufferDescriptor = {
      dataType: 'int32',
      dimensions: [2, 3],
      usage: MLTensorUsage.WRITE_TO,
    };
    let mlBuffer = await mlContext.createBuffer(bufferDescriptor);

    let anotherMLContext = await navigator.ml.createContext(contextOptions);
    let anotherMLTensor = await anotherMLContext.createBuffer(bufferDescriptor);

    let inputData =
        new Uint8Array(sizeOfDescriptor(bufferDescriptor)).fill(0xAA);
    assert_throws_js(
        TypeError, () => mlContext.writeBuffer(anotherMLTensor, inputData));
    assert_throws_js(
        TypeError, () => anotherMLContext.writeBuffer(mlBuffer, inputData));
  }, `${testName} / context_mismatch`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });

    // Initialize the buffer.
    const inputData = Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]);
    mlContext.writeBuffer(mlBuffer, inputData);

    // Writing zero bytes from a zero write size.
    mlContext.writeBuffer(mlBuffer, Uint8Array.from([0xBB]), 0, 0);

    await assert_buffer_data_equals(mlContext, mlBuffer, inputData);

    // Writing zero bytes at the end of the buffer.
    mlContext.writeBuffer(
        mlBuffer, Uint32Array.from([0xBBBBBBBB]), /*srcOffset=*/ 1);

    await assert_buffer_data_equals(mlContext, mlBuffer, inputData);
  }, `${testName} / zero_write`);

  promise_test(async () => {
    const bufferDescriptor = {
      dataType: 'int32',
      dimensions: [2, 2],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    };
    let mlBuffer = await mlContext.createBuffer(bufferDescriptor);

    const bufferByteLength = sizeOfDescriptor(bufferDescriptor);
    let inputBuffer = new ArrayBuffer(bufferByteLength);

    // Initialize the buffer.
    const int32View = new Int32Array(inputBuffer);
    int32View.fill(0xBBBBBBBB);

    mlContext.writeBuffer(mlBuffer, int32View);

    // Writing to a detached buffer should be ignored.
    const detachedBuffer = inputBuffer.transfer();
    assert_true(inputBuffer.detached, 'array buffer should be detached.');

    mlContext.writeBuffer(mlBuffer, inputBuffer);

    await assert_buffer_data_equals(
        mlContext, mlBuffer, new Int32Array(detachedBuffer));
  }, `${testName} / detached`);
};

/**
 * WebNN read buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testReadWebNNBuffer = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlBuffer =
          await mlContext.createBuffer({dataType: 'int32', dimensions: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create buffer for ${variant} variant. ${e}`);
    }
  });

  promise_test(async t => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [2, 2],
      usage: MLTensorUsage.READ_FROM,
    });

    // Reading a destroyed MLTensor should reject.
    mlBuffer.destroy();

    await promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readBuffer(mlBuffer));
  }, `${testName} / read_after_destroy`);

  promise_test(async t => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [2, 3],
      usage: MLTensorUsage.READ_FROM,
    });

    let promise = mlContext.readBuffer(mlBuffer);
    let anotherPromise = mlContext.readBuffer(mlBuffer);

    mlBuffer.destroy();

    await promise_rejects_dom(t, 'InvalidStateError', promise);
    await promise_rejects_dom(t, 'InvalidStateError', anotherPromise);
  }, `${testName} / read_before_destroy`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1024],
      usage: MLTensorUsage.READ_FROM,
    });

    await assert_buffer_data_equals(
        mlContext, mlBuffer, new Uint32Array(1024));
  }, `${testName} / uninitialized`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.READ_FROM | MLTensorUsage.WRITE_TO,
    });

    // Initialize the buffer.
    mlContext.writeBuffer(mlBuffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    mlContext.writeBuffer(mlBuffer, Uint32Array.from([0xBBBBBBBB]));
    await assert_buffer_data_equals(
        mlContext, mlBuffer, Uint32Array.from([0xBBBBBBBB]));
    ;
  }, `${testName} / full_size`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });

    // Initialize the buffer.
    mlContext.writeBuffer(mlBuffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Writing to the remainder of the buffer from source offset.
    mlContext.writeBuffer(
        mlBuffer, Uint8Array.from([0xCC, 0xCC, 0xBB, 0xBB]),
        /*srcOffset=*/ 2);
    await assert_buffer_data_equals(
        mlContext, mlBuffer, Uint8Array.from([0xBB, 0xBB, 0xAA, 0xAA]));
  }, `${testName} / src_offset_only`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });

    // Initialize the buffer.
    mlContext.writeBuffer(mlBuffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Writing with both a source offset and size.
    mlContext.writeBuffer(
        mlBuffer, Uint8Array.from([0xDD, 0xDD, 0xCC, 0xDD]),
        /*srcOffset=*/ 2, /*srcSize=*/ 1);
    await assert_buffer_data_equals(
        mlContext, mlBuffer, Uint8Array.from([0xCC, 0xAA, 0xAA, 0xAA]));
  }, `${testName} / src_offset_and_size`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });

    // Initialize the buffer.
    mlContext.writeBuffer(mlBuffer, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Using an offset allows a larger source buffer to fit.
    mlContext.writeBuffer(
        mlBuffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE, 0xEE]),
        /*srcOffset=*/ 1);
    await assert_buffer_data_equals(
        mlContext, mlBuffer, Uint8Array.from([0xEE, 0xEE, 0xEE, 0xEE]));
  }, `${testName} / larger_src_data`);

  promise_test(async () => {
    let mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [1],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });

    const inputData = [0xAA, 0xAA, 0xAA, 0xAA];

    // Writing with a source offset of undefined should be treated as 0.
    mlContext.writeBuffer(
        mlBuffer, Uint8Array.from(inputData), /*srcOffset=*/ undefined,
        /*srcSize=*/ inputData.length);
    await assert_buffer_data_equals(
        mlContext, mlBuffer, Uint8Array.from(inputData));
  }, `${testName} / no_src_offset`);

  promise_test(async t => {
    const bufferDescriptor = {
      dataType: 'int32',
      dimensions: [2, 3],
      usage: MLTensorUsage.READ_FROM,
    };
    let mlBuffer = await mlContext.createBuffer(bufferDescriptor);

    let anotherMLContext = await navigator.ml.createContext(contextOptions);
    let anotherMLTensor = await anotherMLContext.createBuffer(bufferDescriptor);

    await promise_rejects_js(
        t, TypeError, mlContext.readBuffer(anotherMLTensor));
    await promise_rejects_js(
        t, TypeError, anotherMLContext.readBuffer(mlBuffer));
  }, `${testName} / context_mismatch`);
};

/**
 * WebNN dispatch buffer operation test.
 * @param {String} testName - The name of the test operation.
 */
const testDispatchWebNNBuffer = (testName) => {
  let mlContext;
  let mlGraph;
  const shape = [3, 5];
  let inputs = {};
  let outputs = {};
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }
    // Construct a simple graph: A = B + C, with two outputs.
    const builder = new MLGraphBuilder(mlContext);
    const bufferDescriptor = {
      dataType: 'float32',
      dimensions: shape,
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    };
    const lhsOperand = builder.input('lhs', bufferDescriptor);
    const rhsOperand = builder.input('rhs', bufferDescriptor);
    const output1Operand = builder.add(lhsOperand, rhsOperand);
    const output2Operand = builder.add(lhsOperand, rhsOperand);
    mlGraph = await builder.build(
        {'output1': output1Operand, 'output2': output2Operand});

    try {
      const mlBuffer =
          await mlContext.createBuffer({dataType: 'int32', dimensions: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create buffer for ${variant} variant. ${e}`);
    }

    inputs = {
      'lhs': await mlContext.createBuffer(bufferDescriptor),
      'rhs': await mlContext.createBuffer(bufferDescriptor),
    };
    outputs = {
      'output1': await mlContext.createBuffer(bufferDescriptor),
      'output2': await mlContext.createBuffer(bufferDescriptor),
    };
  });

  promise_test(async () => {
    let anotherMLContext = await navigator.ml.createContext(contextOptions);

    // Control case, same context.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Test the wrong context being used for inputs.
    const lhsBuffer = await anotherMLContext.createBuffer(
        getDescriptorFromBuffer(inputs['lhs']));
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': lhsBuffer,
              'rhs': inputs['rhs'],
            },
            outputs));

    // Test the wrong context being used for outputs.
    const outputBuffer1 = await anotherMLContext.createBuffer(
        getDescriptorFromBuffer(outputs['output1']));
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputBuffer1,
      'output2': outputs['output2'],
    }));
  }, `${testName} / context_mismatch`);

  promise_test(async () => {
    // Control case, valid buffers.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Input is a different shape.
    const lhsBuffer = await mlContext.createBuffer({
      dataType: inputs['lhs'].dataType,
      // Input rank is too high.
      dimensions: inputs['lhs'].shape.concat([2])
    });

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': lhsBuffer,
              'rhs': inputs['rhs'],
            },
            outputs));

    const rhsBuffer = await mlContext.createBuffer({
      dataType: inputs['rhs'].dataType,
      // Input rank is too low.
      dimensions: inputs['rhs'].shape.slice(1)
    });

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': rhsBuffer,
            },
            outputs));

    // Output is a different shape. Dimension value is too large.
    let output1WrongShape = [...outputs['output1'].shape];
    output1WrongShape[0] += 2;
    const outputBuffer1 = await mlContext.createBuffer(
        {dataType: outputs['output1'].dataType, dimensions: output1WrongShape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputBuffer1,
      'output2': outputs['output2'],
    }));

    // Output is a different shape. Dimension value is too small.
    let output2WrongShape = [...outputs['output2'].shape];
    output2WrongShape[1] -= 1;
    const outputBuffer2 = await mlContext.createBuffer(
        {dataType: outputs['output2'].dataType, dimensions: output2WrongShape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputBuffer2,
    }));
  }, `${testName} / invalid shape`);

  promise_test(async () => {
    // Control case, valid buffers.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Inputs are a different data type.
    const inputWrongDataType = 'int32';
    assert_not_equals(inputs['lhs'].dataType, inputWrongDataType);
    assert_not_equals(inputs['rhs'].dataType, inputWrongDataType);
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': mlContext.createBuffer({
                dataType: inputWrongDataType,
                dimensions: inputs['lhs'].shape
              }),
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': mlContext.createBuffer({
                dataType: inputWrongDataType,
                dimensions: inputs['rhs'].shape
              }),
            },
            outputs));

    // Outputs are a different data type.
    const outputWrongDataType = 'int32';
    assert_not_equals(outputs['output1'].dataType, outputWrongDataType);
    assert_not_equals(outputs['output2'].dataType, outputWrongDataType);
    const outputBuffer1 = await mlContext.createBuffer(
        {dataType: outputWrongDataType, dimensions: outputs['output1'].shape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputBuffer1,
      'output2': outputs['output2'],
    }));

    const outputBuffer2 = await mlContext.createBuffer(
        {dataType: outputWrongDataType, dimensions: outputs['output2'].shape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputBuffer2,
    }));
  }, `${testName} / invalid data type`);

  promise_test(async () => {
    // Control case, valid names.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // No names is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, {}, {}));

    // Input name is invalid.
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'aDifferentInputName': inputs['lhs'],
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'aDifferentInputName': inputs['rhs'],
            },
            outputs));

    // Output name is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'aDifferentOutputName': outputs['output1'],
      'output2': outputs['output2'],
    }));

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'aDifferentOutputName': outputs['output2'],
    }));

    // Too few named inputs is invalid.
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
            },
            outputs));

    // Too many named inputs is invalid.
    const anotherRhsBuffer =
        await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs']));
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': inputs['rhs'],
              'aDifferentInputName': anotherRhsBuffer,
            },
            outputs));

    // Too few named outputs is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1']
    }));

    // Too many named outputs is invalid.
    const anotherOutputBuffer2 = await mlContext.createBuffer(
        getDescriptorFromBuffer(outputs['output2']));
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output2'],
      'aDifferentOutputName': anotherOutputBuffer2,
    }));
  }, `${testName} / invalid_name`);

  promise_test(async () => {
    // Control case, valid buffers.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Same buffer used as outputs more than once is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output1'],
    }));

    // Same buffer used as input and output is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': inputs['lhs'],
      'output2': outputs['output2'],
    }));

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': outputs['output1'],
              'rhs': inputs['rhs'],
            },
            outputs));

    // Buffer that does not exist is invalid.
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': undefined,
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': undefined,
      'output2': outputs['output2'],
    }));
  }, `${testName} / invalid_buffer`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatch1Outputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    const dispatch2Outputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output_1 = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatch1Outputs);

    // Output_2 = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatch2Outputs);

    await assert_buffer_data_equals(
        mlContext, dispatch1Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        mlContext, dispatch1Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        mlContext, dispatch2Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        mlContext, dispatch2Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs`);

  promise_test(async () => {
    const dispatch1Inputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatch2Inputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const input1Data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatch1Inputs['lhs'], input1Data);
    mlContext.writeBuffer(dispatch1Inputs['rhs'], input1Data);

    const input2Data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeBuffer(dispatch2Inputs['lhs'], input2Data);
    mlContext.writeBuffer(dispatch2Inputs['rhs'], input2Data);

    // Output = LHS_1 + RHS_1 = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatch1Inputs, dispatchOutputs);

    // Output = LHS_2 + RHS_2 = 2 + 2 = 4
    mlContext.dispatch(mlGraph, dispatch2Inputs, dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));
  }, `${testName} / same_outputs`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs_and_outputs`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatch1Outputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    const dispatch2Outputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output_1 = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatch1Outputs);

    // Output_2 = Output_1_LHS + Output_1_RHS = 2 + 2 = 4
    mlContext.dispatch(
        mlGraph, {
          'lhs': dispatch1Outputs['output1'],
          'rhs': dispatch1Outputs['output2'],
        },
        dispatch2Outputs);

    // Output_1 = Output_2_LHS + Output_2_RHS = 4 + 4 = 8
    mlContext.dispatch(
        mlGraph, {
          'lhs': dispatch2Outputs['output1'],
          'rhs': dispatch2Outputs['output2'],
        },
        dispatch1Outputs);

    await assert_buffer_data_equals(
        mlContext, dispatch1Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(8));

    await assert_buffer_data_equals(
        mlContext, dispatch1Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(8));
  }, `${testName} / outputs_as_inputs`);

  promise_test(async () => {
    // Construct a simple graph: OUTPUT = LHS - RHS.
    const builder = new MLGraphBuilder(mlContext);
    const operandType = {dataType: 'float32', dimensions: shape};
    const lhsOperand = builder.input('lhs', operandType);
    const rhsOperand = builder.input('rhs', operandType);
    const graph =
        await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

    const lhsBuffer =
        await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs']));
    const rhsBuffer =
        await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs']));

    const dispatchOutputs = {
      'output': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1']))
    };

    // Initialize inputs
    mlContext.writeBuffer(
        lhsBuffer, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(5.0));
    mlContext.writeBuffer(
        rhsBuffer, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(3.0));

    // Output = LHS - RHS = 5 - 3 = 2
    mlContext.dispatch(
        graph, {
          'lhs': lhsBuffer,
          'rhs': rhsBuffer,
        },
        dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(2));

    // Output = RHS - LHS = 3 - 5 = -2
    mlContext.dispatch(
        graph, {
          'lhs': rhsBuffer,
          'rhs': lhsBuffer,
        },
        dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(-2));
  }, `${testName} / same name diff input buffers`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const outputBuffer1 = await mlContext.createBuffer(
        getDescriptorFromBuffer(outputs['output1']));
    const outputBuffer2 = await mlContext.createBuffer(
        getDescriptorFromBuffer(outputs['output2']));

    // Initialize inputs
    const inputData1 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData1);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData1);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, {
      'output1': outputBuffer1,
      'output2': outputBuffer2,
    });

    // Output = LHS + RHS = 2 + 2 = 4
    const inputData2 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData2);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData2);

    mlContext.dispatch(mlGraph, dispatchInputs, {
      'output1': outputBuffer1,
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    });

    // Ensure the last dispatch() did not modify the original second output
    // buffer.
    await assert_buffer_data_equals(
        mlContext, outputBuffer2, new Float32Array(sizeOfShape(shape)).fill(2));
  }, `${testName} / same name diff outputs buffers`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    // Check destroyed input buffers cannot be re-used in subsequent dispatches.
    dispatchInputs['lhs'].destroy();
    dispatchInputs['lhs'] =
        await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs']));

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));

    dispatchInputs['rhs'].destroy();
    dispatchInputs['rhs'] =
        await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs']));
    mlContext.writeBuffer(dispatchInputs['rhs'], newInputData);

    // Output = LHS + RHS = 2 + 2 = 4
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4));
  }, `${testName} / same name diff inputs buffers destroy`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['lhs'])),
      'rhs':
          await mlContext.createBuffer(getDescriptorFromBuffer(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output1'])),
      'output2': await mlContext.createBuffer(
          getDescriptorFromBuffer(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], inputData);
    mlContext.writeBuffer(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    // Check destroyed output buffers cannot be re-used in subsequent
    // dispatches.
    dispatchOutputs['output1'].destroy();
    dispatchOutputs['output1'] = await mlContext.createBuffer(
        getDescriptorFromBuffer(outputs['output1']));

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeBuffer(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_buffer_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));
  }, `${testName} / same name diff outputs buffers destroy`);
};

if (navigator.ml) {
  testCreateWebNNBuffer('create', {dataType: 'float16', dimensions: [2, 3]});
  testCreateWebNNBuffer('create', {dataType: 'float32', dimensions: [1, 5]});
  testCreateWebNNBuffer('create', {dataType: 'int32', dimensions: [4]});
  testCreateWebNNBuffer('create', {dataType: 'uint8', dimensions: [3, 2, 4]});

  testCreateWebNNBufferFails(
      'createFailsEmptyDimension', {dataType: 'int32', dimensions: [2, 0, 3]});
  testCreateWebNNBufferFails('createFailsTooLarge', {
    dataType: 'int32',
    dimensions: [kMaxUnsignedLong, kMaxUnsignedLong, kMaxUnsignedLong]
  });

  testDestroyWebNNBuffer('destroyTwice');
  testReadWebNNBuffer('read');
  testWriteWebNNBuffer('write');
  testDispatchWebNNBuffer('dispatch');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
