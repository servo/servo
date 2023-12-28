/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for switch statements.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('switch').
desc('Test that flow control executes the correct switch case block').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  switch (${f.value(1)}) {
    case 0: {
      ${f.expect_not_reached()}
      break;
    }
    case 1: {
      ${f.expect_order(1)}
      break;
    }
    case 2: {
      ${f.expect_not_reached()}
      break;
    }
    default: {
      ${f.expect_not_reached()}
      break;
    }
  }
  ${f.expect_order(2)}
`
  );
});

g.test('switch_multiple_case').
desc(
  'Test that flow control executes the correct switch case block with multiple cases per block'
).
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  switch (${f.value(2)}) {
    case 0, 1: {
      ${f.expect_not_reached()}
      break;
    }
    case 2, 3: {
      ${f.expect_order(1)}
      break;
    }
    default: {
      ${f.expect_not_reached()}
      break;
    }
  }
  ${f.expect_order(2)}
`
  );
});

g.test('switch_multiple_case_default').
desc(
  'Test that flow control executes the correct switch case block with multiple cases per block (combined with default)'
).
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  switch (${f.value(2)}) {
    case 0, 1: {
      ${f.expect_not_reached()}
      break;
    }
    case 2, 3, default: {
      ${f.expect_order(1)}
      break;
    }
  }
  ${f.expect_order(2)}
  switch (${f.value(1)}) {
    case 0, 1: {
      ${f.expect_order(3)}
      break;
    }
    case 2, 3, default: {
      ${f.expect_not_reached()}
      break;
    }
  }
  ${f.expect_order(4)}
`
  );
});

g.test('switch_default').
desc('Test that flow control executes the switch default block').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
${f.expect_order(0)}
switch (${f.value(4)}) {
  case 0: {
    ${f.expect_not_reached()}
    break;
  }
  case 1: {
    ${f.expect_not_reached()}
    break;
  }
  case 2: {
    ${f.expect_not_reached()}
    break;
  }
  default: {
    ${f.expect_order(1)}
    break;
  }
}
${f.expect_order(2)}
`
  );
});

g.test('switch_default_only').
desc('Test that flow control executes the switch default block, which is the only case').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
${f.expect_order(0)}
switch (${f.value(4)}) {
default: {
  ${f.expect_order(1)}
  break;
}
}
${f.expect_order(2)}
`
  );
});