// META: title=WebNN full-range int32 arithmetic and selection
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Adjacent integers above 2^24 must not become false ties through float32.
// No division by zero or overflowing quotient is used. Integer division
// truncates toward zero; all these operations require exact integer results.
const fullRangeIntegerTests = [];
const selectionA = [16777216, 16777217, -16777216, -16777217,
  -2147483648, -2147483647, 2147483647, 2147483646];
const selectionB = [16777217, 16777216, -16777217, -16777216,
  -2147483647, -2147483648, 2147483646, 2147483647];
const integerTestCases = [
  {op: 'min', a: selectionA, b: selectionB,
    expected: [16777216, 16777216, -16777217, -16777217,
      -2147483648, -2147483648, 2147483646, 2147483646]},
  {op: 'max', a: selectionA, b: selectionB,
    expected: [16777217, 16777217, -16777216, -16777216,
      -2147483647, -2147483647, 2147483647, 2147483647]},
  {op: 'div',
    a: [-33554435, 33554435, -2147483648, 2147483647, -7, 7],
    b: [2, -2, 1, 1, 3, -3],
    expected: [-16777217, -16777217, -2147483648, 2147483647, -2, -2]},
];
for (const {op, a, b, expected} of integerTestCases) {
  for (const constant of [false, true]) {
    const descriptor = {shape: [a.length], dataType: 'int32'};
    fullRangeIntegerTests.push({
      name: `${op} int32 full range ${constant ? 'constant' : 'runtime'}`,
      graph: {
        inputs: {
          a: {data: a, descriptor, constant},
          b: {data: b, descriptor, constant}
        },
        operators: [{name: op, arguments: [{a: 'a'}, {b: 'b'}], outputs: 'output'}],
        expectedOutputs: {output: {data: expected, descriptor}}
      }
    });
  }
}

// Every row has a unique winner. Opposite row orders detect either choice
// of index after a faulty conversion collapses adjacent integers into ties.
for (const op of ['argMin', 'argMax']) {
  for (const constant of [false, true]) {
    fullRangeIntegerTests.push({
      name: `${op} int32 adjacent unique winners ${constant ? 'constant' : 'runtime'}`,
      graph: {
        inputs: {
          input: {
            data: [16777216, 16777217, 16777217, 16777216,
              -2147483648, -2147483647, 2147483647, 2147483646],
            descriptor: {shape: [4, 2], dataType: 'int32'},
            constant
          }
        },
        operators: [{name: op, arguments: [
          {input: 'input'}, {axis: 1}, {options: {outputDataType: 'int32'}}
        ], outputs: 'output'}],
        expectedOutputs: {
          output: {
            data: op === 'argMin' ? [0, 1, 0, 1] : [1, 0, 1, 0],
            descriptor: {shape: [4], dataType: 'int32'}
          }
        }
      }
    });
  }
}

// Return only Identity's output. In an affected implementation, Div produced
// -16777217 correctly, but Identity changed it to -16777216. Also returning
// the quotient bypassed the faulty copy path and hid the bug.
// Both operations require exact int32 results.
for (const consumer of ['identity']) {
  const descriptor = {shape: [4], dataType: 'int32'};
  const args = [{input: 'quotient'}];
  fullRangeIntegerTests.push({
    name: `div int32 full range through ${consumer} with hidden producer`,
    graph: {
      inputs: {
        a: {data: [-33554435, 33554435, -2147483648, 2147483647], descriptor},
        b: {data: [2, -2, 1, 1], descriptor}
      },
      operators: [
        {name: 'div', arguments: [{a: 'a'}, {b: 'b'}], outputs: 'quotient'},
        {name: consumer, arguments: args, outputs: 'output'}
      ],
      expectedOutputs: {
        output: {
          data: [-16777217, -16777217, -2147483648, 2147483647],
          descriptor: {shape: [4], dataType: 'int32'}
        }
      }
    }
  });
}
webnn_conformance_test(fullRangeIntegerTests, buildAndExecuteGraph, getZeroULPTolerance);
