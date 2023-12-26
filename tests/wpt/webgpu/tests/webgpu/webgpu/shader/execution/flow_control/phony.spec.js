/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for phony assignments.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('phony_assign_call_basic').
desc('Test flow control for a phony assigned with a single function call').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  _ = f();
  ${f.expect_order(2)}
`,
    extra: `
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
`
  }));
});

g.test('phony_assign_call_must_use').
desc(
  'Test flow control for a phony assigned with a single function call annotated with @must_use'
).
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  _ = f();
  ${f.expect_order(2)}
`,
    extra: `
@must_use
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
`
  }));
});

g.test('phony_assign_call_nested').
desc('Test flow control for a phony assigned with nested function calls').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
${f.expect_order(0)}
_ = c(a(), b());
${f.expect_order(4)}
`,
    extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
fn c(x : i32, y : i32) -> i32 {
  ${f.expect_order(3)}
  return x + y;
}
`
  }));
});

g.test('phony_assign_call_nested_must_use').
desc(
  'Test flow control for a phony assigned with nested function calls, all annotated with @must_use'
).
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
${f.expect_order(0)}
_ = c(a(), b());
${f.expect_order(4)}
`,
    extra: `
@must_use
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
@must_use
fn b() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
@must_use
fn c(x : i32, y : i32) -> i32 {
  ${f.expect_order(3)}
  return x + y;
}
`
  }));
});

g.test('phony_assign_call_builtin').
desc(
  'Test flow control for a phony assigned with a builtin call, with two function calls as arguments'
).
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
${f.expect_order(0)}
_ = max(a(), b());
${f.expect_order(3)}
`,
    extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
`
  }));
});