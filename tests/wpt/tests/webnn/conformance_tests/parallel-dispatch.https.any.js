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
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };

  const [mlGraph, inputBuffer1, inputBuffer2, outputBuffer] =
      await Promise.all([
        buildMulGraph(mlContext, operandDescriptor, 2),
        mlContext.createBuffer(operandDescriptor),
        mlContext.createBuffer(operandDescriptor),
        mlContext.createBuffer(operandDescriptor)
      ]);

  mlContext.writeBuffer(inputBuffer1, Float32Array.from([1]));
  mlContext.writeBuffer(inputBuffer2, Float32Array.from([10]));

  let readBufferPromises = [];

  mlContext.dispatch(
      mlGraph, {'input': inputBuffer1}, {'output': outputBuffer});

  // Don't await buffer readback before dispatching again.
  readBufferPromises.push(mlContext.readBuffer(outputBuffer));

  mlContext.dispatch(
      mlGraph, {'input': inputBuffer2}, {'output': outputBuffer});

  readBufferPromises.push(mlContext.readBuffer(outputBuffer));

  const actualOutputs =
      await Promise.all(readBufferPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [2, 20]);
}, 'dispatch queues behind readBuffer');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 3);

  // write/dispatch/read, write/dispatch/read, ...
  const testInputs = [1, 2, 3, 4];
  const actualOutputs = await Promise.all(testInputs.map(async (input) => {
    const [inputBuffer, outputBuffer] = await Promise.all([
      mlContext.createBuffer(operandDescriptor),
      mlContext.createBuffer(operandDescriptor)
    ]);

    mlContext.writeBuffer(inputBuffer, Float32Array.from([input]));

    mlContext.dispatch(
        mlGraph, {'input': inputBuffer}, {'output': outputBuffer});

    const output = await mlContext.readBuffer(outputBuffer);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [3, 6, 9, 12]);
}, 'same graph: write/dispatch/read, write/dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 10);

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const inputAndOutputBuffers =
      await Promise.all(testInputs.map(async (testInput) => {
        const [inputBuffer, outputBuffer] = await Promise.all([
          mlContext.createBuffer(operandDescriptor),
          mlContext.createBuffer(operandDescriptor)
        ]);

        mlContext.writeBuffer(inputBuffer, Float32Array.from([testInput]));
        return [inputBuffer, outputBuffer];
      }));

  // dispatch/read, dispatch/read, ...
  let readBufferPromises = [];
  for (let i = 0; i < testInputs.length; i++) {
    mlContext.dispatch(
        mlGraph, {'input': inputAndOutputBuffers[i][0]},
        {'output': inputAndOutputBuffers[i][1]});
    readBufferPromises.push(mlContext.readBuffer(inputAndOutputBuffers[i][1]));
  };

  const actualOutputs =
      await Promise.all(readBufferPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [10, 20, 30, 40]);
}, 'same graph: write/write..., dispatch/read, dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 9);

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const inputAndOutputBuffers =
      await Promise.all(testInputs.map(async (testInput) => {
        const [inputBuffer, outputBuffer] = await Promise.all([
          mlContext.createBuffer(operandDescriptor),
          mlContext.createBuffer(operandDescriptor)
        ]);

        mlContext.writeBuffer(inputBuffer, Float32Array.from([testInput]));
        return [inputBuffer, outputBuffer];
      }));

  // dispatch/dispatch...
  for (let i = 0; i < testInputs.length; i++) {
    mlContext.dispatch(
        mlGraph, {'input': inputAndOutputBuffers[i][0]},
        {'output': inputAndOutputBuffers[i][1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(
      inputAndOutputBuffers.map(async (inputAndOutputBuffer) => {
        const output = await mlContext.readBuffer(inputAndOutputBuffer[1]);
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [9, 18, 27, 36]);
}, 'same graph: write/write..., dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };
  const mlGraph = await buildMulGraph(mlContext, operandDescriptor, 2);

  const buffers = await Promise.all([
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor)
  ]);

  mlContext.writeBuffer(buffers[0], Float32Array.from([1]));

  // dispatch/dispatch...
  for (let i = 0; i < buffers.length - 1; i++) {
    mlContext.dispatch(
        mlGraph, {'input': buffers[i]}, {'output': buffers[i + 1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(buffers.map(async (buffer) => {
    const output = await mlContext.readBuffer(buffer);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [1, 2, 4, 8, 16]);
}, 'same graph serial inputs: dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const graphsAndBuffers =
      await Promise.all(testInputs.map(async (testInput) => {
        const [graph, inputBuffer, outputBuffer] = await Promise.all([
          buildMulGraph(mlContext, operandDescriptor, testInput),
          mlContext.createBuffer(operandDescriptor),
          mlContext.createBuffer(operandDescriptor)
        ]);

        mlContext.writeBuffer(inputBuffer, Float32Array.from([testInput]));
        return [graph, inputBuffer, outputBuffer];
      }));

  // dispatch/read, dispatch/read, ...
  let readBufferPromises = [];
  for (let i = 0; i < graphsAndBuffers.length; i++) {
    mlContext.dispatch(
        graphsAndBuffers[i][0], {'input': graphsAndBuffers[i][1]},
        {'output': graphsAndBuffers[i][2]});
    readBufferPromises.push(mlContext.readBuffer(graphsAndBuffers[i][2]));
  };

  const actualOutputs =
      await Promise.all(readBufferPromises.map(async (promise) => {
        const output = await promise;
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [1, 4, 9, 16]);
}, 'different graphs: write/write..., dispatch/read, dispatch/read, ...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };

  // write/write...
  const testInputs = [1, 2, 3, 4];
  const graphsAndBuffers =
      await Promise.all(testInputs.map(async (testInput) => {
        const [graph, inputBuffer, outputBuffer] = await Promise.all([
          buildMulGraph(mlContext, operandDescriptor, testInput * 2),
          mlContext.createBuffer(operandDescriptor),
          mlContext.createBuffer(operandDescriptor)
        ]);

        mlContext.writeBuffer(inputBuffer, Float32Array.from([testInput]));
        return [graph, inputBuffer, outputBuffer];
      }));

  // dispatch/dispatch...
  for (let i = 0; i < graphsAndBuffers.length; i++) {
    mlContext.dispatch(
        graphsAndBuffers[i][0], {'input': graphsAndBuffers[i][1]},
        {'output': graphsAndBuffers[i][2]});
  };

  // read/read...
  const actualOutputs =
      await Promise.all(graphsAndBuffers.map(async (graphAndBuffers) => {
        const output = await mlContext.readBuffer(graphAndBuffers[2]);
        return new Float32Array(output)[0];
      }));

  assert_array_equals(actualOutputs, [2, 8, 18, 32]);
}, 'different graphs: write/write..., dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };

  const graphs = await Promise.all([3, 2].map(async (multiplier) => {
    return buildMulGraph(mlContext, operandDescriptor, multiplier);
  }));

  const buffers = await Promise.all([
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor)
  ]);

  mlContext.writeBuffer(buffers[0], Float32Array.from([1]));

  // dispatch/dispatch...
  for (let i = 0; i < buffers.length - 1; i++) {
    mlContext.dispatch(
        graphs[i % 2], {'input': buffers[i]}, {'output': buffers[i + 1]});
  }

  // read/read...
  const actualOutputs = await Promise.all(buffers.map(async (buffer) => {
    const output = await mlContext.readBuffer(buffer);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [1, 3, 6, 18, 36]);
}, 'different graphs serial inputs: dispatch/dispatch..., read/read...');

promise_test(async () => {
  const operandDescriptor = {
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
  };

  const graphs = await Promise.all([2, 3].map(async (multiplier) => {
    return buildMulGraph(mlContext, operandDescriptor, multiplier);
  }));

  const buffers = await Promise.all([
    mlContext.createBuffer(operandDescriptor),
    mlContext.createBuffer(operandDescriptor)
  ]);

  // Write to the buffer which will be initially used as an input.
  mlContext.writeBuffer(buffers[0], Float32Array.from([1]));

  // Double the value in one buffer, sticking the result in the other buffer.
  //
  // buffers[0]  buffers[1]
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
        graphs[i % 2], {'input': buffers[i % 2]},
        {'output': buffers[(i + 1) % 2]});
  };

  // read/read...
  const actualOutputs = await Promise.all(buffers.map(async (buffer) => {
    const output = await mlContext.readBuffer(buffer);
    return new Float32Array(output)[0];
  }));

  assert_array_equals(actualOutputs, [216, 72]);
}, 'different graphs using the same buffers');
