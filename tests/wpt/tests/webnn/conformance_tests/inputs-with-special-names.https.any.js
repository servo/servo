// META: title=test input with special character names
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-input

let mlContext;

// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

const specialNameArray = [
  '12-L#!.☺',
  '🤦🏼‍♂️124DS#!F',

  // Escape Sequence
  'hello\n\t\r\b\f\v\'\"\0\\webnn',
  '\0',
  '\0startWithNullCharacter',

  // Hexadecimal Escape Sequences
  // '\x41'→ 'A'
  '\x41\x41\x41',

  // Unicode & Hexadecimal Characters
  //   "\u00A9" → "©"
  //   "\xA9" → "©"
  //   "\u2665" → "♥"
  //   "\u2026" → "…"
  //   "\U0001F600" → 😀 (Grinning Face Emoji)
  '\u00A9\xA9\u2665\u2026',
  '\U0001F600'
];

specialNameArray.forEach((name) => {
  promise_test(async () => {
    const builder = new MLGraphBuilder(mlContext);
    const inputOperand = builder.input(name, {dataType: 'float32', shape: [4]});
    const outputOperand = builder.abs(inputOperand);

    const [inputTensor, outputTensor, mlGraph] = await Promise.all([
      mlContext.createTensor({
        dataType: 'float32',
        shape: [4],
        readable: true,
        writable: true,
      }),
      mlContext.createTensor({dataType: 'float32', shape: [4], readable: true}),
      builder.build({'output': outputOperand})
    ]);

    const inputData = Float32Array.from([-2, -1, 1, 2]);
    mlContext.writeTensor(inputTensor, inputData);

    const inputs = {};
    inputs[name] = inputTensor;

    mlContext.dispatch(mlGraph, inputs, {'output': outputTensor});

    // Wait for graph execution to complete.
    await mlContext.readTensor(outputTensor);

    assert_array_equals(
        new Float32Array(await mlContext.readTensor(outputTensor)),
        Float32Array.from([2, 1, 1, 2]));
  }, `abs input with special character name '${name}'`);
});

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputA = builder.input('input\0a', { dataType: 'float32', shape: [2] });
  const inputB = builder.input('input\0b', { dataType: 'float32', shape: [2] });
  const output = builder.add(inputA, inputB);

  const [inputATensor, inputBTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({ dataType: 'float32', shape: [2], writable: true }),
    mlContext.createTensor({ dataType: 'float32', shape: [2], writable: true }),
    mlContext.createTensor({ dataType: 'float32', shape: [2], readable: true }),
    builder.build({ 'output': output })
  ]);

  const inputAData = Float32Array.from([1, 1]);
  const inputBData = Float32Array.from([2, 2]);
  mlContext.writeTensor(inputATensor, inputAData);
  mlContext.writeTensor(inputBTensor, inputBData);

  const inputs = { 'input\0a': inputATensor, 'input\0b': inputBTensor };
  mlContext.dispatch(mlGraph, inputs, { 'output': outputTensor });

  // Wait for graph execution to complete.
  await mlContext.readTensor(outputTensor);

  assert_array_equals(
    new Float32Array(await mlContext.readTensor(outputTensor)),
    Float32Array.from([3, 3]));
}, `[add] inputs with null character name in the middle`);
