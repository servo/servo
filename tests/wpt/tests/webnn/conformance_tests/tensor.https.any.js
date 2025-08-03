// META: title=test WebNN API tensor operations
// META: global=window,worker
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
    exportableToGPU: tensor.exportableToGPU,
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
 * WebNN create constant tensor test.
 * @param {String} testName - The name of the test operation.
 * @param {MLOperandDescriptor} descriptor - The intended operand specs.
 */
const testCreateConstantTensor = (testName, descriptor) => {
  let mlContext;
  let isConstantTensorSupported = false;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (error) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${error}`);
    }

    // Check if WebNN has constant tensor support.
    try {
      await mlContext.createConstantTensor(
          {
            dataType: 'float32',
            shape: [1],
          },
          new Float32Array([0xAA]));
      isConstantTensorSupported = true;
    } catch (error) {
      if (error.name !== 'NotSupportedError') {
        throw error;
      }
    }
  });

  promise_test(async t => {
    if (!isConstantTensorSupported) {
      return;
    }

    const inputData =
        new TypedArrayDict[descriptor.dataType](sizeOfShape(descriptor.shape))
            .fill(0xAA);
    if (!mlContext.opSupportLimits().constant.dataTypes.includes(
            descriptor.dataType)) {
      await promise_rejects_js(
          t, TypeError, mlContext.createConstantTensor(descriptor, inputData));
      return;
    }

    const mlTensor =
        await mlContext.createConstantTensor(descriptor, inputData);
    assert_true(mlTensor.constant, 'constant tensors should be constant.');
    assert_false(mlTensor.readable, 'constant tensors should not be readable.');
    assert_false(mlTensor.writable, 'constant tensors should not be writable.');
  }, `${testName} / ${descriptor.dataType}`);

  promise_test(async t => {
    if (!isConstantTensorSupported) {
      return;
    }

    try {
      const inputDataTooBig = new TypedArrayDict[descriptor.dataType](
          sizeOfShape(descriptor.shape) + 1);
      await promise_rejects_js(
          t, TypeError,
          mlContext.createConstantTensor(descriptor, inputDataTooBig));
    } catch (error) {
      if (error instanceof RangeError) {
        return;  // Skip test when dataType is too big.
      } else {
        throw error;
      }
    }
  }, `${testName} / ${descriptor.dataType} / source data too big`);

  promise_test(async t => {
    if (!isConstantTensorSupported) {
      return;
    }

    try {
      const inputDataTooSmall = new TypedArrayDict[descriptor.dataType](
          sizeOfShape(descriptor.shape) - 1);
      await promise_rejects_js(
          t, TypeError,
          mlContext.createConstantTensor(descriptor, inputDataTooSmall));
    } catch (error) {
      if (error instanceof RangeError) {
        return;  // Skip test when dataType is too big.
      } else {
        throw error;
      }
    }
  }, `${testName} / ${descriptor.dataType} / source data too small`);
};

/**
 * Same as above, but expect constant tensor creation to fail.
 * @param {String} testName - The name of the test operation.
 * @param {MLOperandDescriptor} descriptor - The intended operand specs.
 */
const testCreateConstantTensorFails = (testName, descriptor) => {
  let mlContext;

  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (error) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${error}`);
    }
  });

  promise_test(async t => {
    await promise_rejects_js(
        t, TypeError,
        mlContext.createConstantTensor(
            descriptor,
            new TypedArrayDict[descriptor.dataType](
                sizeOfShape(descriptor.shape))));
  }, `${testName} / ${descriptor.dataType}`);
};

promise_test(async t => {
  const tensorDescriptor = {
    dataType: 'int32',
    shape: [(context.opSupportLimits().maxTensorByteLength + 1) / 4],
    writable: true,
  };
  await promise_rejects_js(
    t, TypeError, context.createTensor(tensorDescriptor));
}, `create too large tensor byte length that exceeds limit`);

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

  if ('SharedArrayBuffer' in globalThis) {
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
  }

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
  let isConstantTensorSupported = false;
  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    // Check if WebNN has constant tensor support.
    try {
      await mlContext.createConstantTensor(
          {
            dataType: 'float32',
            shape: [1],
          },
          new Float32Array([0xAA]));
      isConstantTensorSupported = true;
    } catch (error) {
      if (error.name !== 'NotSupportedError') {
        throw error;
      }
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

  promise_test(async () => {
    if (!isConstantTensorSupported) {
      return;
    }

    let constantTensor = await mlContext.createConstantTensor(
        {
          dataType: 'float32',
          shape: shape,
        },
        new Float32Array(sizeOfShape(shape)).fill(3.0));

    const builder = new MLGraphBuilder(mlContext);
    const lhsConstantOperand = builder.constant(constantTensor);
    const rhsConstantOperand = builder.constant(constantTensor);
    const outputOperand = builder.add(lhsConstantOperand, rhsConstantOperand);
    const graphWithOnlyConstants =
        await builder.build({'output': outputOperand});

    const outputTensor = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output1']));

    // Output = LHS + RHS = 3 + 3 = 6
    mlContext.dispatch(graphWithOnlyConstants, {}, {'output': outputTensor});

    await assert_tensor_data_equals(
        mlContext, outputTensor,
        new Float32Array(sizeOfShape(shape)).fill(6.0));
  }, `${testName} / same constant same graph`);

  promise_test(async () => {
    if (!isConstantTensorSupported) {
      return;
    }

    const rhsConstantTensor = await mlContext.createConstantTensor(
        {
          dataType: 'float32',
          shape: shape,
        },
        new Float32Array(sizeOfShape(shape)).fill(3.0));

    const lhsInputOperandDesc = {dataType: 'float32', shape};

    let graphWithConstants;
    {
      const builder = new MLGraphBuilder(mlContext);
      const lhsOperand = builder.input('lhs', lhsInputOperandDesc);
      const rhsConstantOperand = builder.constant(rhsConstantTensor);
      const outputOperand = builder.sub(lhsOperand, rhsConstantOperand);
      graphWithConstants = await builder.build({'output': outputOperand});
    }

    const lhsTensor =
        await mlContext.createTensor(getDescriptorFromTensor(inputs['lhs']));
    mlContext.writeTensor(
        lhsTensor, new Float32Array(sizeOfShape(shape)).fill(5.0));

    const outputTensor = await mlContext.createTensor(
        getDescriptorFromTensor(outputs['output1']));

    // Output = LHS - RHS = 5 - 3 = 2
    mlContext.dispatch(
        graphWithConstants, {
          'lhs': lhsTensor,
        },
        {'output': outputTensor});

    // Create another graph reusing the same constants.
    {
      const builder = new MLGraphBuilder(mlContext);
      const lhsOperand = builder.input('lhs', lhsInputOperandDesc);
      const rhsConstantOperand = builder.constant(rhsConstantTensor);
      const outputOperand = builder.sub(lhsOperand, rhsConstantOperand);
      graphWithConstants = await builder.build({'output': outputOperand});
    }

    mlContext.writeTensor(
        lhsTensor, new Float32Array(sizeOfShape(shape)).fill(4.0));

    // Output = LHS - RHS = 4 - 3 = 1
    mlContext.dispatch(
        graphWithConstants, {
          'lhs': lhsTensor,
        },
        {'output': outputTensor});

    await assert_tensor_data_equals(
        mlContext, outputTensor,
        new Float32Array(sizeOfShape(shape)).fill(1.0));
  }, `${testName} / same constant multiple graphs`);
};

/**
 * Asserts a gpu buffer data matches expected.
 * @param {GPUDevice} gpuDevice - The device used to create the context.
 * @param {GPUBuffer} gpuBuffer - The buffer to read and compare data.
 * @param {Array} expected - Array of the expected data in the tensor.
 */
const assert_gpu_buffer_data_equals =
    async (gpuDevice, gpuBuffer, expected) => {
  const gpuReadbackBuffer = gpuDevice.createBuffer({
    size: expected.byteLength,
    usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
  });

  const gpuCommandEncoder = gpuDevice.createCommandEncoder();
  gpuCommandEncoder.copyBufferToBuffer(
      gpuBuffer, 0, gpuReadbackBuffer, 0, expected.byteLength);
  gpuDevice.queue.submit([gpuCommandEncoder.finish()]);

  await gpuReadbackBuffer.mapAsync(GPUMapMode.READ);
  const outputData =
      new expected.constructor(gpuReadbackBuffer.getMappedRange());
  assert_array_equals(outputData, expected);
  gpuReadbackBuffer.unmap();
};

/**
 * Export to GPU operation test.
 * @param {String} testName - The name of the test operation.
 */
const testExportToGPU = (testName) => {
  let gpuAdapter;
  let gpuDevice;
  let mlContext;
  let mlGraph;
  const shape = [2, 2];
  let gpuComputePipeline;
  let isExportToGPUSupported = true;
  promise_setup(async () => {
    // Initialize GPU
    gpuAdapter = navigator.gpu && await navigator.gpu.requestAdapter();
    if (!gpuAdapter) {
      isExportToGPUSupported = false;
      return;
    }

    gpuDevice = await gpuAdapter.requestDevice();
    if (!gpuDevice) {
      isExportToGPUSupported = false;
      return;
    }

    // Construct a GPU custom op which increments each number of the input
    // buffer by 1.
    const gpuComputeShaderCode = `
        @group(0) @binding(0) var<storage, read> inputBuffer: array<f32>;
        @group(0) @binding(1) var<storage, read_write> outputBuffer: array<f32>;

        @compute @workgroup_size(1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            outputBuffer[index] = inputBuffer[index] + 1.0;
        }`;

    const gpuShaderModule =
        gpuDevice.createShaderModule({code: gpuComputeShaderCode});
    gpuComputePipeline = gpuDevice.createComputePipeline({
      layout: 'auto',
      compute: {module: gpuShaderModule, entryPoint: 'main'},
    });

    // Initialize WebNN
    try {
      mlContext = await navigator.ml.createContext(gpuDevice);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }

    // Check if WebNN interop is supported.
    try {
      await mlContext.createTensor({
        dataType: 'float32',
        shape: shape,
        exportableToGPU: true,
      });
    } catch (e) {
      if (e.name === 'NotSupportedError') {
        isExportToGPUSupported = false;
        return;
      }
      throw e;
    }

    // Construct a simple graph: OUTPUT = LHS + RHS.
    const mlBuilder = new MLGraphBuilder(mlContext);
    const mlOperandDescriptor = {dataType: 'float32', shape};
    const lhsOperand = mlBuilder.input('lhs', mlOperandDescriptor);
    const rhsOperand = mlBuilder.input('rhs', mlOperandDescriptor);
    mlGraph = await mlBuilder.build(
        {'output': mlBuilder.add(lhsOperand, rhsOperand)});
  });

  const dispatchGPU =
      (gpuDevice, gpuPipeline, gpuInputBuffer, gpuOutputBuffer, inputData) => {
        const gpuBindGroup = gpuDevice.createBindGroup({
          layout: gpuPipeline.getBindGroupLayout(0),
          entries: [
            {binding: 0, resource: {buffer: gpuInputBuffer}},
            {binding: 1, resource: {buffer: gpuOutputBuffer}},
          ],
        });

        const gpuCommandEncoder = gpuDevice.createCommandEncoder();
        {
          const gpuComputePass = gpuCommandEncoder.beginComputePass();
          gpuComputePass.setPipeline(gpuPipeline);
          gpuComputePass.setBindGroup(0, gpuBindGroup);
          gpuComputePass.dispatchWorkgroups(
              inputData.byteLength / inputData.BYTES_PER_ELEMENT);
          gpuComputePass.end();
        }
        gpuDevice.queue.submit([gpuCommandEncoder.finish()]);
      };

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensorDescriptor = {
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true
    };

    const mlTensor = await mlContext.createTensor(mlTensorDescriptor);
    const gpuTensorBuffer = await mlContext.exportToGPU(mlTensor);

    assert_equals(
        gpuTensorBuffer.usage,
        GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC |
            GPUBufferUsage.COPY_DST);
    assert_equals(gpuTensorBuffer.size, sizeOfDescriptor(mlTensorDescriptor));
  }, `${testName} / export tensor`);

  promise_test(async t => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensor = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: false,
    });

    await promise_rejects_js(t, TypeError, mlContext.exportToGPU(mlTensor));
  }, `${testName} / export wrong tensor`);

  promise_test(async t => {
    if (!isExportToGPUSupported) {
      return;
    }

    const maxBufferSizeOOB = gpuDevice.limits.maxBufferSize + 1;
    const elementSize = Float32Array.BYTES_PER_ELEMENT;
    const shape = [maxBufferSizeOOB / elementSize];

    const mlTensor = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
    });

    await mlContext.exportToGPU(mlTensor);
  }, `${testName} / export big tensor`)

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensorDescriptor = {
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      readable: true,
      writable: true
    };

    let mlTensor = await mlContext.createTensor(mlTensorDescriptor);
    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(mlTensor, inputData);

    const gpuTensorBuffer = await mlContext.exportToGPU(mlTensor);
    gpuTensorBuffer.destroy();

    await assert_tensor_data_equals(mlContext, mlTensor, inputData);
  }, `${testName} / export then destroy buffer`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensorDescriptor = {
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      writable: true
    };

    let mlTensor = await mlContext.createTensor(mlTensorDescriptor);

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(mlTensor, inputData);

    const gpuTensorBuffer = await mlContext.exportToGPU(mlTensor);
    mlTensor.destroy();

    await assert_gpu_buffer_data_equals(gpuDevice, gpuTensorBuffer, inputData);
  }, `${testName} / export then destroy tensor`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensor = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
    });
    await mlContext.exportToGPU(mlTensor);
    assert_throws_js(
        TypeError,
        () => mlContext.writeTensor(
            mlTensor, new Float32Array([1.0, 2.0, 3.0, 4.0])));
  }, `${testName} / write tensor after export`);

  promise_test(async t => {
    if (!isExportToGPUSupported) {
      return;
    }

    const mlTensor = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
    });

    // Second call rejects because the first export is still pending and multiple
    // exports aren’t allowed.
    let export_promise = mlContext.exportToGPU(mlTensor);
    await promise_rejects_js(t, TypeError, mlContext.exportToGPU(mlTensor));

    let gpuTensorBuffer1 = await export_promise;
    let gpuTensorBuffer2 = await mlContext.exportToGPU(mlTensor);
    assert_equals(
        gpuTensorBuffer1, gpuTensorBuffer2, 'Same buffers should be returned.');
  }, `${testName} / export twice`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    // Initialize the tensor buffers from WebNN.
    let mlTensorInput = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      writable: true
    });

    const inputData1 = new Float32Array(sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(mlTensorInput, inputData1);

    let mlTensorOutput = await mlContext.createTensor(
        {dataType: 'float32', shape: shape, exportableToGPU: true});

    let gpuTensorBufferInput = await mlContext.exportToGPU(mlTensorInput);
    let gpuTensorBufferOutput = await mlContext.exportToGPU(mlTensorOutput);

    dispatchGPU(
        gpuDevice, gpuComputePipeline, gpuTensorBufferInput,
        gpuTensorBufferOutput, inputData1);

    gpuTensorBufferInput.destroy();
    gpuTensorBufferOutput.destroy();

    // Write different data to the input tensor.
    const inputData2 = new Float32Array(sizeOfShape(shape)).fill(2.0);
    mlContext.writeTensor(mlTensorInput, inputData2);

    gpuTensorBufferInput = await mlContext.exportToGPU(mlTensorInput);
    gpuTensorBufferOutput = await mlContext.exportToGPU(mlTensorOutput);

    dispatchGPU(
        gpuDevice, gpuComputePipeline, gpuTensorBufferInput,
        gpuTensorBufferOutput, inputData2);

    await assert_gpu_buffer_data_equals(
        gpuDevice, gpuTensorBufferOutput, inputData2.map(x => x + 1));
  }, `${testName} / dispatch gpu twice`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    // Initialize the tensor buffers from WebNN.
    let mlTensorInput = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      writable: true
    });

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(mlTensorInput, inputData);

    let mlTensorOutput = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      readable: true
    });

    let gpuTensorBufferInput = await mlContext.exportToGPU(mlTensorInput);
    let gpuTensorBufferOutput = await mlContext.exportToGPU(mlTensorOutput);

    gpuTensorBufferInput.destroy();
    gpuTensorBufferOutput.destroy();

    mlContext.dispatch(
        mlGraph, {
          'lhs': mlTensorInput,
          'rhs': mlTensorInput,
        },
        {
          'output': mlTensorOutput,
        });

    await assert_tensor_data_equals(
        mlContext, mlTensorOutput, inputData.map(x => x + 1));
  }, `${testName} / webnn dispatch only`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    // Initialize the tensor buffers from WebNN.
    let mlTensorInput = await mlContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      writable: true
    });

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    mlContext.writeTensor(mlTensorInput, inputData);

    let mlTensorOutput = await mlContext.createTensor(
        {dataType: 'float32', shape: shape, exportableToGPU: true});

    let gpuTensorBufferInput = await mlContext.exportToGPU(mlTensorInput);
    let gpuTensorBufferOutput = await mlContext.exportToGPU(mlTensorOutput);

    dispatchGPU(
        gpuDevice, gpuComputePipeline, gpuTensorBufferInput,
        gpuTensorBufferOutput, inputData);

    gpuTensorBufferInput.destroy();
    gpuTensorBufferOutput.destroy();

    mlContext.dispatch(
        mlGraph, {
          'lhs': mlTensorOutput,
          'rhs': mlTensorOutput,
        },
        {
          'output': mlTensorInput,
        });

    gpuTensorBufferInput = await mlContext.exportToGPU(mlTensorInput);
    gpuTensorBufferOutput = await mlContext.exportToGPU(mlTensorOutput);

    dispatchGPU(
        gpuDevice, gpuComputePipeline, gpuTensorBufferInput,
        gpuTensorBufferOutput, inputData);

    await assert_gpu_buffer_data_equals(
        gpuDevice, gpuTensorBufferOutput,
        new Float32Array(sizeOfShape(shape)).fill(5.0));
  }, `${testName} / dispatch from webgpu then webnn`);

  promise_test(async () => {
    if (!isExportToGPUSupported) {
      return;
    }

    let anotherMLContext = await navigator.ml.createContext(gpuDevice);

    let mlTensor = await anotherMLContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      writable: true
    });

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    anotherMLContext.writeTensor(mlTensor, inputData);

    const gpuTensorBuffer = await anotherMLContext.exportToGPU(mlTensor);

    anotherMLContext.destroy();

    await assert_gpu_buffer_data_equals(gpuDevice, gpuTensorBuffer, inputData);
  }, `${testName} / destroy context after export`);

  promise_test(async t => {
    if (!isExportToGPUSupported) {
      return;
    }

    let anotherGPUDevice = await gpuAdapter.requestDevice();
    let anotherMLContext = await navigator.ml.createContext(anotherGPUDevice);

    let mlTensor = await anotherMLContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      readable: true,
      writable: true
    });

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    anotherMLContext.writeTensor(mlTensor, inputData);

    const gpuTensorBuffer = await anotherMLContext.exportToGPU(mlTensor);

    anotherGPUDevice.destroy();

    gpuTensorBuffer.destroy();

    await promise_rejects_dom(
        t, 'InvalidStateError', anotherMLContext.readTensor(mlTensor));
  }, `${testName} / destroy device after export`);

  promise_test(async t => {
    if (!isExportToGPUSupported) {
      return;
    }

    let anotherGPUDevice = await gpuAdapter.requestDevice();
    let anotherMLContext = await navigator.ml.createContext(anotherGPUDevice);

    let mlTensor = await anotherMLContext.createTensor({
      dataType: 'float32',
      shape: shape,
      exportableToGPU: true,
      readable: true,
      writable: true
    });

    const inputData = new Float32Array(sizeOfShape(shape)).fill(1.0);
    anotherMLContext.writeTensor(mlTensor, inputData);

    anotherGPUDevice.destroy();

    await promise_rejects_dom(
        t, 'InvalidStateError', anotherMLContext.exportToGPU(mlTensor));
    await assert_tensor_data_equals(anotherMLContext, mlTensor, inputData);
  }, `${testName} / destroy device before export`);
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

  testCreateConstantTensor('createConstant', {dataType: 'int32', shape: [4]});
  testCreateConstantTensor(
      'createConstant', {dataType: 'uint8', shape: [3, 2, 4]});

  testCreateConstantTensorFails(
      'createConstantFailsEmptyDimension',
      {dataType: 'int32', shape: [2, 0, 3]});

  testDestroyTensor('destroyTwice');
  testReadTensor('read');
  testWriteTensor('write');
  testDispatchTensor('dispatch');
  testExportToGPU('interop');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
