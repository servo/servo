// META: title=WebNN float32 underflow boundaries
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Use the existing standalone Exp/Sqrt/Mul/Div allowances. In particular,
// supplied subnormal inputs to Sqrt/Mul/Div have normal, nonzero results.
// References were rounded to float32 from independent 160- and 240-digit
// calculations (mpmath and Decimal); no graph composition budget is assumed.
const underflowBudgets = {exp: 32, sqrt: 1, mul: 1, div: 2};
const getUnderflowTolerance = graph => ({
  metricType: 'ULP', value: underflowBudgets[graph.operators[0].name]
});
const underflowTests = [];
const unaryTestCases = [
  {op: 'exp',
    input: [-95, -90, -88, -87],
    expected: [5.521115949439779e-42, 8.194008692231508e-40, 6.054601485195952e-39, 1.6458114537543937e-38]},
  {op: 'sqrt',
    input: [1.401298464324817e-45, 1.1754942106924411e-38, 1.1754943508222875e-38],
    expected: [3.743392066509216e-23, 1.0842021078620191e-19, 1.0842021724855044e-19]},
];
for (const {op, input, expected} of unaryTestCases) {
  for (const constant of [false, true]) {
    for (const scalar of [false, true]) {
      const descriptor = {shape: scalar ? [] : [input.length], dataType: 'float32'};
      underflowTests.push({
        name: `${op} float32 underflow ${scalar ? 'scalar' : 'vector'} ` +
            `${constant ? 'constant' : 'runtime'}`,
        graph: {
          inputs: {input: {
            data: scalar ? [input[0]] : input, descriptor, constant
          }},
          operators: [{name: op, arguments: [{input: 'input'}], outputs: 'output'}],
          expectedOutputs: {output: {
            data: scalar ? [expected[0]] : expected, descriptor
          }}
        }
      });
    }
  }
}

// All products/quotients are exact dyadics. The large scalar is normal too:
// 2^-149 * 2^126 = 2^-23, and 2^-149 / 2^-126 = 2^-23.
const binaryTestCases = [
  {op: 'mul', b: 2 ** 126},
  {op: 'div', b: 2 ** -126},
];
const scalarTestData = {a: [2 ** -149], expected: [2 ** -23]};
const vectorTestData = {
  a: [2 ** -149, -(2 ** -149), 2 ** -127, -(2 ** -127)],
  expected: [2 ** -23, -(2 ** -23), 0.5, -0.5]
};
for (const {op, b} of binaryTestCases) {
  for (const constant of [false, true]) {
    for (const scalar of [false, true]) {
      const {a, expected} = scalar ? scalarTestData : vectorTestData;
      const descriptor = {shape: scalar ? [] : [a.length], dataType: 'float32'};
      underflowTests.push({
        name: `${op} float32 subnormal to normal ${scalar ? 'scalar' : 'vector'} ` +
            `${constant ? 'constant' : 'runtime'}`,
        graph: {
          inputs: {
            a: {data: a, descriptor, constant},
            b: {data: [b], descriptor: {shape: [], dataType: 'float32'}, constant}
          },
          operators: [{name: op, arguments: [{a: 'a'}, {b: 'b'}], outputs: 'output'}],
          expectedOutputs: {output: {data: expected, descriptor}}
        }
      });
    }
  }
}
webnn_conformance_test(underflowTests, buildAndExecuteGraph, getUnderflowTolerance);
