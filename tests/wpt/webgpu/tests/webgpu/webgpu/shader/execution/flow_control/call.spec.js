/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Flow control tests for function calls.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('call_basic').
desc('Test that flow control enters a called function').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  f();
  ${f.expect_order(2)}
`,
    extra: `
fn f() {
  ${f.expect_order(1)}
}`
  }));
});

g.test('call_nested').
desc('Test that flow control enters a nested function calls').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
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
}`
  }));
});

g.test('call_repeated').
desc('Test that flow control enters a nested function calls').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
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
}`
  }));
});

g.test('arg_eval').
desc('Test that arguments are evaluated left to right').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  a(b(), c(), d());
  ${f.expect_order(5)}
`,
    extra: `
fn a(p1 : u32, p2 : u32, p3 : u32) {
  ${f.expect_order(4)}
}
fn b() -> u32 {
  ${f.expect_order(1)}
  return 0;
}
fn c() -> u32 {
  ${f.expect_order(2)}
  return 0;
}
fn d() -> u32 {
  ${f.expect_order(3)}
  return 0;
}`
  }));
});

g.test('arg_eval_logical_and').
desc('Test that arguments are evaluated left to right').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  a(b(${f.value(1)}) && c());
  a(b(${f.value(0)}) && c());
  ${f.expect_order(6)}
`,
    extra: `
fn a(p : bool) {
  ${f.expect_order(3, 5)}
}
fn b(x : i32) -> bool {
  ${f.expect_order(1, 4)}
  return x == 1;
}
fn c() -> bool {
  ${f.expect_order(2)}
  return true;
}`
  }));
});

g.test('arg_eval_logical_or').
desc('Test that arguments are evaluated left to right').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  ${f.expect_order(0)}
  a(b(${f.value(1)}) || c());
  a(b(${f.value(0)}) || c());
  ${f.expect_order(6)}
`,
    extra: `
fn a(p : bool) {
  ${f.expect_order(3, 5)}
}
fn b(x : i32) -> bool {
  ${f.expect_order(1, 4)}
  return x == 0;
}
fn c() -> bool {
  ${f.expect_order(2)}
  return true;
}`
  }));
});

g.test('arg_eval_pointers').
desc('Test that arguments are evaluated left to right').
params((u) => u.combine('preventValueOptimizations', [true, false])).
fn((t) => {
  runFlowControlTest(t, (f) => ({
    entrypoint: `
  var x : i32 = ${f.value(0)};
  ${f.expect_order(0)}
  _ = c(&x);
  a(b(&x), c(&x));
  ${f.expect_order(5)}
`,
    extra: `
fn a(p1 : i32, p2 : i32) {
  ${f.expect_order(4)}
}
fn b(p : ptr<function, i32>) -> i32 {
  (*p)++;
  ${f.expect_order(2)}
  return 0;
}
fn c(p : ptr<function, i32>) -> i32 {
  if (*p == 1) {
    ${f.expect_order(3)}
  } else {
    ${f.expect_order(1)}
  }
  return 0;
}`
  }));
});