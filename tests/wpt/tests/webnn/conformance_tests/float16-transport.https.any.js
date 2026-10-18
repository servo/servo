// META: title=WebNN float16 subnormal value transport
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Every value is exactly represented by float16. Value movement must not
// flush its subnormals, including at the normal/subnormal boundary.
const transportValues = [
  2 ** -24, 3 * 2 ** -24, 1023 * 2 ** -24,
  -(2 ** -24), -3 * 2 ** -24, -1023 * 2 ** -24,
  2 ** -14, -(2 ** -14)
];
const transportTensor = (data, shape, dataType = 'float16') =>
    ({data, descriptor: {shape, dataType}});
const transportTest = (name, inputs, operator, data, shape) => ({
  name: `${name} preserves float16 subnormal values`,
  graph: {
    inputs,
    operators: [operator],
    expectedOutputs: {result: transportTensor(data, shape)}
  }
});
const transportOp = (name, arguments_) =>
    ({name, arguments: arguments_, outputs: 'result'});
const transportInput = {input: transportTensor(transportValues, [8])};
const transportReversed = transportValues.toReversed();
const transportTests = [
  transportTest('same-type cast', transportInput,
      transportOp('cast', [{input: 'input'}, {type: 'float16'}]),
      transportValues, [8]),
  transportTest('same-type cast constant', {
    input: {...transportInput.input, constant: true}
  }, transportOp('cast', [{input: 'input'}, {type: 'float16'}]),
      transportValues, [8]),
  transportTest('same-type cast scalar', {
    input: transportTensor([2 ** -24], [])
  }, transportOp('cast', [{input: 'input'}, {type: 'float16'}]),
      [2 ** -24], []),
  transportTest('identity', transportInput,
      transportOp('identity', [{input: 'input'}]), transportValues, [8]),
  transportTest('identity constant', {
    input: {...transportInput.input, constant: true}
  }, transportOp('identity', [{input: 'input'}]), transportValues, [8]),
  transportTest('identity scalar', {
    input: transportTensor([2 ** -24], [])
  }, transportOp('identity', [{input: 'input'}]), [2 ** -24], []),
  transportTest('reshape', transportInput,
      transportOp('reshape', [{input: 'input'}, {newShape: [2, 4]}]),
      transportValues, [2, 4]),
  transportTest('transpose', {
    input: transportTensor(transportValues, [2, 4])
  }, transportOp('transpose', [
    {input: 'input'}, {options: {permutation: [1, 0]}}
  ]), [0, 4, 1, 5, 2, 6, 3, 7].map(i => transportValues[i]), [4, 2]),
  transportTest('reverse', transportInput,
      transportOp('reverse', [{input: 'input'}, {options: {axes: [0]}}]),
      transportReversed, [8]),
  transportTest('slice', transportInput,
      transportOp('slice', [{input: 'input'}, {starts: [1]}, {sizes: [7]}]),
      transportValues.slice(1), [7]),
  transportTest('positive-strided slice', {
    input: transportTensor(transportValues.flatMap(value => [value, value]), [16])
  }, transportOp('slice', [
    {input: 'input'}, {starts: [1]}, {sizes: [15]}, {options: {strides: [2]}}
  ]), transportValues, [8]),
  transportTest('concat', {
    left: transportTensor(transportValues.slice(0, 4), [4]),
    right: transportTensor(transportValues.slice(4), [4])
  }, transportOp('concat', [{inputs: ['left', 'right']}, {axis: 0}]),
      transportValues, [8]),
  {
    name: 'split preserves float16 subnormal values in both outputs',
    graph: {
      inputs: transportInput,
      operators: [{
        name: 'split', arguments: [{input: 'input'}, {splits: [4, 4]}],
        outputs: ['first', 'second']
      }],
      expectedOutputs: {
        first: transportTensor(transportValues.slice(0, 4), [4]),
        second: transportTensor(transportValues.slice(4), [4])
      }
    }
  },
  transportTest('gather', {
    ...transportInput,
    indices: transportTensor([7, 6, 5, 4, 3, 2, 1, 0], [8], 'int32')
  }, transportOp('gather', [{input: 'input'}, {indices: 'indices'}]),
      transportReversed, [8]),
  transportTest('gatherElements', {
    input: transportTensor(transportValues, [2, 4]),
    indices: {...transportTensor([3, 2, 1, 0, 3, 2, 1, 0], [2, 4], 'int32'),
      constant: true}
  }, transportOp('gatherElements', [
    {input: 'input'}, {indices: 'indices'}, {options: {axis: 1}}
  ]), [3, 2, 1, 0, 7, 6, 5, 4].map(i => transportValues[i]), [2, 4]),
  transportTest('gatherND', {
    input: transportTensor(transportValues, [2, 4]),
    indices: transportTensor([1, 3, 0, 0, 1, 0, 0, 3], [4, 2], 'int32')
  }, transportOp('gatherND', [{input: 'input'}, {indices: 'indices'}]),
      [7, 0, 4, 3].map(i => transportValues[i]), [4]),
  transportTest('expand', {
    input: transportTensor(transportValues, [1, 8])
  }, transportOp('expand', [{input: 'input'}, {newShape: [2, 8]}]),
      [...transportValues, ...transportValues], [2, 8]),
  transportTest('tile', {
    input: transportTensor(transportValues, [1, 8])
  }, transportOp('tile', [{input: 'input'}, {repetitions: [2, 1]}]),
      [...transportValues, ...transportValues], [2, 8]),
  transportTest('where', {
    condition: transportTensor([0, 1, 0, 1, 0, 1, 0, 1], [8], 'uint8'),
    yes: transportTensor(transportValues, [8]),
    no: transportTensor(transportReversed, [8])
  }, transportOp('where', [
    {condition: 'condition'}, {trueValue: 'yes'}, {falseValue: 'no'}
  ]), transportValues.map((value, i) => i % 2 ? value : transportReversed[i]), [8]),
  transportTest('constant pad', transportInput, transportOp('pad', [
    {input: 'input'}, {beginningPadding: [1]}, {endingPadding: [2]},
    {options: {value: 2 ** -24}}
  ]), [2 ** -24, ...transportValues, 2 ** -24, 2 ** -24], [11]),
  transportTest('edge pad', transportInput, transportOp('pad', [
    {input: 'input'}, {beginningPadding: [1]}, {endingPadding: [1]},
    {options: {mode: 'edge'}}
  ]), [transportValues[0], ...transportValues, transportValues[7]], [10]),
  transportTest('reflection pad', transportInput, transportOp('pad', [
    {input: 'input'}, {beginningPadding: [1]}, {endingPadding: [1]},
    {options: {mode: 'reflection'}}
  ]), [transportValues[1], ...transportValues, transportValues[6]], [10]),
  transportTest('scatterElements', {
    input: transportTensor(Array(8).fill(0), [8]),
    indices: {...transportTensor([7, 6, 5, 4, 3, 2, 1, 0], [8], 'int32'),
      constant: true},
    updates: transportTensor(transportValues, [8])
  }, transportOp('scatterElements', [
    {input: 'input'}, {indices: 'indices'}, {updates: 'updates'}
  ]), transportReversed, [8]),
  transportTest('scatterND', {
    input: transportTensor(Array(8).fill(0), [8]),
    indices: transportTensor([7, 6, 5, 4, 3, 2, 1, 0], [8, 1], 'int32'),
    updates: transportTensor(transportValues, [8])
  }, transportOp('scatterND', [
    {input: 'input'}, {indices: 'indices'}, {updates: 'updates'}
  ]), transportReversed, [8])
];

webnn_conformance_test(transportTests, buildAndExecuteGraph, getZeroULPTolerance);

// Feed the native output tensor directly into the next dispatch, without
// re-uploading state from the host. Read both tensors afterward to check that the
// dispatch preserved the input as well as producing the correct next state.
const stateBits = [
  0x0000, 0x8000, 0x0001, 0x8001, 0x0003, 0x8003,
  0x03ff, 0x83ff, 0x0400, 0x8400
];
for (const op of ['identity', 'cast']) {
  for (const shape of [[10], [1, 2, 1, 5]]) {
    promise_test(async t => {
      assert_implements(navigator.ml, 'WebNN must be implemented');
      const context = await getContext();
      t.add_cleanup(() => context.destroy());
      const limits = context.opSupportLimits();
      const supported = operand =>
          operand.dataTypes.includes('float16') &&
          operand.rankRange.min <= shape.length &&
          shape.length <= operand.rankRange.max;
      assert_true(supported(limits[op].input),
          `${op} must support the required float16 input rank`);
      assert_true(supported(limits[op].output),
          `${op} must support the required float16 output rank`);
      assert_implements_optional(
          supported(limits.input) && supported(limits.output),
          'The exact float16 input/output tensor ranks must be supported');
      const builder = new MLGraphBuilder(context);
      const descriptor = {dataType: 'float16', shape};
      const input = builder.input('input', descriptor);
      const result = op === 'identity' ? builder.identity(input) :
                                       builder.cast(input, 'float16');
      const graph = await builder.build({result});
      t.add_cleanup(() => graph.destroy());
      const first = await context.createTensor(
          {...descriptor, readable: true, writable: true});
      const second = await context.createTensor(
          {...descriptor, readable: true, writable: true});
      for (const bits of [stateBits, stateBits.toReversed()]) {
        context.writeTensor(first, new Uint16Array(bits));
        let current = first;
        let next = second;
        for (let step = 0; step < 8; ++step) {
          context.dispatch(graph, {input: current}, {result: next});
          assert_array_equals(
              new Uint16Array(await context.readTensor(next)), bits,
              `step ${step} preserves the next state's exact bits`);
          assert_array_equals(
              new Uint16Array(await context.readTensor(current)), bits,
              `step ${step} does not modify the input state`);
          [current, next] = [next, current];
        }
      }
    }, `${op} float16 signed/subnormal state ping-pong, shape [${shape}]`);
  }
}
