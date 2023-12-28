/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for return statements.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('return').
desc("Test that flow control does not execute after a 'return' statement").
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  return;
  ${f.expect_not_reached()}
`
  );
});

g.test('return_conditional_true').
desc("Test that flow control does not execute after a 'return' statement in a if (true) block").
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  if (${f.value(true)}) {
    return;
  }
  ${f.expect_not_reached()}
`
  );
});

g.test('return_conditional_false').
desc("Test that flow control does not execute after a 'return' statement in a if (false) block").
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(
    t,
    (f) => `
  ${f.expect_order(0)}
  if (${f.value(false)}) {
    return;
  }
  ${f.expect_order(1)}
`
  );
});