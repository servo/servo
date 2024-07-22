/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for for-loops.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('for_basic').
desc('Test that flow control executes a for-loop body the correct number of times').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(3)}; i++) {
    ${f.expect_order(1, 2, 3)}
  }
  ${f.expect_order(4)}
`
  );
});

g.test('for_break').
desc('Test that flow control exits a for-loop when reaching a break statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(5)}; i++) {
    ${f.expect_order(1, 3, 5, 7)}
    if (i == 3) {
      break;
      ${f.expect_not_reached()}
    }
    ${f.expect_order(2, 4, 6)}
  }
  ${f.expect_order(8)}
`
  );
});

g.test('for_continue').
desc('Test flow control for a for-loop continue statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(5)}; i++) {
    ${f.expect_order(1, 3, 5, 7, 8)}
    if (i == 3) {
      continue;
      ${f.expect_not_reached()}
    }
    ${f.expect_order(2, 4, 6, 9)}
  }
  ${f.expect_order(10)}
`
  );
});

g.test('for_initalizer').
desc('Test flow control for a for-loop initializer').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = initializer(); i < ${f.value(3)}; i++) {
    ${f.expect_order(2, 3, 4)}
  }
  ${f.expect_order(5)}
`,
    extra: `
fn initializer() -> i32 {
  ${f.expect_order(1)}
  return ${f.value(0)};
}
`
  }));
});

g.test('for_complex_initalizer').
desc('Test flow control for a complex for-loop initializer').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = initializer(max(a(), b())); i < ${f.value(5)}; i++) {
    ${f.expect_order(4, 5, 6)}
  }
  ${f.expect_order(7)}
`,
    extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return ${f.value(1)};
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return ${f.value(2)};
}
fn initializer(v : i32) -> i32 {
  ${f.expect_order(3)}
  return v;
}
`
  }));
});

g.test('for_condition').
desc('Test flow control for a for-loop condition').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; condition(i); i++) {
    ${f.expect_order(2, 4, 6)}
  }
  ${f.expect_order(8)}
`,
    extra: `
fn condition(i : i32) -> bool {
  ${f.expect_order(1, 3, 5, 7)}
  return i < ${f.value(3)};
}
`
  }));
});

g.test('for_complex_condition').
desc('Test flow control for a for-loop condition').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; condition(i, a() * b()); i++) {
    ${f.expect_order(4, 8)}
  }
  ${f.expect_order(12)}
`,
    extra: `
fn a() -> i32 {
  ${f.expect_order(1, 5, 9)}
  return ${f.value(1)};
}
fn b() -> i32 {
  ${f.expect_order(2, 6, 10)}
  return ${f.value(2)};
}
fn condition(i : i32, j : i32) -> bool {
  ${f.expect_order(3, 7, 11)}
  return i < j;
}
`
  }));
});

g.test('for_continuing').
desc('Test flow control for a for-loop continuing statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(3)}; i = cont(i)) {
    ${f.expect_order(1, 3, 5)}
  }
  ${f.expect_order(7)}
`,
    extra: `
fn cont(i : i32) -> i32 {
  ${f.expect_order(2, 4, 6)}
  return i + 1;
}
`
  }));
});

g.test('for_complex_continuing').
desc('Test flow control for a for-loop continuing statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(3)}; i += cont(a(), b())) {
    ${f.expect_order(1, 5, 9)}
  }
  ${f.expect_order(13)}
`,
    extra: `
fn a() -> i32 {
  ${f.expect_order(2, 6, 10)}
  return ${f.value(1)};
}
fn b() -> i32 {
  ${f.expect_order(3, 7, 11)}
  return ${f.value(2)};
}
fn cont(i : i32, j : i32) -> i32 {
  ${f.expect_order(4, 8, 12)}
  return j >> u32(i);
}
`
  }));
});

g.test('nested_for_break').
desc('Test flow control for a for-loop break statement in an outer for-loop').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(2)}; i++) {
    ${f.expect_order(1, 5)}
    for (var i = ${f.value(5)}; i < ${f.value(7)}; i++) {
      ${f.expect_order(2, 4, 6, 8)}
      if (i == ${f.value(6)}) {
        break;
        ${f.expect_not_reached()}
      }
      ${f.expect_order(3, 7)}
    }
  }
  ${f.expect_order(9)}
`
  );
});

g.test('nested_for_continue').
desc('Test flow control for a for-loop continue statement in an outer for-loop').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; i < ${f.value(2)}; i++) {
    ${f.expect_order(1, 5)}
    for (var i = ${f.value(5)}; i < ${f.value(7)}; i++) {
      ${f.expect_order(2, 3, 6, 7)}
      if (i == ${f.value(5)}) {
        continue;
        ${f.expect_not_reached()}
      }
      ${f.expect_order(4, 8)}
    }
  }
  ${f.expect_order(9)}
`
  );
});

g.test('for_logical_and_condition').
desc('Test flow control for a for-loop with a logical and condition').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; a(i) && b(i); i++) {
    ${f.expect_order(3, 6)}
  }
  ${f.expect_order(8)}
      `,
    extra: `
fn a(i : i32) -> bool {
  ${f.expect_order(1, 4, 7)}
  return i < ${f.value(2)};
}
fn b(i : i32) -> bool {
  ${f.expect_order(2, 5)}
  return i < ${f.value(5)};
}
      `
  }));
});

g.test('for_logical_or_condition').
desc('Test flow control for a for-loop with a logical or condition').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  for (var i = ${f.value(0)}; a(i) || b(i); i++) {
    ${f.expect_order(2, 4, 7, 10)}
  }
  ${f.expect_order(13)}
      `,
    extra: `
fn a(i : i32) -> bool {
  ${f.expect_order(1, 3, 5, 8, 11)}
  return i < ${f.value(2)};
}
fn b(i : i32) -> bool {
  ${f.expect_order(6, 9, 12)}
  return i < ${f.value(4)};
}
      `
  }));
});