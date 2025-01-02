/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'acos';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConcreteIntegerScalarsAndVectors,
  kConvertableToFloatScalarsAndVectors,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { absBigInt } from '../../../../../util/math.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  minusTwoToTwoRangeForType,
  stageSupportsType,
  unique,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

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
expand('value', (u) =>
unique(
  minusTwoToTwoRangeForType(kValuesTypes[u.type]),
  fullRangeForType(kValuesTypes[u.type])
)
)
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult =
  typeof t.params.value === 'bigint' ?
  absBigInt(t.params.value) <= 1n :
  Math.abs(t.params.value) <= 1;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kValuesTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

// f32 is included here to confirm that validation is failing due to a type issue and not something else.
const kIntegerArgumentTypes = objectsToRecord([Type.f32, ...kConcreteIntegerScalarsAndVectors]);

g.test('integer_argument').
desc(
  `
Validates that scalar and vector integer arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kIntegerArgumentTypes))).
fn((t) => {
  const type = kIntegerArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */type === Type.f32,
    [type.create(0)],
    'constant'
  );
});

const kTests = {
  valid: {
    src: `_ = acos(1);`,
    pass: true
  },
  alias: {
    src: `_ = acos(f32_alias(1));`,
    pass: true
  },

  bool: {
    src: `_ = acos(false);`,
    pass: false
  },
  i32: {
    src: `_ = acos(1i);`,
    pass: false
  },
  u32: {
    src: `_ = acos(1u);`,
    pass: false
  },
  vec_bool: {
    src: `_ = acos(vec2<bool>(false, true));`,
    pass: false
  },
  vec_i32: {
    src: `_ = acos(vec2<i32>(1, 1));`,
    pass: false
  },
  vec_u32: {
    src: `_ = acos(vec2<u32>(1, 1));`,
    pass: false
  },
  matrix: {
    src: `_ = acos(mat2x2(1, 1, 1, 1));`,
    pass: false
  },
  atomic: {
    src: ` _ = acos(a);`,
    pass: false
  },
  array: {
    src: `var a: array<u32, 5>;
          _ = acos(a);`,
    pass: false
  },
  array_runtime: {
    src: `_ = acos(k.arry);`,
    pass: false
  },
  struct: {
    src: `var a: A;
          _ = acos(a);`,
    pass: false
  },
  enumerant: {
    src: `_ = acos(read_write);`,
    pass: false
  },
  ptr: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = acos(p);`,
    pass: false
  },
  ptr_deref: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = acos(*p);`,
    pass: true
  },
  sampler: {
    src: `_ = acos(s);`,
    pass: false
  },
  texture: {
    src: `_ = acos(t);`,
    pass: false
  },
  no_params: {
    src: `_ = acos();`,
    pass: false
  },
  too_many_params: {
    src: `_ = acos(1, 2);`,
    pass: false
  },

  greater_then_one: {
    src: `_ = acos(1.1f);`,
    pass: false
  },
  less_then_negative_one: {
    src: `_ = acos(-1.1f);`,
    pass: false
  },

  must_use: {
    src: `acos(1);`,
    pass: false
  }
};

g.test('parameters').
desc(`Test that ${builtin} is validated correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const src = kTests[t.params.test].src;
  const code = `
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