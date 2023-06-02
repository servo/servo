/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Flow control tests for function calls.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('call_basic')
  .desc('Test that flow control enters a called function')
  .params(u => u.combine('preventValueOptimizations', [true, false]))
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  f();
  ${f.expect_order(2)}
`,
      extra: `
fn f() {
  ${f.expect_order(1)}
}`,
    }));
  });

g.test('call_nested')
  .desc('Test that flow control enters a nested function calls')
  .params(u => u.combine('preventValueOptimizations', [true, false]))
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  a();
  ${f.expect_order(6)}
`,
      extra: `
fn a() {
  ${f.expect_order(1)}
  b();
  ${f.expect_order(5)}
}
fn b() {
  ${f.expect_order(2)}
  c();
  ${f.expect_order(4)}
}
fn c() {
  ${f.expect_order(3)}
}`,
    }));
  });

g.test('call_repeated')
  .desc('Test that flow control enters a nested function calls')
  .params(u => u.combine('preventValueOptimizations', [true, false]))
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  a();
  ${f.expect_order(10)}
`,
      extra: `
fn a() {
  ${f.expect_order(1)}
  b();
  ${f.expect_order(5)}
  b();
  ${f.expect_order(9)}
}
fn b() {
  ${f.expect_order(2, 6)}
  c();
  ${f.expect_order(4, 8)}
}
fn c() {
  ${f.expect_order(3, 7)}
}`,
    }));
  });
