/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for operator precedence.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// Bit set for the binary operator groups.
const kMultiplicative = 1 << 0;
const kAdditive = 1 << 1;
const kShift = 1 << 2;
const kRelational = 1 << 3;
const kBinaryAnd = 1 << 4;
const kBinaryXor = 1 << 5;
const kBinaryOr = 1 << 6;
const kLogical = 1 << 7;

// Set of other operators that each operator can precede without any parentheses.
const kCanPrecedeWithoutParens = {};
kCanPrecedeWithoutParens[kMultiplicative] = kMultiplicative | kAdditive | kRelational;
kCanPrecedeWithoutParens[kAdditive] = kMultiplicative | kAdditive | kRelational;
kCanPrecedeWithoutParens[kShift] = kRelational | kLogical;
kCanPrecedeWithoutParens[kRelational] = kMultiplicative | kAdditive | kShift | kLogical;
kCanPrecedeWithoutParens[kBinaryAnd] = kBinaryAnd;
kCanPrecedeWithoutParens[kBinaryXor] = kBinaryXor;
kCanPrecedeWithoutParens[kBinaryOr] = kBinaryOr;
kCanPrecedeWithoutParens[kLogical] = kRelational;

// The list of binary operators.




const kBinaryOperators = {
  mul: { op: '*', group: kMultiplicative },
  div: { op: '/', group: kMultiplicative },
  mod: { op: '%', group: kMultiplicative },

  add: { op: '+', group: kAdditive },
  sub: { op: '-', group: kAdditive },

  shl: { op: '<<', group: kShift },
  shr: { op: '>>', group: kShift },

  lt: { op: '<', group: kRelational },
  gt: { op: '>', group: kRelational },
  le: { op: '<=', group: kRelational },
  ge: { op: '>=', group: kRelational },
  eq: { op: '==', group: kRelational },
  ne: { op: '!=', group: kRelational },

  bin_and: { op: '&', group: kBinaryAnd },
  bin_xor: { op: '^', group: kBinaryXor },
  bin_or: { op: '|', group: kBinaryOr },

  log_and: { op: '&&', group: kLogical },
  log_or: { op: '||', group: kLogical }
};

g.test('binary_requires_parentheses').
desc(
  `
  Validates that certain binary operators require parentheses to bind correctly.
  `
).
params((u) =>
u.
combine('op1', keysOf(kBinaryOperators)).
combine('op2', keysOf(kBinaryOperators)).
filter((p) => {
  // Skip expressions that would parse as template lists.
  if (p.op1 === 'lt' && ['gt', 'ge', 'shr'].includes(p.op2)) {
    return false;
  }
  // Only combine logical operators with relational operators.
  if (kBinaryOperators[p.op1].group === kLogical) {
    return kBinaryOperators[p.op2].group === kRelational;
  }
  if (kBinaryOperators[p.op2].group === kLogical) {
    return kBinaryOperators[p.op1].group === kRelational;
  }
  return true;
})
).
fn((t) => {
  const op1 = kBinaryOperators[t.params.op1];
  const op2 = kBinaryOperators[t.params.op2];
  const code = `
var<private> a : ${op1.group === kLogical ? 'bool' : 'u32'};
var<private> b : u32;
var<private> c : ${op2.group === kLogical ? 'bool' : 'u32'};
fn foo() {
  let foo = a ${op1.op} b ${op2.op} c;
}
`;

  const valid = (kCanPrecedeWithoutParens[op1.group] & op2.group) !== 0;
  t.expectCompileResult(valid, code);
});

g.test('mixed_logical_requires_parentheses').
desc(
  `
  Validates that mixed logical operators require parentheses to bind correctly.
  `
).
params((u) =>
u.
combine('op1', keysOf(kBinaryOperators)).
combine('op2', keysOf(kBinaryOperators)).
combine('parens', ['none', 'left', 'right']).
filter((p) => {
  const group1 = kBinaryOperators[p.op1].group;
  const group2 = kBinaryOperators[p.op2].group;
  return group1 === kLogical && group2 === kLogical;
})
).
fn((t) => {
  const op1 = kBinaryOperators[t.params.op1];
  const op2 = kBinaryOperators[t.params.op2];
  let expr = `a ${op1.op} b ${op2.op} c;`;
  if (t.params.parens === 'left') {
    expr = `(a ${op1.op} b) ${op2.op} c;`;
  } else if (t.params.parens === 'right') {
    expr = `a ${op1.op} (b ${op2.op} c);`;
  }
  const code = `
var<private> a : bool;
var<private> b : bool;
var<private> c : bool;
fn foo() {
  let bar = ${expr};
}
`;
  const valid = t.params.parens !== 'none' || t.params.op1 === t.params.op2;
  t.expectCompileResult(valid, code);
});

// The list of miscellaneous other test cases.




const kExpressions = {
  neg_member: { expr: '- str . a', result: true },
  comp_member: { expr: '~ str . a', result: true },
  addr_member: { expr: '& str . a', result: true },
  log_and_member: { expr: 'false && str . b', result: true },
  log_or_member: { expr: 'false || str . b', result: true },
  and_addr: { expr: '      v &  &str .a', result: false },
  and_addr_paren: { expr: 'v & (&str).a', result: true },
  deref_member: { expr: '       * ptr_str  . a', result: false },
  deref_member_paren: { expr: '(* ptr_str) . a', result: true },
  deref_idx: { expr: '       * ptr_vec  [0]', result: false },
  deref_idx_paren: { expr: '(* ptr_vec) [1]', result: true }
};

g.test('other').
desc(
  `
    Test that other operator precedence rules are correctly implemented.
    `
).
params((u) => u.combine('expr', keysOf(kExpressions))).
fn((t) => {
  const expr = kExpressions[t.params.expr];
  const wgsl = `
      struct S {
        a: i32,
        b: bool,
      }

      fn main() {
        var v = 42;
        var vec = vec4();
        var str = S(42, false);
        let ptr_vec = &vec;
        let ptr_str = &str;

        let foo = ${expr.expr};
      }
    `;

  t.expectCompileResult(expr.result, wgsl);
});

const kLHSExpressions = {
  deref_invalid1: { expr: `*p.b`, result: false },
  deref_invalid2: { expr: `*p.a[0]`, result: false },
  deref_valid1: { expr: `(*p).b`, result: true },
  deref_valid2: { expr: `(*p).a[2]`, result: true },
  addr_valid1: { expr: `*&v.b`, result: true },
  addr_valid2: { expr: `(*&v).b`, result: true },
  addr_valid3: { expr: `*&(v.b)`, result: true }
};

g.test('other_lhs').
desc('Test precedence of * and [] in LHS').
params((u) => u.combine('expr', keysOf(kLHSExpressions))).
fn((t) => {
  const expr = kLHSExpressions[t.params.expr];
  const code = `
    struct S {
      a : array<i32, 4>,
      b : i32,
    }
    fn main() {
      var v : S;
      let p = &v;
      let q = &v.a;

      ${expr.expr} = 1i;
    }`;

  t.expectCompileResult(expr.result, code);
});