// META: title=test WebNN API tensor operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mltensor

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
  return descriptor.shape.reduce(
      (accumulator, currentValue) => accumulator * currentValue,
      bytesPerDataType(descriptor.dataType));
};

const getDescriptorFromTensor = (tensor) => {
  return {
    dataType: tensor.dataType,
    shape: tensor.shape,
    readable: tensor.readable,
    writable: tensor.writable,
    importableToWebGPU: tensor.importableToWebGPU,
  };
};


/**
 * WebNN destroy tensor twice test.
 * @param {String} testName - The name of the test operation.
 */
const testDestroyTensor = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlTensor =
          await mlContext.createTensor({dataType: 'int32', shape: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create tensor for ${variant} variant. ${e}`);
    }
  });
  promise_test(async () => {
    let mlTensor =
        await mlContext.createTensor({dataType: 'int32', shape: [2, 3]});
    mlTensor.destroy();
    mlTensor.destroy();
  }, `${testName}`);
};

/**
 * WebNN create tensor test.
 * @param {String} testName - The name of the test operation.
 * @param {MLTensorDescriptor} tensorDescriptor - The intended tensor specs.
 */
const testCreateTensor = (testName, tensorDescriptor) => {
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
            tensorDescriptor.dataType)) {
      await promise_rejects_js(
          t, TypeError, mlContext.createTensor(tensorDescriptor));
      return;
    }

    const mlTensor = await mlContext.createTensor(tensorDescriptor);
    assert_equals(
        mlTensor.dataType, tensorDescriptor.dataType,
        'tensor data types do not match');
    assert_array_equals(
        mlTensor.shape, tensorDescriptor.shape, 'tensor shapes do not match');
  }, `${testName} / ${tensorDescriptor.dataType}`);
};

/**
 * Same as above, but expect creating the tensor to fail.
 * @param {String} testName - The name of the test operation.
 * @param {MLTensorDescriptor} tensorDescriptor - The intended tensor specs.
 */
const testCreateTensorFails = (testName, tensorDescriptor) => {
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
        t, TypeError, mlContext.createTensor(tensorDescriptor));
  }, `${testName} / ${tensorDescriptor.dataType}`);
};

/**
 * Asserts the tensor data in MLTensor matches expected.
 * @param {MLContext} mlContext - The context used to create the tensor.
 * @param {MLTensor} mlTensor - The tensor to read and compare data.
 * @param {Array} expected - Array of the expected data in the tensor.
 */
const assert_tensor_data_equals = async (mlContext, mlTensor, expected) => {
  const actual = await mlContext.readTensor(mlTensor);
  assert_array_equals(
      new expected.constructor(actual), expected,
      'Read tensor data equals expected data.');
};

/**
 * WebNN write tensor operation test.
 * @param {String} testName - The name of the test operation.
 */
const testWriteTensor = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlTensor =
          await mlContext.createTensor({dataType: 'int32', shape: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create tensor for ${variant} variant. ${e}`);
    }
  });

  promise_test(async () => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [4],
      readable: true,
      writable: true,
    };
    const tensorByteLength = sizeOfDescriptor(tensorDescriptor);

    // Required to use SharedArrayBuffer.
    assert_true(
        self.crossOriginIsolated,
        'The page is served with COOP and COEP, it should be cross-origin-isolated.');

    let arrayBuffer = new ArrayBuffer(tensorByteLength);
    let arrayBufferView = new Int32Array(arrayBuffer);
    arrayBufferView.fill(7);

    let sharedArrayBuffer = new SharedArrayBuffer(tensorByteLength);
    let sharedArrayBufferView = new Int32Array(sharedArrayBuffer);
    sharedArrayBufferView.fill(7);

    const tensors = await Promise.all([
      mlContext.createTensor(tensorDescriptor),
      mlContext.createTensor(tensorDescriptor),
      mlContext.createTensor(tensorDescriptor),
      mlContext.createTensor(tensorDescriptor)
    ]);

    mlContext.writeTensor(tensors[0], arrayBuffer);
    mlContext.writeTensor(tensors[2], arrayBufferView);
    mlContext.writeTensor(tensors[1], sharedArrayBuffer);
    mlContext.writeTensor(tensors[3], sharedArrayBufferView);

    await Promise.all(tensors.map(async (tensor) => {
      assert_tensor_data_equals(mlContext, tensor, arrayBufferView);
    }));
  }, `${testName} / write with different kinds of buffers`);

  promise_test(async () => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [1],
      writable: true,
    };
    let mlTensor = await mlContext.createTensor(tensorDescriptor);

    const tensorByteLength = sizeOfDescriptor(tensorDescriptor);

    // Writing with a buffer larger than the source tensor.
    assert_throws_js(
        TypeError,
        () => mlContext.writeTensor(
            mlTensor, new ArrayBuffer(tensorByteLength + 1)));
    // Writing with a buffer smaller than the source tensor.
    assert_throws_js(
        TypeError,
        () => mlContext.writeTensor(
            mlTensor, new ArrayBuffer(tensorByteLength - 1)));
  }, `${testName} / write with buffer of wrong size`);

  promise_test(async () => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [2, 2],
      writable: true,
    };
    let mlTensor = await mlContext.createTensor(tensorDescriptor);

    // Writing data to a destroyed MLTensor should throw.
    mlTensor.destroy();

    assert_throws_dom(
        'InvalidStateError',
        () => mlContext.writeTensor(
            mlTensor, new Uint8Array(sizeOfDescriptor(tensorDescriptor))));
  }, `${testName} / destroy`);

  promise_test(async () => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [2, 3],
      writable: true,
    };
    let mlTensor = await mlContext.createTensor(tensorDescriptor);

    let anotherMLContext = await navigator.ml.createContext(contextOptions);
    let anotherMLTensor = await anotherMLContext.createTensor(tensorDescriptor);

    let inputData =
        new Uint8Array(sizeOfDescriptor(tensorDescriptor)).fill(0xAA);
    assert_throws_js(
        TypeError, () => mlContext.writeTensor(anotherMLTensor, inputData));
    assert_throws_js(
        TypeError, () => anotherMLContext.writeTensor(mlTensor, inputData));
  }, `${testName} / context_mismatch`);

  promise_test(async () => {
    let mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [],
      readable: true,
      writable: true,
    });

    const inputData = Int32Array.from([0xAAAABBBB]);
    mlContext.writeTensor(mlTensor, inputData);
    await assert_tensor_data_equals(mlContext, mlTensor, inputData);
  }, `${testName} / scalar`);

  promise_test(async () => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [2, 2],
      readable: true,
      writable: true,
    };
    let mlTensor = await mlContext.createTensor(tensorDescriptor);

    const tensorByteLength = sizeOfDescriptor(tensorDescriptor);
    let inputBuffer = new ArrayBuffer(tensorByteLength);

    const int32View = new Int32Array(inputBuffer);
    int32View.fill(0xBBBBBBBB);

    mlContext.writeTensor(mlTensor, int32View);

    // Writing to a detached buffer should fail.
    const detachedBuffer = inputBuffer.transfer();
    assert_true(inputBuffer.detached, 'array buffer should be detached.');

    assert_throws_js(
        TypeError, () => mlContext.writeTensor(mlTensor, inputBuffer));

    await assert_tensor_data_equals(
        mlContext, mlTensor, new Int32Array(detachedBuffer));
  }, `${testName} / detached`);
};

/**
 * WebNN read tensor operation test.
 * @param {String} testName - The name of the test operation.
 */
const testReadTensor = (testName) => {
  let mlContext;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    try {
      const mlTensor =
          await mlContext.createTensor({dataType: 'int32', shape: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create tensor for ${variant} variant. ${e}`);
    }
  });

  promise_test(async t => {
    let mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [2, 2],
      readable: true,
    });

    // Reading a destroyed MLTensor should reject.
    mlTensor.destroy();

    await promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readTensor(mlTensor));
  }, `${testName} / read_after_destroy`);

  promise_test(async t => {
    let mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [2, 3],
      readable: true,
    });

    let promise = mlContext.readTensor(mlTensor);
    let anotherPromise = mlContext.readTensor(mlTensor);

    mlTensor.destroy();

    await promise_rejects_dom(t, 'InvalidStateError', promise);
    await promise_rejects_dom(t, 'InvalidStateError', anotherPromise);
  }, `${testName} / read_before_destroy`);

  promise_test(async () => {
    let mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [1024],
      readable: true,
    });

    await assert_tensor_data_equals(mlContext, mlTensor, new Uint32Array(1024));
  }, `${testName} / uninitialized`);

  promise_test(async () => {
    let mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [1],
      readable: true,
      writable: true,
    });

    mlContext.writeTensor(mlTensor, Uint8Array.from([0xAA, 0xAA, 0xAA, 0xAA]));

    // Write over previously-written data.
    mlContext.writeTensor(mlTensor, Uint32Array.from([0xBBBBBBBB]));
    await assert_tensor_data_equals(
        mlContext, mlTensor, Uint32Array.from([0xBBBBBBBB]));
    ;
  }, `${testName} / overwrite`);

  promise_test(async t => {
    const tensorDescriptor = {
      dataType: 'int32',
      shape: [2, 3],
      readable: true,
    };
    let mlTensor = await mlContext.createTensor(tensorDescriptor);

    let anotherMLContext = await navigator.ml.createContext(contextOptions);
    let anotherMLTensor = await anotherMLContext.createTensor(tensorDescriptor);

    await promise_rejects_js(
        t, TypeError, mlContext.readTensor(anotherMLTensor));
    await promise_rejects_js(
        t, TypeError, anotherMLContext.readTensor(mlTensor));
  }, `${testName} / context_mismatch`);
};

/**
 * WebNN dispatch tensor operation test.
 * @param {String} testName - The name of the test operation.
 */
const testDispatchTensor = (testName) => {
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
    const tensorDescriptor = {
      dataType: 'float32',
      shape: shape,
      readable: true,
      writable: true,
    };
    const lhsOperand = builder.input('lhs', tensorDescriptor);
    const rhsOperand = builder.input('rhs', tensorDescriptor);
    const output1Operand = builder.add(lhsOperand, rhsOperand);
    const output2Operand = builder.add(lhsOperand, rhsOperand);
    mlGraph = await builder.build(
        {'output1': output1Operand, 'output2': output2Operand});

    try {
      const mlTensor =
          await mlContext.createTensor({dataType: 'int32', shape: [2, 3]});
    } catch (e) {
      throw new AssertionError(
          `Unable to create tensor for ${variant} variant. ${e}`);
    }

    inputs = {
      'lhs': await mlContext.createTensor(tensorDescriptor),
      'rhs': await mlContext.createTensor(tensorDescriptor),
    };
    outputs = {
      'output1': await mlContext.createTensor(tensorDescriptor),
      'output2': await mlContext.createTensor(tensorDescriptor),
    };
  });

  promise_test(async () => {
    let anotherMLContext = await navigator.ml.createContext(contextOptions);

    // Control case, same context.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Test the wrong context being used for inputs.
    const lhsTensor = await anotherMLContext.createTensor(
        getDescriptorFromTensor(inputs['lhs']));
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': lhsTensor,
              'rhs': inputs['rhs'],
            },
            outputs));

    // Test the wrong context being used for outputs.
    const outputTensor1 = await anotherMLContext.createTensor(
        getDescriptorFromTensor(outputs['output1']));
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputTensor1,
      'output2': outputs['output2'],
    }));
  }, `${testName} / context_mismatch`);

  promise_test(async () => {
    // Control case, valid tensors.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Input is a different shape.
    const lhsTensor = await mlContext.createTensor({
      dataType: inputs['lhs'].dataType,
      // Input rank is too high.
      shape: inputs['lhs'].shape.concat([2])
    });

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': lhsTensor,
              'rhs': inputs['rhs'],
            },
            outputs));

    const rhsTensor = await mlContext.createTensor({
      dataType: inputs['rhs'].dataType,
      // Input rank is too low.
      shape: inputs['rhs'].shape.slice(1)
    });

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': rhsTensor,
            },
            outputs));

    // Output is a different shape. Dimension value is too large.
    let output1WrongShape = [...outputs['output1'].shape];
    output1WrongShape[0] += 2;
    const outputTensor1 = await mlContext.createTensor(
        {dataType: outputs['output1'].dataType, shape: output1WrongShape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputTensor1,
      'output2': outputs['output2'],
    }));

    // Output is a different shape. Dimension value is too small.
    let output2WrongShape = [...outputs['output2'].shape];
    output2WrongShape[1] -= 1;
    const outputTensor2 = await mlContext.createTensor(
        {dataType: outputs['output2'].dataType, shape: output2WrongShape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputTensor2,
    }));
  }, `${testName} / invalid shape`);

  promise_test(async () => {
    // Control case, valid tensors.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Inputs are a different data type.
    const inputWrongDataType = 'int32';
    assert_not_equals(inputs['lhs'].dataType, inputWrongDataType);
    assert_not_equals(inputs['rhs'].dataType, inputWrongDataType);
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': mlContext.createTensor(
                  {dataType: inputWrongDataType, shape: inputs['lhs'].shape}),
              'rhs': inputs['rhs'],
            },
            outputs));

    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': mlContext.createTensor(
                  {dataType: inputWrongDataType, shape: inputs['rhs'].shape}),
            },
            outputs));

    // Outputs are a different data type.
    const outputWrongDataType = 'int32';
    assert_not_equals(outputs['output1'].dataType, outputWrongDataType);
    assert_not_equals(outputs['output2'].dataType, outputWrongDataType);
    const outputTensor1 = await mlContext.createTensor(
        {dataType: outputWrongDataType, shape: outputs['output1'].shape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputTensor1,
      'output2': outputs['output2'],
    }));

    const outputTensor2 = await mlContext.createTensor(
        {dataType: outputWrongDataType, shape: outputs['output2'].shape});

    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputTensor2,
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
    const anotherRhsTensor =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs']));
    assert_throws_js(
        TypeError,
        () => mlContext.dispatch(
            mlGraph, {
              'lhs': inputs['lhs'],
              'rhs': inputs['rhs'],
              'aDifferentInputName': anotherRhsTensor,
            },
            outputs));

    // Too few named outputs is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1']
    }));

    // Too many named outputs is invalid.
    const anotherOutputTensor2 = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output2']));
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output2'],
      'aDifferentOutputName': anotherOutputTensor2,
    }));
  }, `${testName} / invalid_name`);

  promise_test(async () => {
    // Control case, valid tensors.
    mlContext.dispatch(mlGraph, inputs, outputs);

    // Same tensor used as outputs more than once is invalid.
    assert_throws_js(TypeError, () => mlContext.dispatch(mlGraph, inputs, {
      'output1': outputs['output1'],
      'output2': outputs['output1'],
    }));

    // Same tensor used as input and output is invalid.
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

    // Tensor that does not exist is invalid.
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
  }, `${testName} / invalid_tensor`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatch1Outputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    const dispatch2Outputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData);

    // Output_1 = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatch1Outputs);

    // Output_2 = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatch2Outputs);

    await assert_tensor_data_equals(
        mlContext, dispatch1Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_tensor_data_equals(
        mlContext, dispatch1Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_tensor_data_equals(
        mlContext, dispatch2Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_tensor_data_equals(
        mlContext, dispatch2Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs`);

  promise_test(async () => {
    const dispatch1Inputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatch2Inputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const input1Data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatch1Inputs['lhs'], input1Data);
    mlContext.writeTensor(dispatch1Inputs['rhs'], input1Data);

    const input2Data =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeTensor(dispatch2Inputs['lhs'], input2Data);
    mlContext.writeTensor(dispatch2Inputs['rhs'], input2Data);

    // Output = LHS_1 + RHS_1 = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatch1Inputs, dispatchOutputs);

    // Output = LHS_2 + RHS_2 = 2 + 2 = 4
    mlContext.dispatch(mlGraph, dispatch2Inputs, dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(4.0));
  }, `${testName} / same_outputs`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(2.0));
  }, `${testName} / same_inputs_and_outputs`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatch1Outputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    const dispatch2Outputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData);

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

    await assert_tensor_data_equals(
        mlContext, dispatch1Outputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(8));

    await assert_tensor_data_equals(
        mlContext, dispatch1Outputs['output2'],
        new Float32Array(sizeOfShape(shape)).fill(8));
  }, `${testName} / outputs_as_inputs`);

  promise_test(async () => {
    // Construct a simple graph: OUTPUT = LHS - RHS.
    const builder = new MLGraphBuilder(mlContext);
    const operandType = {dataType: 'float32', shape};
    const lhsOperand = builder.input('lhs', operandType);
    const rhsOperand = builder.input('rhs', operandType);
    const graph =
        await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

    const lhsTensor =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs']));
    const rhsTensor =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs']));

    const dispatchOutputs = {
      'output': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1']))
    };

    // Initialize inputs
    mlContext.writeTensor(
        lhsTensor, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(5.0));
    mlContext.writeTensor(
        rhsTensor, new TypedArrayDict['float32'](sizeOfShape(shape)).fill(3.0));

    // Output = LHS - RHS = 5 - 3 = 2
    mlContext.dispatch(
        graph, {
          'lhs': lhsTensor,
          'rhs': rhsTensor,
        },
        dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(2));

    // Output = RHS - LHS = 3 - 5 = -2
    mlContext.dispatch(
        graph, {
          'lhs': rhsTensor,
          'rhs': lhsTensor,
        },
        dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output'],
        new Float32Array(sizeOfShape(shape)).fill(-2));
  }, `${testName} / same name diff input tensors`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const outputTensor1 = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output1']));
    const outputTensor2 = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output2']));

    // Initialize inputs
    const inputData1 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData1);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData1);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, {
      'output1': outputTensor1,
      'output2': outputTensor2,
    });

    // Output = LHS + RHS = 2 + 2 = 4
    const inputData2 =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData2);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData2);

    mlContext.dispatch(mlGraph, dispatchInputs, {
      'output1': outputTensor1,
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    });

    // Ensure the last dispatch() did not modify the original second output
    // tensor.
    await assert_tensor_data_equals(
        mlContext, outputTensor2, new Float32Array(sizeOfShape(shape)).fill(2));
  }, `${testName} / same name diff outputs tensors`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    // Check destroyed input tensors cannot be re-used in subsequent dispatches.
    dispatchInputs['lhs'].destroy();
    dispatchInputs['lhs'] =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs']));

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeTensor(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));

    dispatchInputs['rhs'].destroy();
    dispatchInputs['rhs'] =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs']));
    mlContext.writeTensor(dispatchInputs['rhs'], newInputData);

    // Output = LHS + RHS = 2 + 2 = 4
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(4));
  }, `${testName} / same name diff inputs tensors destroy`);

  promise_test(async () => {
    const dispatchInputs = {
      'lhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs'])),
      'rhs':
          await mlContext.createTensor(getDescriptorFromTensor(inputs['rhs'])),
    };

    const dispatchOutputs = {
      'output1': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output1'])),
      'output2': await mlContext.createTensor(
          getDescriptorFromTensor(outputs['output2'])),
    };

    // Initialize inputs
    const inputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(dispatchInputs['lhs'], inputData);
    mlContext.writeTensor(dispatchInputs['rhs'], inputData);

    // Output = LHS + RHS = 1 + 1 = 2
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    // Check destroyed output tensors cannot be re-used in subsequent
    // dispatches.
    dispatchOutputs['output1'].destroy();
    dispatchOutputs['output1'] = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output1']));

    const newInputData =
        new TypedArrayDict['float32'](sizeOfShape(shape)).fill(2.0);
    mlContext.writeTensor(dispatchInputs['lhs'], newInputData);

    // Output = LHS + RHS = 2 + 1 = 3
    mlContext.dispatch(mlGraph, dispatchInputs, dispatchOutputs);

    await assert_tensor_data_equals(
        mlContext, dispatchOutputs['output1'],
        new Float32Array(sizeOfShape(shape)).fill(3));
  }, `${testName} / same name diff outputs tensors destroy`);
};

if (navigator.ml) {
  testCreateTensor('create', {dataType: 'float16', shape: [2, 3]});
  testCreateTensor('create', {dataType: 'float32', shape: [1, 5]});
  testCreateTensor('create', {dataType: 'int32', shape: [4]});
  testCreateTensor('create', {dataType: 'uint8', shape: [3, 2, 4]});

  testCreateTensorFails(
      'createFailsEmptyDimension', {dataType: 'int32', shape: [2, 0, 3]});
  testCreateTensorFails('createFailsTooLarge', {
    dataType: 'int32',
    shape: [kMaxUnsignedLong, kMaxUnsignedLong, kMaxUnsignedLong]
  });

  testDestroyTensor('destroyTwice');
  testReadTensor('read');
  testWriteTensor('write');
  testDispatchTensor('dispatch');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
