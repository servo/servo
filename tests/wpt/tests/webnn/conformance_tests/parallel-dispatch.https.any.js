// META: title=test parallel WebNN API dispatch calls
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlcontext-dispatch

let mlContext;


// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

function buildMulGraph(context, operandDescriptor, multiplier) {
  // Construct a simple graph: A = B * `multiplier`.
  const builder = new MLGraphBuilder(context);
  const inputOperand = builder.input('input', operandDescriptor);
  const constantOperand =
      builder.constant(operandDescriptor, Float32Array.from([multiplier]));
  const outputOperand = builder.mul(inputOperand, constantOperand);
  return builder.build({'output': outputOperand});
}

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };

  const [mlGraph, inputTensor1, inputTensor2, outputTensor] =
      await Promise.all([
        buildMulGraph(mlContext, operandDescriptor, 2),
        mlContext.createTensor(operandDescriptor),
        mlContext.createTensor(operandDescriptor),
        mlContext.createTensor(operandDescriptor)
      ]);

  mlContext.writeTensor(inputTensor1, Float32Array.from([1]));
  mlContext.writeTensor(inputTensor2, Float32Array.from([10]));

  let readTensorPromises = [];

  mlContext.dispatch(
      mlGraph, {'input': inputTensor1}, {'output': outputTensor});

  // Don't await tensor readback before dispatching again.
  readTensorPromises.push(mlContext.readTensor(outputTensor));

  mlContext.dispatch(
      mlGraph, {'input': inputTensor2}, {'output': outputTensor});

  readTensorPromises.push(mlContext.readTensor(outputTensor));

  const actualOutputs =
      await Promise.all(readTensorPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [2, 20]);
}, 'dispatch queues behind readTensor');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 3);

  // write/dispatch/read, write/dispatch/read, ...
  const testInputs = [1, 2, 3, 4];
  const actualOutputs = await Promise.all(testInputs.map(async (input) => {
    const [inputTensor, outputTensor] = await Promise.all([
      mlContext.createTensor(operandDescriptor),
      mlContext.createTensor(operandDescriptor)
    ]);

    mlContext.writeTensor(inputTensor, Float32Array.from([input]));

    mlContext.dispatch(
        mlGraph, {'input': inputTensor}, {'output': outputTensor});

    const output = await mlContext.readTensor(outputTensor);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [3, 6, 9, 12]);
}, 'same graph: write/dispatch/read, write/dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 10);

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const inputAndOutputTensors =
      await Promise.all(testInputs.map(async (testInput) => {
        const [inputTensor, outputTensor] = await Promise.all([
          mlContext.createTensor(operandDescriptor),
          mlContext.createTensor(operandDescriptor)
        ]);

        mlContext.writeTensor(inputTensor, Float32Array.from([testInput]));
        return [inputTensor, outputTensor];
      }));

  // dispatch/read, dispatch/read, ...
  let readTensorPromises = [];
  for (let i = 0; i < testInputs.length; i++) {
    mlContext.dispatch(
        mlGraph, {'input': inputAndOutputTensors[i][0]},
        {'output': inputAndOutputTensors[i][1]});
    readTensorPromises.push(mlContext.readTensor(inputAndOutputTensors[i][1]));
  };

  const actualOutputs =
      await Promise.all(readTensorPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [10, 20, 30, 40]);
}, 'same graph: write/write..., dispatch/read, dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 9);

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const inputAndOutputTensors =
      await Promise.all(testInputs.map(async (testInput) => {
        const [inputTensor, outputTensor] = await Promise.all([
          mlContext.createTensor(operandDescriptor),
          mlContext.createTensor(operandDescriptor)
        ]);

        mlContext.writeTensor(inputTensor, Float32Array.from([testInput]));
        return [inputTensor, outputTensor];
      }));

  // dispatch/dispatch...
  for (let i = 0; i < testInputs.length; i++) {
    mlContext.dispatch(
        mlGraph, {'input': inputAndOutputTensors[i][0]},
        {'output': inputAndOutputTensors[i][1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(
      inputAndOutputTensors.map(async (inputAndOutputTensor) => {
        const output = await mlContext.readTensor(inputAndOutputTensor[1]);
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [9, 18, 27, 36]);
}, 'same graph: write/write..., dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 2);

  const tensors = await Promise.all([
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor)
  ]);

  mlContext.writeTensor(tensors[0], Float32Array.from([1]));

  // dispatch/dispatch...
  for (let i = 0; i < tensors.length - 1; i++) {
    mlContext.dispatch(
        mlGraph, {'input': tensors[i]}, {'output': tensors[i + 1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(tensors.map(async (tensor) => {
    const output = await mlContext.readTensor(tensor);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [1, 2, 4, 8, 16]);
}, 'same graph serial inputs: dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const graphsAndTensors =
      await Promise.all(testInputs.map(async (testInput) => {
        const [graph, inputTensor, outputTensor] = await Promise.all([
          buildMulGraph(mlContext, operandDescriptor, testInput),
          mlContext.createTensor(operandDescriptor),
          mlContext.createTensor(operandDescriptor)
        ]);

        mlContext.writeTensor(inputTensor, Float32Array.from([testInput]));
        return [graph, inputTensor, outputTensor];
      }));

  // dispatch/read, dispatch/read, ...
  let readTensorPromises = [];
  for (let i = 0; i < graphsAndTensors.length; i++) {
    mlContext.dispatch(
        graphsAndTensors[i][0], {'input': graphsAndTensors[i][1]},
        {'output': graphsAndTensors[i][2]});
    readTensorPromises.push(mlContext.readTensor(graphsAndTensors[i][2]));
  };

  const actualOutputs =
      await Promise.all(readTensorPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [1, 4, 9, 16]);
}, 'different graphs: write/write..., dispatch/read, dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const graphsAndTensors =
      await Promise.all(testInputs.map(async (testInput) => {
        const [graph, inputTensor, outputTensor] = await Promise.all([
          buildMulGraph(mlContext, operandDescriptor, testInput * 2),
          mlContext.createTensor(operandDescriptor),
          mlContext.createTensor(operandDescriptor)
        ]);

        mlContext.writeTensor(inputTensor, Float32Array.from([testInput]));
        return [graph, inputTensor, outputTensor];
      }));

  // dispatch/dispatch...
  for (let i = 0; i < graphsAndTensors.length; i++) {
    mlContext.dispatch(
        graphsAndTensors[i][0], {'input': graphsAndTensors[i][1]},
        {'output': graphsAndTensors[i][2]});
  };

  // read/read...
  const actualOutputs =
      await Promise.all(graphsAndTensors.map(async (graphAndTensors) => {
        const output = await mlContext.readTensor(graphAndTensors[2]);
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [2, 8, 18, 32]);
}, 'different graphs: write/write..., dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };

  const graphs = await Promise.all([3, 2].map(async (multiplier) => {
    return buildMulGraph(mlContext, operandDescriptor, multiplier);
  }));

  const tensors = await Promise.all([
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor)
  ]);

  mlContext.writeTensor(tensors[0], Float32Array.from([1]));

  // dispatch/dispatch...
  for (let i = 0; i < tensors.length - 1; i++) {
    mlContext.dispatch(
        graphs[i % 2], {'input': tensors[i]}, {'output': tensors[i + 1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(tensors.map(async (tensor) => {
    const output = await mlContext.readTensor(tensor);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [1, 3, 6, 18, 36]);
}, 'different graphs serial inputs: dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE | MLTensorUsage.READ,
  };

  const graphs = await Promise.all([2, 3].map(async (multiplier) => {
    return buildMulGraph(mlContext, operandDescriptor, multiplier);
  }));

  const tensors = await Promise.all([
    mlContext.createTensor(operandDescriptor),
    mlContext.createTensor(operandDescriptor)
  ]);

  // Write to the tensor which will be initially used as an input.
  mlContext.writeTensor(tensors[0], Float32Array.from([1]));

  // Double the value in one tensor, sticking the result in the other tensor.
  //
  // tensors[0]  tensors[1]
  //     1
  //        >---->  2
  //     6  <----<
  //        >---->  12
  //     36 <----<
  //        >---->  72
  //    216 <----<

  // dispatch/dispatch...
  for (let i = 0; i < 6; i++) {
    mlContext.dispatch(
        graphs[i % 2], {'input': tensors[i % 2]},
        {'output': tensors[(i + 1) % 2]});
  };

  // read/read...
  const actualOutputs = await Promise.all(tensors.map(async (tensor) => {
    const output = await mlContext.readTensor(tensor);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [216, 72]);
}, 'different graphs using the same tensors');
