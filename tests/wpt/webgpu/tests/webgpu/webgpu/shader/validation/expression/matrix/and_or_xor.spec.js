/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for matrix binary ops expressions.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// A list of operators
const kOperators = {
  and: { op: '&' },
  or: { op: '|' },
  xor: { op: '^' }
};








const kTests = {
  bool: {
    src: 'false'
  },
  vec: {
    src: 'vec2f()'
  },
  i32: {
    src: '1i'
  },
  u32: {
    src: '1u'
  },
  ai: {
    src: '1'
  },
  f32: {
    src: '1f'
  },
  f16: {
    src: '1h',
    is_f16: true
  },
  af: {
    src: '1.0'
  },
  texture: {
    src: 't'
  },
  sampler: {
    src: 's'
  },
  atomic: {
    src: 'a'
  },
  struct: {
    src: 'str'
  },
  array: {
    src: 'arr'
  },
  matf_matching: {
    src: 'mat2x3f()'
  },
  matf_no_match: {
    src: 'mat4x4f()'
  },
  math: {
    src: 'mat2x3h()',
    is_f16: true
  }
};

g.test('invalid').
desc(`Validates that binary operator expressions are never accepted for matrix types.`).
params((u) =>
u.
combine('op', keysOf(kOperators))
// 1i is the control that the test passes
.combine('rhs', ['1i', 'ai', 'mat2x3f()', 'mat2x3h()']).
combine('test', keysOf(kTests)).
combine('swap', [true, false])
).
beforeAllSubcases((t) => {
  if (kTests[t.params.test].is_f16 === true || t.params.rhs.startsWith('mat2x3h(')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  let lhs = kTests[t.params.test].src;
  let rhs = t.params.rhs === 'ai' ? 'mat2x3(0, 0, 0, 0, 0, 0)' : t.params.rhs;

  if (t.params.swap) {
    const a = lhs;
    lhs = rhs;
    rhs = a;
  }

  const code = `
${kTests[t.params.test].is_f16 || t.params.rhs.startsWith('mat2x3h(') ? 'enable f16;' : ''}
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<i32>;

struct S { u : u32 }

var<private> arr : array<i32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${lhs} ${kOperators[t.params.op].op} ${rhs};
}
`;

  const pass = (lhs === '1i' || lhs === '1') && (rhs === '1i' || rhs === '1');
  t.expectCompileResult(pass, code);
});