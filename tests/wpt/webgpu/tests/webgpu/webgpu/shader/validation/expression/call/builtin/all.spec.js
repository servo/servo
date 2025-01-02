/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'all';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { Type, elementTypeOf, kAllScalarsAndVectors } from '../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import { validateConstOrOverrideBuiltinEval } from './const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kArgumentTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('argument_types').
desc(
  `
Validates that scalar and vector arguments are rejected by ${builtin}() if not bool or vecN<bool>
`
).
params((u) => u.combine('type', keysOf(kArgumentTypes))).
fn((t) => {
  const type = kArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */elementTypeOf(type) === Type.bool,
    [type.create(0)],
    'constant',
    /* returnType */Type.bool
  );
});

const kTests = {
  valid: {
    src: `_ = ${builtin}(true);`,
    pass: true
  },
  alias: {
    src: `_ = ${builtin}(bool_alias(true));`,
    pass: true
  },
  bool: {
    src: `_ = ${builtin}(false);`,
    pass: true
  },
  i32: {
    src: `_ = ${builtin}(1i);`,
    pass: false
  },
  u32: {
    src: `_ = ${builtin}(1u);`,
    pass: false
  },
  f32: {
    src: `_ = ${builtin}(1.0f);`,
    pass: false
  },
  f16: {
    src: `_ = ${builtin}(1.0h);`,
    pass: false
  },
  vec_bool: {
    src: `_ = ${builtin}(vec2<bool>(false, true));`,
    pass: true
  },
  vec2_bool_implicit: {
    src: `_ = ${builtin}(vec2(false, true));`,
    pass: true
  },
  vec3_bool_implicit: {
    src: `_ = ${builtin}(vec3(true));`,
    pass: true
  },
  vec_i32: {
    src: `_ = ${builtin}(vec2<i32>(1, 1));`,
    pass: false
  },
  vec_u32: {
    src: `_ = ${builtin}(vec2<u32>(1, 1));`,
    pass: false
  },
  vec_f32: {
    src: `_ = ${builtin}(vec2<f32>(1, 1));`,
    pass: false
  },
  vec_f16: {
    src: `_ = ${builtin}(vec2<f16>(1, 1));`,
    pass: false
  },
  matrix: {
    src: `_ = ${builtin}(mat2x2(1, 1, 1, 1));`,
    pass: false
  },
  atomic: {
    src: ` _ = ${builtin}(a);`,
    pass: false
  },
  array: {
    src: `var a: array<bool, 5>;
            _ = ${builtin}(a);`,
    pass: false
  },
  array_runtime: {
    src: `_ = ${builtin}(k.arry);`,
    pass: false
  },
  struct: {
    src: `var a: A;
            _ = ${builtin}(a);`,
    pass: false
  },
  enumerant: {
    src: `_ = ${builtin}(read_write);`,
    pass: false
  },
  ptr: {
    src: `var<function> a = true;
            let p: ptr<function, bool> = &a;
            _ = ${builtin}(p);`,
    pass: false
  },
  ptr_deref: {
    src: `var<function> a = true;
            let p: ptr<function, bool> = &a;
            _ = ${builtin}(*p);`,
    pass: true
  },
  sampler: {
    src: `_ = ${builtin}(s);`,
    pass: false
  },
  texture: {
    src: `_ = ${builtin}(t);`,
    pass: false
  },
  no_args: {
    src: `_ = ${builtin}();`,
    pass: false
  },
  too_many_args: {
    src: `_ = ${builtin}(true, true);`,
    pass: false
  }
};

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(true); }`);
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