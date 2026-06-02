/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for pointer_composite_access extension
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

function makeSource(module, init_expr, pointer_read_expr) {
  return `
    ${module}
    fn f() {
        var a = ${init_expr};
        let p = &a;
        let r = ${pointer_read_expr};
    }`;
}

const kCases = {
  // Via identifier 'a'
  array_index_access_via_identifier: {
    module: '',
    init_expr: 'array<i32, 3>()',
    via_deref: '(*(&a))[0]',
    via_pointer: '(&a)[0]'
  },
  vector_index_access_via_identifier: {
    module: '',
    init_expr: 'vec3<i32>()',
    via_deref: '(*(&a))[0]',
    via_pointer: '(&a)[0]'
  },
  vector_member_access_via_identifier: {
    module: '',
    init_expr: 'vec3<i32>()',
    via_deref: '(*(&a)).x',
    via_pointer: '(&a).x'
  },
  matrix_index_access_via_identifier: {
    module: '',
    init_expr: 'mat2x3<f32>()',
    via_deref: '(*(&a))[0]',
    via_pointer: '(&a)[0]'
  },
  struct_member_access_via_identifier: {
    module: 'struct S { a : i32, }',
    init_expr: 'S()',
    via_deref: '(*(&a)).a',
    via_pointer: '(&a).a'
  },
  builtin_struct_modf_via_identifier: {
    module: '',
    init_expr: 'modf(1.5)',
    via_deref: 'vec2((*(&a)).fract, (*(&a)).whole)',
    via_pointer: 'vec2((&a).fract, (&a).whole)'
  },
  builtin_struct_frexp_via_identifier: {
    module: '',
    init_expr: 'frexp(1.5)',
    via_deref: 'vec2((*(&a)).fract, f32((*(&a)).exp))',
    via_pointer: 'vec2((&a).fract, f32((&a).exp))'
  },

  // Via pointer 'p'
  array_index_access_via_pointer: {
    module: '',
    init_expr: 'array<i32, 3>()',
    via_deref: '(*p)[0]',
    via_pointer: 'p[0]'
  },
  vector_index_access_via_pointer: {
    module: '',
    init_expr: 'vec3<i32>()',
    via_deref: '(*p)[0]',
    via_pointer: 'p[0]'
  },
  vector_member_access_via_pointer: {
    module: '',
    init_expr: 'vec3<i32>()',
    via_deref: '(*p).x',
    via_pointer: 'p.x'
  },
  matrix_index_access_via_pointer: {
    module: '',
    init_expr: 'mat2x3<f32>()',
    via_deref: '(*p)[0]',
    via_pointer: 'p[0]'
  },
  struct_member_access_via_pointer: {
    module: 'struct S { a : i32, }',
    init_expr: 'S()',
    via_deref: '(*p).a',
    via_pointer: 'p.a'
  },
  builtin_struct_modf_via_pointer: {
    module: '',
    init_expr: 'modf(1.5)',
    via_deref: 'vec2((*p).fract, (*p).whole)',
    via_pointer: 'vec2(p.fract, p.whole)'
  },
  builtin_struct_frexp_via_pointer: {
    module: '',
    init_expr: 'frexp(1.5)',
    via_deref: 'vec2((*p).fract, f32((*p).exp))',
    via_pointer: 'vec2(p.fract, f32(p.exp))'
  }
};

g.test('deref').
desc('Baseline test: pointer deref is always valid').
params((u) => u.combine('case', keysOf(kCases))).
fn((t) => {
  const curr = kCases[t.params.case];
  const source = makeSource(curr.module, curr.init_expr, curr.via_deref);
  t.expectCompileResult(true, source);
});

g.test('pointer').
desc(
  'Tests that direct pointer access is valid if pointer_composite_access is supported, else it should fail'
).
params((u) => u.combine('case', keysOf(kCases))).
fn((t) => {
  const curr = kCases[t.params.case];
  const source = makeSource(curr.module, curr.init_expr, curr.via_pointer);
  const should_pass = t.hasLanguageFeature('pointer_composite_access');
  t.expectCompileResult(should_pass, source);
});