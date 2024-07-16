/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'asinh';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  kConcreteIntegerScalarsAndVectors,
  kConvertableToFloatScalarsAndVectors,
  scalarTypeOf,
  Type } from
'../../../../../util/conversion.js';
import { isRepresentable } from '../../../../../util/floating_point.js';
import { linearRange, linearRangeBigInt } from '../../../../../util/math.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  rangeForType,
  stageSupportsType,
  unique,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

const additionalRangeForType = rangeForType(
  linearRange(-2000, 2000, 10),
  linearRangeBigInt(-2000n, 2000n, 10)
);

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
unique(fullRangeForType(kValuesTypes[u.type]), additionalRangeForType(kValuesTypes[u.type]))
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
    Math.asinh(Number(t.params.value)),
    // AbstractInt is converted to AbstractFloat before calling into the builtin
    scalarTypeOf(type).kind === 'abstract-int' ? Type.abstractFloat : scalarTypeOf(type)
  );
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.value)],
    t.params.stage
  );
});

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
    [type.create(1)],
    'constant'
  );
});

const kTests = {
  valid: {
    src: `_ = asinh(1);`,
    pass: true
  },
  alias: {
    src: `_ = asinh(f32_alias(1));`,
    pass: true
  },

  bool: {
    src: `_ = asinh(false);`,
    pass: false
  },
  i32: {
    src: `_ = asinh(1i);`,
    pass: false
  },
  u32: {
    src: `_ = asinh(1u);`,
    pass: false
  },
  vec_bool: {
    src: `_ = asinh(vec2<bool>(false, true));`,
    pass: false
  },
  vec_i32: {
    src: `_ = asinh(vec2<i32>(1, 1));`,
    pass: false
  },
  vec_u32: {
    src: `_ = asinh(vec2<u32>(1, 1));`,
    pass: false
  },
  matrix: {
    src: `_ = asinh(mat2x2(1, 1, 1, 1));`,
    pass: false
  },
  atomic: {
    src: ` _ = asinh(a);`,
    pass: false
  },
  array: {
    src: `var a: array<u32, 5>;
          _ = asinh(a);`,
    pass: false
  },
  array_runtime: {
    src: `_ = asinh(k.arry);`,
    pass: false
  },
  struct: {
    src: `var a: A;
          _ = asinh(a);`,
    pass: false
  },
  enumerant: {
    src: `_ = asinh(read_write);`,
    pass: false
  },
  ptr: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = asinh(p);`,
    pass: false
  },
  ptr_deref: {
    src: `var<function> a = 1f;
          let p: ptr<function, f32> = &a;
          _ = asinh(*p);`,
    pass: true
  },
  sampler: {
    src: `_ = asinh(s);`,
    pass: false
  },
  texture: {
    src: `_ = asinh(t);`,
    pass: false
  },
  no_params: {
    src: `_ = asinh();`,
    pass: false
  },
  too_many_params: {
    src: `_ = asinh(1, 2);`,
    pass: false
  },

  must_use: {
    src: `asinh(1);`,
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