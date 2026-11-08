// META: title=WebNN comparison results used as where conditions
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-logical
// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-where
//
// Every test builds the same two-operator composition, with `a` a constant and
// `b` an input of the same float type:
//
//   const condition = builder.<comparison>(a, b);   // uint8 zero/one
//   const selected = builder.where(condition, one, zero);
//
// where `one` and `zero` are float32 scalar constants, so `selected` is the
// comparison result re-expressed as exact float32 1.0 / 0.0. The expected
// value is computed in JavaScript from the decoded float values and compared
// with zero ULP: `where` is data movement, so any deviation is a wrong truth
// value, not a rounding difference. Each comparison runs twice, once with
// `condition` also exposed as a graph output and once with it kept internal,
// because an implementation may lower the two differently (for example by
// fusing the comparison into the selection, or by converting the internal
// uint8 result through another type) and only the exposed form had coverage.
//
// The float16 operands are generated as bit patterns so the set covers every
// subnormal, both zeros, both infinities and several NaN payloads exactly.
// They are decoded to JavaScript numbers because the expected truth values
// need IEEE semantics, which bit patterns do not give: +0 and -0 compare
// equal, every NaN compares unequal to everything, and negative values order
// in the opposite direction of their bit patterns. The decoded numbers are
// also what the harness feeds to the float16 input and constant.
const decodeComparisonHalf = bits => {
  const sign = bits & 0x8000 ? -1 : 1;
  const exponent = (bits >>> 10) & 31;
  const fraction = bits & 1023;
  if (exponent === 31) {
    return fraction ? NaN : sign * Infinity;
  }
  return exponent === 0 ? sign * fraction * 2 ** -24 :
      sign * (1024 + fraction) * 2 ** (exponent - 25);
};
const comparisonBits = [
  ...Array.from({length: 1023}, (_, index) => index + 1),
  ...Array.from({length: 1023}, (_, index) => 0x8001 + index),
  0, 0x8000, 0x0400, 0x8400, 0x3555, 0xb555, 0x3c00, 0xbc00,
  0x3c01, 0xbc01, 0x3800, 0xb800, 0x4000, 0xc000, 0x7bff, 0xfbff,
  0x7c00, 0xfc00, 0x7e00, 0xfe00, 0x7e01, 0xfe01, 0x7c01, 0xfc01,
  0x1000, 0x9000, 0x2000, 0xa000, 0x0800, 0x8800, 0x4c00, 0xcc00,
  0x6c00, 0xec00
];
const comparisonThresholdBits = [
  0, 0x8000, 1, 0x8001, 0x0200, 0x8200, 0x03ff, 0x83ff,
  0x0400, 0x8400, 0x3c00, 0xbc00, 0x7e01, 0x7c00, 0xfc00, 0x3555
];
const comparisonInputs = comparisonBits.map(decodeComparisonHalf);
const comparisonOther = comparisonBits.map((_, index) =>
  decodeComparisonHalf(
      comparisonThresholdBits[index % comparisonThresholdBits.length]));
// The reference predicate for each WebNN comparison operator, evaluated on the
// decoded float values exactly as the specification defines the operator.
const comparisonFunctions = {
  equal: (a, b) => a === b,
  notEqual: (a, b) => a !== b,
  greater: (a, b) => a > b,
  greaterOrEqual: (a, b) => a >= b,
  lesser: (a, b) => a < b,
  lesserOrEqual: (a, b) => a <= b
};

const comparisonSelectTests = [];
for (const dataType of ['float16', 'float32']) {
  for (const [operation, compare] of Object.entries(comparisonFunctions)) {
    for (const exposeComparison of [false, true]) {
      const descriptor = {shape: [comparisonInputs.length], dataType};
      const expected = comparisonInputs.map(
          (value, index) => Number(compare(value, comparisonOther[index])));
      const expectedOutputs = {
        selected: {
          data: expected,
          descriptor: {shape: descriptor.shape, dataType: 'float32'}
        }
      };
      if (exposeComparison) {
        expectedOutputs.condition = {
          data: expected,
          descriptor: {shape: descriptor.shape, dataType: 'uint8'}
        };
      }
      comparisonSelectTests.push({
        name: operation + ' ' + dataType +
            ' constant/input result selects exact zero/one' +
            (exposeComparison ? ' with both outputs' : ' with select output'),
        graph: {
          inputs: {
            a: {data: comparisonInputs, descriptor, constant: true},
            b: {data: comparisonOther, descriptor},
            one: {
              data: [1], descriptor: {shape: [], dataType: 'float32'},
              constant: true
            },
            zero: {
              data: [0], descriptor: {shape: [], dataType: 'float32'},
              constant: true
            }
          },
          operators: [
            {
              name: operation,
              arguments: [{a: 'a'}, {b: 'b'}],
              outputs: 'condition'
            },
            {
              name: 'where',
              arguments: [
                {condition: 'condition'}, {trueValue: 'one'},
                {falseValue: 'zero'}
              ],
              outputs: 'selected'
            }
          ],
          expectedOutputs
        }
      });
    }
  }
}

webnn_conformance_test(
    comparisonSelectTests, buildAndExecuteGraph, getZeroULPTolerance);
