// META: title=test input with special character names
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#dom-mloperatoroptions-label

let mlContext;

// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

const specialNameArray = [
  ['12-L#!.☺', '🤦🏼‍♂️124DS#!F'],

  // Escape Sequence
  ['\0node_a', '\0node_b'],
  ['node\0a', 'node\0b'],

  // Hexadecimal Escape Sequences
  // '\x41'→ 'A'
  ['\x41\x41\x41', '\x42\x42\x42'],

  // Unicode & Hexadecimal Characters
  //   "\u00A9" → "©"
  //   "\xA9" → "©"
  //   "\u2665" → "♥"
  //   "\u2026" → "…"
  //   "\U0001F600" → 😀 (Grinning Face Emoji)
  ['\u00A9\xA9\u2665\u2026', '\U0001F600']
];

specialNameArray.forEach((name) => {
  promise_test(async () => {
    // The following code builds a graph as:
    // constant1 ---+
    //              +--- Add (label_0) ---> intermediateOutput1 ---+
    // input1    ---+                                              |
    //                                                             +--- Mul---> output
    // constant2 ---+                                              |
    //              +--- Add (label_1) ---> intermediateOutput2 ---+
    // input2    ---+

    const TENSOR_DIMS = [1, 2, 2, 2];
    const TENSOR_SIZE = 8;

    const builder = new MLGraphBuilder(mlContext);
    const desc = { dataType: 'float32', shape: TENSOR_DIMS };
    const constantBuffer1 = new Float32Array(TENSOR_SIZE).fill(0.5);
    const constant1 = builder.constant(desc, constantBuffer1);

    const input1 = builder.input('input1', desc);
    const constantBuffer2 = new Float32Array(TENSOR_SIZE).fill(0.5);
    const constant2 = builder.constant(desc, constantBuffer2);

    const input2 = builder.input('input2', desc);

    const intermediateOutput1 = builder.add(constant1, input1, {label: name[0]});
    const intermediateOutput2 = builder.add(constant2, input2, {label: name[1]});

    const output = builder.mul(intermediateOutput1, intermediateOutput2);
    const graph = await builder.build({'output': output});

    const inputBuffer1 = new Float32Array(TENSOR_SIZE).fill(1);
    const inputBuffer2 = new Float32Array(TENSOR_SIZE).fill(1);

    desc.writable = true;
    const inputTensor1 = await mlContext.createTensor(desc);
    const inputTensor2 = await mlContext.createTensor(desc);
    mlContext.writeTensor(inputTensor1, inputBuffer1);
    mlContext.writeTensor(inputTensor2, inputBuffer2);

    const outputTensor = await mlContext.createTensor({
      ...desc,
      readable: true,
      writable: false,
    });

    const inputs = {
      'input1': inputTensor1,
      'input2': inputTensor2,
    };
    const outputs = {'output': outputTensor};
    mlContext.dispatch(graph, inputs, outputs);

    assert_array_equals(
      new Float32Array(await mlContext.readTensor(outputTensor)),
      Float32Array.from([2.25, 2.25, 2.25, 2.25, 2.25, 2.25, 2.25, 2.25]));
  }, `'add' nodes with special character name '${name[0]}' and '${name[1]}'`);
});
