/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for loops.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('loop_break').
desc('Test that flow control exits a loop when reaching a break statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 3, 5, 7)}
    if i == 3 {
      break;
    }
    ${f.expect_order(2, 4, 6)}
    i++;
  }
  ${f.expect_order(8)}
`
  );
});

g.test('loop_continue').
desc('Test flow control for a loop continue statement').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 3, 5, 7, 8)}
    if i == 3 {
      i++;
      continue;
      ${f.expect_not_reached()}
    }
    ${f.expect_order(2, 4, 6, 9)}
    if i == 4 {
      break;
    }
    i++;
  }
  ${f.expect_order(10)}
`
  );
});

g.test('loop_continuing_basic').
desc('Test basic flow control for a loop continuing block').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 3, 5)}
    i++;

    continuing {
      ${f.expect_order(2, 4, 6)}
      break if i == 3;
    }
  }
  ${f.expect_order(7)}
`
  );
});

g.test('nested_loops').
desc('Test flow control for a loop nested in another loop').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) =>
    `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 11, 21)}
    if i == ${f.value(6)} {
      ${f.expect_order(22)}
      break;
      ${f.expect_not_reached()}
    }
    ${f.expect_order(2, 12)}
    loop {
      i++;
      ${f.expect_order(3, 6, 9, 13, 16, 19)}
      if (i % ${f.value(3)}) == 0 {
        ${f.expect_order(10, 20)}
        break;
        ${f.expect_not_reached()}
      }
      ${f.expect_order(4, 7, 14, 17)}
      if (i & ${f.value(1)}) == 0 {
        ${f.expect_order(8, 15)}
        continue;
        ${f.expect_not_reached()}
      }
      ${f.expect_order(5, 18)}
    }
  }
  ${f.expect_order(23)}
`
  );
});

g.test('loop_break_if_logical_and_condition').
desc('Test flow control for a loop with a logical and break if').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 4, 7)}
    continuing {
      i++;
      break if !(a(i) && b(i));
    }
  }
  ${f.expect_order(9)}
      `,
    extra: `
fn a(i : i32) -> bool {
  ${f.expect_order(2, 5, 8)}
  return i < ${f.value(3)};
}
fn b(i : i32) -> bool {
  ${f.expect_order(3, 6)}
  return i < ${f.value(5)};
}
      `
  }));
});

g.test('loop_break_if_logical_or_condition').
desc('Test flow control for a loop with a logical or break if').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  var i = ${f.value(0)};
  loop {
    ${f.expect_order(1, 3, 6, 9)}
    continuing {
      i++;
      break if !(a(i) || b(i));
    }
  }
  ${f.expect_order(12)}
      `,
    extra: `
fn a(i : i32) -> bool {
  ${f.expect_order(2, 4, 7, 10)}
  return i < ${f.value(2)};
}
fn b(i : i32) -> bool {
  ${f.expect_order(5, 8, 11)}
  return i < ${f.value(4)};
}
      `
  }));
});