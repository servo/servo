/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'select';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  concreteTypeOf,
  isConvertible,
  kAllScalarsAndVectors,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import { validateConstOrOverrideBuiltinEval } from './const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kArgumentTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('argument_types_1_and_2').
desc(
  `
Validates that scalar and vector arguments are not rejected by ${builtin}() for args 1 and 2
`
).
params((u) => u.combine('type1', keysOf(kArgumentTypes)).combine('type2', keysOf(kArgumentTypes))).
beforeAllSubcases((t) => {
  if (
  scalarTypeOf(kArgumentTypes[t.params.type1]) === Type.f16 ||
  scalarTypeOf(kArgumentTypes[t.params.type2]) === Type.f16)
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type1 = kArgumentTypes[t.params.type1];
  const type2 = kArgumentTypes[t.params.type2];
  // First and second arg must be the same or one convertible to the other.
  // Note that we specify a concrete return type even if both args are abstract.
  const returnType = isConvertible(type1, type2) ?
  concreteTypeOf(type2) :
  isConvertible(type2, type1) ?
  concreteTypeOf(type1) :
  undefined;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */returnType !== undefined,
    [type1.create(0), type2.create(0), Type.bool.create(0)],
    'constant',
    returnType
  );
});

g.test('argument_types_3').
desc(
  `
Validates that third argument must be bool for ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kArgumentTypes))).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kArgumentTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */type === Type.bool,
    [Type.i32.create(0), Type.i32.create(0), type.create(0)],
    'constant',
    /*return_type*/Type.i32
  );
});

const kTests = {
  valid: {
    src: `_ = ${builtin}(1, 2, true);`,
    pass: true
  },
  alias: {
    src: `_ = ${builtin}(i32_alias(1), i32_alias(2), bool_alias(true));`,
    pass: true
  },
  bool: {
    src: `_ = ${builtin}(false, false, true);`,
    pass: true
  },
  i32: {
    src: `_ = ${builtin}(1i, 1i, true);`,
    pass: true
  },
  u32: {
    src: `_ = ${builtin}(1u, 1u, true);`,
    pass: true
  },
  f32: {
    src: `_ = ${builtin}(1.0f, 1.0f, true);`,
    pass: true
  },
  f16: {
    src: `_ = ${builtin}(1.0h, 1.0h, true);`,
    pass: true
  },
  mixed_aint_afloat: {
    src: `_ = ${builtin}(1, 1.0, true);`,
    pass: true
  },
  mixed_i32_u32: {
    src: `_ = ${builtin}(1i, 1u, true);`,
    pass: false
  },
  vec_bool: {
    src: `_ = ${builtin}(vec2<bool>(false, true), vec2<bool>(false, true), true);`,
    pass: true
  },
  vec2_bool_implicit: {
    src: `_ = ${builtin}(vec2(false, true), vec2(false, true), true);`,
    pass: true
  },
  vec3_bool_implicit: {
    src: `_ = ${builtin}(vec3(false), vec3(true), true);`,
    pass: true
  },
  vec_i32: {
    src: `_ = ${builtin}(vec2<i32>(1, 1), vec2<i32>(1, 1), true);`,
    pass: true
  },
  vec_u32: {
    src: `_ = ${builtin}(vec2<u32>(1, 1), vec2<u32>(1, 1), true);`,
    pass: true
  },
  vec_f32: {
    src: `_ = ${builtin}(vec2<f32>(1, 1), vec2<f32>(1, 1), true);`,
    pass: true
  },
  vec_f16: {
    src: `_ = ${builtin}(vec2<f16>(1, 1), vec2<f16>(1, 1), true);`,
    pass: true
  },
  matrix: {
    src: `_ = ${builtin}(mat2x2(1, 1, 1, 1), mat2x2(1, 1, 1, 1), true);`,
    pass: false
  },
  atomic: {
    src: ` _ = ${builtin}(a, a, true);`,
    pass: false
  },
  array: {
    src: `var a: array<bool, 5>;
            _ = ${builtin}(a, a, true);`,
    pass: false
  },
  array_runtime: {
    src: `_ = ${builtin}(k.arry, k.arry, true);`,
    pass: false
  },
  struct: {
    src: `var a: A;
            _ = ${builtin}(a, a, true);`,
    pass: false
  },
  enumerant: {
    src: `_ = ${builtin}(read_write, read_write, true);`,
    pass: false
  },
  ptr: {
    src: `var<function> a = true;
            let p: ptr<function, bool> = &a;
            _ = ${builtin}(p, p, true);`,
    pass: false
  },
  ptr_deref: {
    src: `var<function> a = true;
            let p: ptr<function, bool> = &a;
            _ = ${builtin}(*p, *p, true);`,
    pass: true
  },
  sampler: {
    src: `_ = ${builtin}(s, s, true);`,
    pass: false
  },
  texture: {
    src: `_ = ${builtin}(t, t, true);`,
    pass: false
  },
  no_args: {
    src: `_ = ${builtin}();`,
    pass: false
  },
  too_few_args: {
    src: `_ = ${builtin}(1, true);`,
    pass: false
  },
  too_many_args: {
    src: `_ = ${builtin}(1, 1, 1, true);`,
    pass: false
  }
};

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1, 2, true); }`);
});

g.test('arguments').
desc(`Test that ${builtin} is validated correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
beforeAllSubcases((t) => {
  if (t.params.test.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const src = kTests[t.params.test].src;
  const enables = t.params.test.includes('f16') ? 'enable f16;' : '';
  const code = `
  ${enables}
  alias bool_alias = bool;
  alias i32_alias = i32;

  @group(0) @binding(0) var s: sampler;
  @group(0) @binding(1) var t: texture_2d<f32>;

  var<workgroup> a: atomic<u32>;

  struct A {
    i: bool,
  }
  struct B {
    arry: array<u32>,
  }
  @group(0) @binding(3) var<storage> k: B;

  @vertex
  fn main() -> @builtin(position) vec4<f32> {
    ${src}
    return vec4<f32>(.4, .2, .3, .1);
  }`;
  t.expectCompileResult(kTests[t.params.test].pass, code);
});