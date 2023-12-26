/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for function alias analysis`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);






const kUses = {
  no_access: { is_write: false, gen: (ref) => `{ let p = &*&${ref}; }` },
  assign: { is_write: true, gen: (ref) => `${ref} = 42;` },
  compound_assign_lhs: { is_write: true, gen: (ref) => `${ref} += 1;` },
  compound_assign_rhs: { is_write: false, gen: (ref) => `{ var tmp : i32; tmp += ${ref}; }` },
  increment: { is_write: true, gen: (ref) => `${ref}++;` },
  binary_lhs: { is_write: false, gen: (ref) => `_ = ${ref} + 1;` },
  binary_rhs: { is_write: false, gen: (ref) => `_ = 1 + ${ref};` },
  unary_minus: { is_write: false, gen: (ref) => `_ = -${ref};` },
  bitcast: { is_write: false, gen: (ref) => `_ = bitcast<f32>(${ref});` },
  convert: { is_write: false, gen: (ref) => `_ = f32(${ref});` },
  builtin_arg: { is_write: false, gen: (ref) => `_ = abs(${ref});` },
  index_access: { is_write: false, gen: (ref) => `{ var arr : array<i32, 4>; _ = arr[${ref}]; }` },
  let_init: { is_write: false, gen: (ref) => `{ let tmp = ${ref}; }` },
  var_init: { is_write: false, gen: (ref) => `{ var tmp = ${ref}; }` },
  return: { is_write: false, gen: (ref) => `{ return ${ref}; }` },
  switch_cond: { is_write: false, gen: (ref) => `switch(${ref}) { default { break; } }` }
};



function shouldPass(aliased, ...uses) {
  // Expect fail if the pointers are aliased and at least one of the accesses is a write.
  // If either of the accesses is a "no access" then expect pass.
  return !aliased || !uses.some((u) => kUses[u].is_write) || uses.includes('no_access');
}

g.test('two_pointers').
desc(`Test aliasing of two pointers passed to a function.`).
params((u) =>
u.
combine('address_space', ['private', 'function']).
combine('a_use', keysOf(kUses)).
combine('b_use', keysOf(kUses)).
combine('aliased', [true, false]).
beginSubcases()
).
fn((t) => {
  const code = `
${t.params.address_space === 'private' ? `var<private> x : i32; var<private> y : i32;` : ``}

fn callee(pa : ptr<${t.params.address_space}, i32>,
          pb : ptr<${t.params.address_space}, i32>) -> i32 {
  ${kUses[t.params.a_use].gen(`*pa`)}
  ${kUses[t.params.b_use].gen(`*pb`)}
  return 0;
}

fn caller() {
  ${t.params.address_space === 'function' ? `var x : i32; var y : i32;` : ``}
  callee(&x, ${t.params.aliased ? `&x` : `&y`});
}
`;
  t.expectCompileResult(shouldPass(t.params.aliased, t.params.a_use, t.params.b_use), code);
});

g.test('one_pointer_one_module_scope').
desc(`Test aliasing of a pointer with a direct access to a module-scope variable.`).
params((u) =>
u.
combine('a_use', keysOf(kUses)).
combine('b_use', keysOf(kUses)).
combine('aliased', [true, false]).
beginSubcases()
).
fn((t) => {
  const code = `
var<private> x : i32;
var<private> y : i32;

fn callee(pb : ptr<private, i32>) -> i32 {
  ${kUses[t.params.a_use].gen(`x`)}
  ${kUses[t.params.b_use].gen(`*pb`)}
  return 0;
}

fn caller() {
  callee(${t.params.aliased ? `&x` : `&y`});
}
`;
  t.expectCompileResult(shouldPass(t.params.aliased, t.params.a_use, t.params.b_use), code);
});

g.test('subcalls').
desc(`Test aliasing of two pointers passed to a function, and then passed to other functions.`).
params((u) =>
u.
combine('a_use', ['no_access', 'assign', 'binary_lhs']).
combine('b_use', ['no_access', 'assign', 'binary_lhs']).
combine('aliased', [true, false]).
beginSubcases()
).
fn((t) => {
  const code = `
var<private> x : i32;
var<private> y : i32;

fn subcall_no_access(p : ptr<private, i32>) {
  let pp = &*p;
}

fn subcall_binary_lhs(p : ptr<private, i32>) -> i32 {
  return *p + 1;
}

fn subcall_assign(p : ptr<private, i32>) {
  *p = 42;
}

fn callee(pa : ptr<private, i32>, pb : ptr<private, i32>) -> i32 {
  let new_pa = &*pa;
  let new_pb = &*pb;
  subcall_${t.params.a_use}(new_pa);
  subcall_${t.params.b_use}(new_pb);
  return 0;
}

fn caller() {
  callee(&x, ${t.params.aliased ? `&x` : `&y`});
}
`;
  t.expectCompileResult(shouldPass(t.params.aliased, t.params.a_use, t.params.b_use), code);
});

g.test('member_accessors').
desc(`Test aliasing of two pointers passed to a function and used with member accessors.`).
params((u) =>
u.
combine('a_use', ['no_access', 'assign', 'binary_lhs']).
combine('b_use', ['no_access', 'assign', 'binary_lhs']).
combine('aliased', [true, false]).
beginSubcases()
).
fn((t) => {
  const code = `
struct S { a : i32 }

var<private> x : S;
var<private> y : S;

fn callee(pa : ptr<private, S>,
          pb : ptr<private, S>) -> i32 {
  ${kUses[t.params.a_use].gen(`(*pa).a`)}
  ${kUses[t.params.b_use].gen(`(*pb).a`)}
  return 0;
}

fn caller() {
  callee(&x, ${t.params.aliased ? `&x` : `&y`});
}
`;
  t.expectCompileResult(shouldPass(t.params.aliased, t.params.a_use, t.params.b_use), code);
});

g.test('same_pointer_read_and_write').
desc(`Test that we can read from and write to the same pointer.`).
params((u) => u.beginSubcases()).
fn((t) => {
  const code = `
var<private> v : i32;

fn callee(p : ptr<private, i32>) {
  *p = *p + 1;
}

fn caller() {
  callee(&v);
}
`;
  t.expectCompileResult(true, code);
});

g.test('aliasing_inside_function').
desc(`Test that we can alias pointers inside a function.`).
params((u) => u.beginSubcases()).
fn((t) => {
  const code = `
var<private> v : i32;

fn foo() {
  var v : i32;
  let p1 = &v;
  let p2 = &v;
  *p1 = 42;
  *p2 = 42;
}
`;
  t.expectCompileResult(true, code);
});