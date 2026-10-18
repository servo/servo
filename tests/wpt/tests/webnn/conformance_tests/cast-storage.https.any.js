// META: title=WebNN cast and identity preserve exact floating-point storage
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// ULP distance treats signed zeros as equal. Check the actual tensor bits for
// value-preserving identity/casts and the casting algorithm's signed underflow.
// NaN payloads are not required to be preserved.
const halfBits = [
  0x0000, 0x8000, 0x0001, 0x8001, 0x03ff, 0x83ff, 0x0400, 0x8400,
  0x3c00, 0xbc00, 0x7bff, 0xfbff, 0x7c00, 0xfc00, 0x7e00, 0xfe00
];
const widenedBits = [
  0x00000000, 0x80000000, 0x33800000, 0xb3800000,
  0x387fc000, 0xb87fc000, 0x38800000, 0xb8800000,
  0x3f800000, 0xbf800000, 0x477fe000, 0xc77fe000,
  0x7f800000, 0xff800000, 0x7fc00000, 0xffc00000
];
const floatValues = new Float32Array([
  0, -0, 1.0003, -1.0003, 2 ** -149, -(2 ** -149), 65504, -65504,
  1, -1, 3.4e38, -3.4e38, Infinity, -Infinity, NaN, -NaN
]);
const underflowValues = new Float32Array([
  0, -0, 2 ** -26, -(2 ** -26), 2 ** -25, -(2 ** -25),
  3 * 2 ** -25, -3 * 2 ** -25, 5 * 2 ** -25, -5 * 2 ** -25,
  1.00048828125, -1.00048828125
]);

const storageCases = [
  {name: 'float16 identity', op: 'identity', inputType: 'float16',
   outputType: 'float16', data: new Uint8Array(new Uint16Array(halfBits).buffer),
   expected: halfBits},
  {name: 'same-type float16 cast', op: 'cast', inputType: 'float16',
   outputType: 'float16', data: new Uint8Array(new Uint16Array(halfBits).buffer),
   expected: halfBits},
  {name: 'float16 to float32 cast', op: 'cast', inputType: 'float16',
   outputType: 'float32', data: new Uint8Array(new Uint16Array(halfBits).buffer),
   expected: widenedBits},
  {name: 'same-type float32 cast', op: 'cast', inputType: 'float32',
   outputType: 'float32', data: floatValues,
   expected: Array.from(new Uint32Array(floatValues.buffer))},
  {name: 'float32 to float16 signed underflow and ties', op: 'cast',
   inputType: 'float32', outputType: 'float16', data: underflowValues,
   expected: [0, 0x8000, 0, 0x8000, 0, 0x8000, 2, 0x8002, 2, 0x8002,
              0x3c00, 0xbc00]}
];

// Do not add compatible widening casts: these tests inspect the exact native
// storage path. Optional global tensor limits may make that path unavailable,
// but missing required operator support must not turn into a skipped test.
const assertExactStorageSupported = (context, fixture, shape, constant) => {
  const limits = context.opSupportLimits();
  const supported = (operand, dataType) =>
      operand.dataTypes.includes(dataType) &&
      operand.rankRange.min <= shape.length &&
      shape.length <= operand.rankRange.max;
  for (const [name, dataType] of [
    ['input', fixture.inputType], ['output', fixture.outputType]
  ]) {
    const available = supported(limits[fixture.op][name], dataType);
    const required = checkMinimum(
        {dataType, shape}, requiredDataTypesAndRanks[fixture.op][name]);
    const message = `${fixture.op} ${name} must support ${dataType} rank ${shape.length}`;
    if (required) {
      assert_true(available, message);
    } else {
      assert_implements_optional(available, message);
    }
  }
  assert_implements_optional(
      supported(constant ? limits.constant : limits.input, fixture.inputType) &&
          supported(limits.output, fixture.outputType),
      'The exact input/constant and output tensor types and ranks must be supported');
};

promise_setup(getRequiredDataTypesAndRanks);

for (const fixture of storageCases) {
  const shapes = fixture.expected.length === 16 ? [[16], [1, 2, 2, 4]] : [[12]];
  for (const shape of shapes) {
    for (const constant of [false, true]) {
      promise_test(async t => {
        assert_implements(navigator.ml, 'WebNN must be implemented');
        const context = await getContext();
        t.add_cleanup(() => context.destroy());
        assertExactStorageSupported(context, fixture, shape, constant);
        const builder = new MLGraphBuilder(context);
        const descriptor = {dataType: fixture.inputType, shape};
        const input = constant ? builder.constant(descriptor, fixture.data) :
                                 builder.input('input', descriptor);
        const result = fixture.op === 'identity' ? builder.identity(input) :
                                                  builder.cast(input, fixture.outputType);
        const graph = await builder.build({result});
        t.add_cleanup(() => graph.destroy());
        const output = await context.createTensor(
            {dataType: fixture.outputType, shape, readable: true});
        const inputs = {};
        if (!constant) {
          inputs.input = await context.createTensor({...descriptor, writable: true});
          context.writeTensor(inputs.input, fixture.data);
        }
        for (let iteration = 0; iteration < 2; ++iteration) {
          context.dispatch(graph, inputs, {result: output});
          const buffer = await context.readTensor(output);
          const actual = fixture.outputType === 'float16' ?
              new Uint16Array(buffer) : new Uint32Array(buffer);
          assert_equals(actual.length, fixture.expected.length);
          const mask = fixture.outputType === 'float16' ? 0x7fff : 0x7fffffff;
          const infinity = fixture.outputType === 'float16' ? 0x7c00 : 0x7f800000;
          for (let i = 0; i < actual.length; ++i) {
            if ((fixture.expected[i] & mask) > infinity) {
              assert_greater_than(actual[i] & mask, infinity, `NaN at ${i}`);
            } else {
              assert_equals(actual[i], fixture.expected[i],
                            `iteration ${iteration}, element ${i}`);
            }
          }
        }
      }, `${fixture.name}, ${constant ? 'constant' : 'runtime'}, shape [${shape}]`);
    }
  }
}
