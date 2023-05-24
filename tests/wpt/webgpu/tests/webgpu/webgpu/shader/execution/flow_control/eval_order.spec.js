/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Flow control tests for expression evaluation order.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import { runFlowControlTest } from './harness.js';

export const g = makeTestGroup(GPUTest);

g.test('binary_op')
  .desc('Test that a binary operator evaluates the LHS then the RHS')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = lhs() + rhs();
  ${f.expect_order(3)}
`,
      extra: `
fn lhs() -> i32 {
  ${f.expect_order(1)}
  return 0;
}
fn rhs() -> i32 {
  ${f.expect_order(2)}
  return 0;
}`,
    }));
  });

g.test('binary_op_rhs_const')
  .desc('Test that a binary operator evaluates the LHS, when the RHS is a constant expression')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = lhs() + rhs();
  ${f.expect_order(2)}
`,
      extra: `
fn lhs() -> i32 {
  ${f.expect_order(1)}
  return 0;
}
fn rhs() -> i32 {
  return 0;
}`,
    }));
  });

g.test('binary_op_lhs_const')
  .desc('Test that a binary operator evaluates the RHS, when the LHS is a constant expression')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = lhs() + rhs();
  ${f.expect_order(2)}
`,
      extra: `
fn lhs() -> i32 {
  return 0;
}
fn rhs() -> i32 {
  ${f.expect_order(1)}
  return 0;
}`,
    }));
  });

g.test('binary_op_chain')
  .desc('Test that a binary operator chain evaluates left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = a() + b() - c() * d();
  ${f.expect_order(5)}
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
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 1;
}`,
    }));
  });

g.test('binary_op_chain_R_C_C_C')
  .desc(
    'Test evaluation order of a binary operator chain with a runtime-expression for the left-most expression'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = f() + 1 + 2 + 3;
  ${f.expect_order(2)}
`,
      extra: `
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
`,
    }));
  });

g.test('binary_op_chain_C_R_C_C')
  .desc(
    'Test evaluation order of a binary operator chain with a runtime-expression for the second-left-most-const'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = 1 + f() + 2 + 3;
  ${f.expect_order(2)}
  `,
      extra: `
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
  `,
    }));
  });

g.test('binary_op_chain_C_C_R_C')
  .desc(
    'Test evaluation order of a binary operator chain with a runtime-expression for the second-right-most-const'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = 1 + 2 + f() + 3;
  ${f.expect_order(2)}
`,
      extra: `
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
  `,
    }));
  });

g.test('binary_op_chain_C_C_C_R')
  .desc(
    'Test evaluation order of a binary operator chain with a runtime-expression for the right-most expression'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
    ${f.expect_order(0)}
    let l = 1 + 2 + 3 + f();
    ${f.expect_order(2)}
  `,
      extra: `
fn f() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
  `,
    }));
  });

g.test('binary_op_parenthesized_expr')
  .desc('Test that a parenthesized binary operator expression evaluates left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let x = (a() + b()) - (c() * d());
  ${f.expect_order(5)}
  let y = a() + (b() - c()) * d();
  ${f.expect_order(10)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1, 6)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2, 7)}
  return 1;
}
fn c() -> i32 {
  ${f.expect_order(3, 8)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4, 9)}
  return 1;
}`,
    }));
  });

g.test('array_index')
  .desc('Test that array indices are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<array<i32, 8>, 8>, 8>;
  ${f.expect_order(0)}
  let x = arr[a()][b()][c()];
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
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}`,
    }));
  });

g.test('array_index_lhs_assignment')
  .desc(
    'Test that array indices are evaluated left-to-right, when indexing the LHS of an assignment'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<array<i32, 8>, 8>, 8>;
  ${f.expect_order(0)}
  arr[a()][b()][c()] = ~d();
  ${f.expect_order(5)}
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
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 1;
}`,
    }));
  });

g.test('array_index_lhs_member_assignment')
  .desc(
    'Test that array indices are evaluated left-to-right, when indexing with member-accessors in the LHS of an assignment'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<S, 8>, 8>;
  ${f.expect_order(0)}
  arr[a()][b()].member[c()] = d();
  ${f.expect_order(5)}
`,
      extra: `
struct S {
  member : array<i32, 8>,
}
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 1;
}`,
    }));
  });

g.test('array_index_via_ptrs')
  .desc('Test that array indices are evaluated in order, when used via pointers')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<array<i32, 8>, 8>, 8>;
  ${f.expect_order(0)}
  let p0 = &arr;
  ${f.expect_order(1)}
  let p1 = &(*p0)[a()];
  ${f.expect_order(3)}
  let p2 = &(*p1)[b()];
  ${f.expect_order(5)}
  let p3 = &(*p2)[c()];
  ${f.expect_order(7)}
  let p4 = *p3;
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(4)}
  return 1;
}
fn c() -> i32 {
  ${f.expect_order(6)}
  return 1;
}`,
    }));
  });

g.test('array_index_via_struct_members')
  .desc('Test that array indices are evaluated in order, when accessed via structure members')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var x : X;
  ${f.expect_order(0)}
  let r = x.y[a()].z[b()].a[c()];
  ${f.expect_order(4)}
`,
      extra: `
struct X {
  y : array<Y, 3>,
};
struct Y {
  z : array<Z, 3>,
};
struct Z {
  a : array<i32, 3>,
};
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}`,
    }));
  });

g.test('matrix_index')
  .desc('Test that matrix indices are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var mat : mat4x4<f32>;
  ${f.expect_order(0)}
  let x = mat[a()][b()];
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
}`,
    }));
  });

g.test('matrix_index_via_ptr')
  .desc('Test that matrix indices are evaluated in order, when used via pointers')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var mat : mat4x4<f32>;
  ${f.expect_order(0)}
  let p0 = &mat;
  ${f.expect_order(1)}
  let p1 = &(*p0)[a()];
  ${f.expect_order(3)}
  let v = (*p1)[b()];
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(2)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(4)}
  return 1;
}`,
    }));
  });

g.test('logical_and')
  .desc(
    'Test that a chain of logical-AND expressions are evaluated left-to-right, stopping at the first false'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = a() && b() && c();
  ${f.expect_order(3)}
`,
      extra: `
fn a() -> bool {
  ${f.expect_order(1)}
  return true;
}
fn b() -> bool {
  ${f.expect_order(2)}
  return false;
}
fn c() -> bool {
  ${f.expect_not_reached()}
  return true;
}
`,
    }));
  });

g.test('logical_or')
  .desc(
    'Test that a chain of logical-OR expressions are evaluated left-to-right, stopping at the first true'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = a() || b() || c();
  ${f.expect_order(3)}
`,
      extra: `
fn a() -> bool {
  ${f.expect_order(1)}
  return false;
}
fn b() -> bool {
  ${f.expect_order(2)}
  return true;
}
fn c() -> bool {
  ${f.expect_not_reached()}
  return true;
}
`,
    }));
  });

g.test('bitwise_and')
  .desc(
    'Test that a chain of bitwise-AND expressions are evaluated left-to-right, with no short-circuiting'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = a() & b() & c();
  ${f.expect_order(4)}
`,
      extra: `
fn a() -> bool {
  ${f.expect_order(1)}
  return true;
}
fn b() -> bool {
  ${f.expect_order(2)}
  return false;
}
fn c() -> bool {
  ${f.expect_order(3)}
  return true;
}
`,
    }));
  });

g.test('bitwise_or')
  .desc(
    'Test that a chain of bitwise-OR expressions are evaluated left-to-right, with no short-circuiting'
  )
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = a() | b() | c();
  ${f.expect_order(4)}
`,
      extra: `
fn a() -> bool {
  ${f.expect_order(1)}
  return false;
}
fn b() -> bool {
  ${f.expect_order(2)}
  return true;
}
fn c() -> bool {
  ${f.expect_order(3)}
  return true;
}
`,
    }));
  });

g.test('user_fn_args')
  .desc('Test user function call arguments are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = f(a(), b(), c());
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 3;
}
fn f(x : i32, y : i32, z : i32) -> i32 {
  ${f.expect_order(4)}
  return x + y + z;
}`,
    }));
  });

g.test('nested_fn_args')
  .desc('Test user nested call arguments are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = g(c(a(), b()), f(d(), e()));
  ${f.expect_order(8)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 0;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 0;
}
fn c(x : i32, y : i32) -> i32 {
  ${f.expect_order(3)}
  return x + y;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 0;
}
fn e() -> i32 {
  ${f.expect_order(5)}
  return 0;
}
fn f(x : i32, y : i32) -> i32 {
  ${f.expect_order(6)}
  return x + y;
}
fn g(x : i32, y : i32) -> i32 {
  ${f.expect_order(7)}
  return x + y;
}`,
    }));
  });

g.test('builtin_fn_args')
  .desc('Test builtin function call arguments are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = mix(a(), b(), c());
  ${f.expect_order(4)}
`,
      extra: `
fn a() -> f32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> f32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> f32 {
  ${f.expect_order(3)}
  return 3;
}
`,
    }));
  });

g.test('nested_builtin_fn_args')
  .desc('Test nested builtin function call arguments are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let l = mix(a(), mix(b(), c(), d()), e());
  ${f.expect_order(6)}
`,
      extra: `
fn a() -> f32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> f32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> f32 {
  ${f.expect_order(3)}
  return 3;
}
fn d() -> f32 {
  ${f.expect_order(4)}
  return 3;
}
fn e() -> f32 {
  ${f.expect_order(5)}
  return 3;
}
`,
    }));
  });

g.test('1d_array_constructor')
  .desc('Test arguments of an array constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = array(a(), b(), c(), d());
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('2d_array_constructor')
  .desc('Test arguments of a 2D array constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = array(array(a(), b()), array(c(), d()));
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('vec4_constructor')
  .desc('Test arguments of a vector constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = vec4(a(), b(), c(), d());
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('nested_vec4_constructor')
  .desc('Test arguments of a nested vector constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = vec4(a(), vec2(b(), c()), d());
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('struct_constructor')
  .desc('Test arguments of a structure constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = S(a(), b(), c(), d());
  ${f.expect_order(5)}
`,
      extra: `
struct S {
  a : i32,
  b : i32,
  c : i32,
  d : i32,
}
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('nested_struct_constructor')
  .desc('Test arguments of a nested structure constructor are evaluated left-to-right')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  ${f.expect_order(0)}
  let v = Y(a(), X(b(), c()), d());
  ${f.expect_order(5)}
`,
      extra: `
struct Y {
  a : i32,
  x : X,
  c : i32,
}
struct X {
  b : i32,
  c : i32,
}
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('1d_array_assignment')
  .desc('Test LHS of an array element assignment is evaluated before RHS')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<i32, 8>;
  ${f.expect_order(0)}
  arr[a()] = arr[b()];
  ${f.expect_order(3)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
`,
    }));
  });

g.test('2d_array_assignment')
  .desc('Test LHS of 2D-array element assignment is evaluated before RHS')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<i32, 8>, 8>;
  ${f.expect_order(0)}
  arr[a()][b()] = arr[c()][d()];
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('1d_array_compound_assignment')
  .desc('Test LHS of an array element compound assignment is evaluated before RHS')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<i32, 8>;
  ${f.expect_order(0)}
  arr[a()] += arr[b()];
  ${f.expect_order(3)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
`,
    }));
  });

g.test('2d_array_compound_assignment')
  .desc('Test LHS of a 2D-array element compound assignment is evaluated before RHS')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<i32, 8>, 8>;
  ${f.expect_order(0)}
  arr[a()][b()] += arr[c()][d()];
  ${f.expect_order(5)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
fn b() -> i32 {
  ${f.expect_order(2)}
  return 2;
}
fn c() -> i32 {
  ${f.expect_order(3)}
  return 1;
}
fn d() -> i32 {
  ${f.expect_order(4)}
  return 2;
}
`,
    }));
  });

g.test('1d_array_increment')
  .desc('Test index of an array element increment is evaluated only once')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<i32, 8>;
  ${f.expect_order(0)}
  arr[a()]++;
  ${f.expect_order(2)}
`,
      extra: `
fn a() -> i32 {
  ${f.expect_order(1)}
  return 1;
}
`,
    }));
  });

g.test('2d_array_increment')
  .desc('Test index of a 2D-array element increment is evaluated only once')
  .fn(t => {
    runFlowControlTest(t, f => ({
      entrypoint: `
  var arr : array<array<i32, 8>, 8>;
  ${f.expect_order(0)}
  arr[a()][b()]++;
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
`,
    }));
  });
