/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'atan2';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  VectorValue,
  kFloatScalarsAndVectors,
  kConcreteIntegerScalarsAndVectors,
  kAllMatrices,
  kAllBoolScalarsAndVectors,
  scalarTypeOf,
  Type } from
'../../../../../util/conversion.js';
import { isRepresentable } from '../../../../../util/floating_point.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  sparseMinusThreePiToThreePiRangeForType,
  stageSupportsType,
  unique,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kFloatScalarsAndVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() rejects invalid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValuesTypes)).
filter((u) => stageSupportsType(u.stage, kValuesTypes[u.type])).
beginSubcases().
expand('y', (u) =>
unique(
  sparseMinusThreePiToThreePiRangeForType(kValuesTypes[u.type]),
  fullRangeForType(kValuesTypes[u.type], 4)
)
).
expand('x', (u) =>
unique(
  sparseMinusThreePiToThreePiRangeForType(kValuesTypes[u.type]),
  fullRangeForType(kValuesTypes[u.type], 4)
)
)
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kValuesTypes[t.params.type];
  const expectedResult = isRepresentable(
    Math.abs(Math.atan2(Number(t.params.x), Number(t.params.y))),
    scalarTypeOf(type)
  );
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.y), type.create(t.params.x)],
    t.params.stage
  );
});

const kInvalidArgumentTypes = objectsToRecord([
Type.f32,
...kConcreteIntegerScalarsAndVectors,
...kAllBoolScalarsAndVectors,
...kAllMatrices]
);

g.test('invalid_argument_y').
desc(
  `
Validates that scalar and vector integer arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kInvalidArgumentTypes))).
fn((t) => {
  const yTy = kInvalidArgumentTypes[t.params.type];
  const xTy = yTy instanceof VectorValue ? Type.vec(yTy.size, Type.f32) : Type.f32;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */yTy === Type.f32,
    [yTy.create(1), xTy.create(1)],
    'constant'
  );
});

g.test('invalid_argument_x').
desc(
  `
Validates that scalar and vector integer arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kInvalidArgumentTypes))).
fn((t) => {
  const xTy = kInvalidArgumentTypes[t.params.type];
  const yTy = xTy instanceof VectorValue ? Type.vec(xTy.size, Type.f32) : Type.f32;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */xTy === Type.f32,
    [yTy.create(1), xTy.create(1)],
    'constant'
  );
});

const kTests = {
  af: {
    src: `_ = atan2(1.2, 2.2);`,
    pass: true,
    is_f16: false
  },
  ai: {
    src: `_ = atan2(1, 2);`,
    pass: true,
    is_f16: false
  },
  ai_af: {
    src: `_ = atan2(1, 2.1);`,
    pass: true,
    is_f16: false
  },
  af_ai: {
    src: `_ = atan2(1.2, 2);`,
    pass: true,
    is_f16: false
  },
  ai_f32: {
    src: `_ = atan2(1, 1.2f);`,
    pass: true,
    is_f16: false
  },
  f32_ai: {
    src: `_ = atan2(1.2f, 1);`,
    pass: true,
    is_f16: false
  },
  af_f32: {
    src: `_ = atan2(1.2, 2.2f);`,
    pass: true,
    is_f16: false
  },
  f32_af: {
    src: `_ = atan2(2.2f, 1.2);`,
    pass: true,
    is_f16: false
  },
  f16_ai: {
    src: `_ = atan2(1.2h, 1);`,
    pass: true,
    is_f16: true
  },
  ai_f16: {
    src: `_ = atan2(1, 1.2h);`,
    pass: true,
    is_f16: true
  },
  af_f16: {
    src: `_ = atan2(1.2, 1.2h);`,
    pass: true,
    is_f16: true
  },
  f16_af: {
    src: `_ = atan2(1.2h, 1.2);`,
    pass: true,
    is_f16: true
  },

  mixed_types: {
    src: `_ = atan2(1.2f, vec2(1.2f));`,
    pass: false,
    is_f16: false
  },
  mixed_types_2: {
    src: `_ = atan2(vec2(1.2f), 1.2f);`,
    pass: false,
    is_f16: false
  },
  f16_f32: {
    src: `_ = atan2(1.2h, 1.2f);`,
    pass: false,
    is_f16: true
  },
  u32_f32: {
    src: `_ = atan2(1u, 1.2f);`,
    pass: false,
    is_f16: false
  },
  f32_u32: {
    src: `_ = atan2(1.2f, 1u);`,
    pass: false,
    is_f16: false
  },
  f32_i32: {
    src: `_ = atan2(1.2f, 1i);`,
    pass: false,
    is_f16: false
  },
  i32_f32: {
    src: `_ = atan2(1i, 1.2f);`,
    pass: false,
    is_f16: false
  },
  f32_bool: {
    src: `_ = atan2(1.2f, true);`,
    pass: false,
    is_f16: false
  },
  bool_f32: {
    src: `_ = atan2(false, 1.2f);`,
    pass: false,
    is_f16: false
  },
  vec_f32: {
    src: `_ = atan2(vec2(1i), vec2(1.2f));`,
    pass: false,
    is_f16: false
  },
  f32_vec: {
    src: `_ = atan2(vec2(1.2f), vec2(1i));`,
    pass: false,
    is_f16: false
  },
  matrix: {
    src: `_ = atan2(mat2x2(1, 1, 1, 1), mat2x2(1, 1, 1, 1));`,
    pass: false,
    is_f16: false
  },
  atomic: {
    src: ` _ = atan2(a, a);`,
    pass: false,
    is_f16: false
  },
  array: {
    src: `var a: array<u32, 5>;
          _ = atan2(a, a);`,
    pass: false,
    is_f16: false
  },
  array_runtime: {
    src: `_ = atan2(k.arry, k.arry);`,
    pass: false,
    is_f16: false
  },
  struct: {
    src: `var a: A;
          _ = atan2(a, a);`,
    pass: false,
    is_f16: false
  },
  enumerant: {
    src: `_ = atan2(read_write, read_write);`,
    pass: false,
    is_f16: false
  },
  ptr: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = atan2(p, p);`,
    pass: false,
    is_f16: false
  },
  ptr_deref: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = atan2(*p, *p);`,
    pass: true,
    is_f16: false
  },
  sampler: {
    src: `_ = atan2(s, s);`,
    pass: false,
    is_f16: false
  },
  texture: {
    src: `_ = atan2(t, t);`,
    pass: false,
    is_f16: false
  },
  no_params: {
    src: `_ = atan2();`,
    pass: false,
    is_f16: false
  },
  too_many_params: {
    src: `_ = atan2(1, 2, 3);`,
    pass: false,
    is_f16: false
  },

  must_use: {
    src: `atan2(1, 2);`,
    pass: false,
    is_f16: false
  }
};

g.test('parameters').
desc(`Test that ${builtin} is validated correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
beforeAllSubcases((t) => {
  if (kTests[t.params.test].is_f16 === true) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const src = kTests[t.params.test].src;
  const code = `
${kTests[t.params.test].is_f16 ? 'enable f16;' : ''}
alias f32_alias = f32;

@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: texture_2d<f32>;

var<workgroup> a: atomic<u32>;

struct A {
  i: u32,
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

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1, 2); }`);
});