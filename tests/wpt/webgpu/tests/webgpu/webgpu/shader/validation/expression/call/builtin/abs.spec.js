/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'abs';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kAllNumericScalarsAndVectors,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kAllNumericScalarsAndVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() never errors
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValuesTypes)).
filter((u) => stageSupportsType(u.stage, kValuesTypes[u.type])).
beginSubcases().
expand('value', (u) => fullRangeForType(kValuesTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = true; // abs() should never error
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kValuesTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

const kTests = {
  valid: {
    src: `_ = abs(1);`,
    pass: true
  },
  alias: {
    src: `_ = abs(i32_alias(1));`,
    pass: true
  },

  bool: {
    src: `_ = abs(false);`,
    pass: false
  },
  vec_bool: {
    src: `_ = abs(vec2<bool>(false, true));`,
    pass: false
  },
  matrix: {
    src: `_ = abs(mat2x2(1, 1, 1, 1));`,
    pass: false
  },
  atomic: {
    src: ` _ = abs(a);`,
    pass: false
  },
  array: {
    src: `var a: array<u32, 5>;
          _ = abs(a);`,
    pass: false
  },
  array_runtime: {
    src: `_ = abs(k.arry);`,
    pass: false
  },
  struct: {
    src: `var a: A;
          _ = abs(a);`,
    pass: false
  },
  enumerant: {
    src: `_ = abs(read_write);`,
    pass: false
  },
  ptr: {
    src: `var<function> a = 1u;
          let p: ptr<function, u32> = &a;
          _ = abs(p);`,
    pass: false
  },
  ptr_deref: {
    src: `var<function> a = 1u;
          let p: ptr<function, u32> = &a;
          _ = abs(*p);`,
    pass: true
  },
  sampler: {
    src: `_ = abs(s);`,
    pass: false
  },
  texture: {
    src: `_ = abs(t);`,
    pass: false
  },
  no_params: {
    src: `_ = abs();`,
    pass: false
  },
  too_many_params: {
    src: `_ = abs(1, 2);`,
    pass: false
  },
  must_use: {
    src: `abs(1);`,
    pass: false
  }
};

g.test('parameters').
desc(`Test that ${builtin} is validated correctly.`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const src = kTests[t.params.test].src;
  const code = `
alias i32_alias = i32;

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