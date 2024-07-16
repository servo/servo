/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'clamp';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConvertableToFloatScalarsAndVectors,
  kConcreteIntegerScalarsAndVectors,
  scalarTypeOf,
  isConvertible } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord([
...kConvertableToFloatScalarsAndVectors,
...kConcreteIntegerScalarsAndVectors]
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
expand('e', (u) => fullRangeForType(kValuesTypes[u.type], 3)).
expand('low', (u) => fullRangeForType(kValuesTypes[u.type], 4)).
expand('high', (u) => fullRangeForType(kValuesTypes[u.type], 4))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kValuesTypes[t.params.type];
  const expectedResult = t.params.low <= t.params.high;
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.e), type.create(t.params.low), type.create(t.params.high)],
    t.params.stage
  );
});

g.test('mismatched').
desc(
  `
Validates that even with valid types, if types do not match, ${builtin}() errors
`
).
params((u) =>
u.
combine('e', keysOf(kValuesTypes)).
beginSubcases().
combine('low', keysOf(kValuesTypes)).
combine('high', keysOf(kValuesTypes))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.e]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const e = kValuesTypes[t.params.e];
  const low = kValuesTypes[t.params.low];
  const high = kValuesTypes[t.params.high];

  // Skip if shader-16 isn't available.
  t.skipIf(scalarTypeOf(low) === Type.f16 || scalarTypeOf(high) === Type.f16);

  // If there exists 1 type of the 3 args that the other 2 can be converted into, then the args
  // are valid.
  const expectedResult =
  isConvertible(low, e) && isConvertible(high, e) ||
  isConvertible(e, low) && isConvertible(high, low) ||
  isConvertible(e, high) && isConvertible(low, high);
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [e.create(1), low.create(0), high.create(2)],
    'constant'
  );
});

const kStages = ['constant', 'override', 'runtime'];

g.test('low_high').
desc(
  `
Validates that low <= high.
`
).
params((u) =>
u.
combine('type', keysOf(kValuesTypes)).
combine('lowStage', kStages).
combine('highStage', kStages).
beginSubcases().
combineWithParams([
{ low: 0, high: 1 },
{ low: 1, high: 1 },
{ low: 1, high: 0 }]
).
filter((t) => {
  // Avoid abstracts since the runtime value will force concretization.
  const ty = kValuesTypes[t.type];
  const scalar = scalarTypeOf(ty);
  return scalar !== Type.abstractInt && scalar !== Type.abstractFloat;
})
).
beforeAllSubcases((t) => {
  const ty = kValuesTypes[t.params.type];
  if (ty.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const ty = kValuesTypes[t.params.type];
  const scalar = scalarTypeOf(ty);
  let low_arg = '';
  let high_arg = '';
  switch (t.params.lowStage) {
    case 'constant':
      low_arg = `${ty.create(t.params.low).wgsl()}`;
      break;
    case 'override':
      low_arg = `${ty.toString()}(o_low)`;
      break;
    case 'runtime':
      low_arg = 'v_low';
      break;
  }
  switch (t.params.highStage) {
    case 'constant':
      high_arg = `${ty.create(t.params.high).wgsl()}`;
      break;
    case 'override':
      high_arg = `${ty.toString()}(o_high)`;
      break;
    case 'runtime':
      high_arg = 'v_high';
      break;
  }
  const enable = `${ty.requiresF16() ? 'enable f16;' : ''}`;
  const wgsl = `
${enable}
override o_low : ${scalar};
override o_high : ${scalar};
fn foo() {
  var v_low : ${t.params.type};
  var v_high : ${t.params.type};
  var v : ${t.params.type};
  let tmp = clamp(v, ${low_arg}, ${high_arg});
}`;
  const error = t.params.low > t.params.high;
  const shader_error =
  error && t.params.lowStage === 'constant' && t.params.highStage === 'constant';
  const pipeline_error =
  error && t.params.lowStage !== 'runtime' && t.params.highStage !== 'runtime';
  t.expectCompileResult(!shader_error, wgsl);
  if (!shader_error) {
    const constants = {};
    constants['o_low'] = t.params.low;
    constants['o_high'] = t.params.high;
    t.expectPipelineResult({
      expectedResult: !pipeline_error,
      code: wgsl,
      constants,
      reference: ['o_low', 'o_high']
    });
  }
});

g.test('low_high_abstract').
desc('Values low <= high for abstracts').
params((u) =>
u.
combine('type', ['abstract-int', 'abstract-float']).
beginSubcases().
combineWithParams([
{ low: 0, high: 1 },
{ low: 1, high: 1 },
{ low: 1, high: 0 }]
)
).
fn((t) => {
  const ty = kValuesTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */t.params.low <= t.params.high,
    [ty.create(1), ty.create(t.params.low), ty.create(t.params.high)],
    'constant'
  );
});










function typesToArguments(types, pass) {
  return types.reduce(
    (res, type) => ({
      ...res,
      [type.toString()]: { arg: type.create(0).wgsl(), pass }
    }),
    {}
  );
}

// f32 is included here to confirm that validation is failing due to a type issue and not something else.
const kInputArgTypes = {
  ...typesToArguments([Type.f32], true),
  ...typesToArguments([Type.bool, Type.mat2x2f], false),
  alias: { arg: 'f32_alias(1.f)', pass: true },
  vec_bool: { arg: 'vec2<bool>(false,true)', pass: false },
  atomic: { arg: 'a', pass: false },
  array: {
    preamble: 'var arry: array<f32, 5>;',
    arg: 'arry',
    pass: false
  },
  array_runtime: { arg: 'k.arry', pass: false },
  struct: {
    preamble: 'var x: A;',
    arg: 'x',
    pass: false
  },
  enumerant: { arg: 'read_write', pass: false },
  ptr: {
    preamble: `var<function> f = 1.f;
               let p: ptr<function, f32> = &f;`,
    arg: 'p',
    pass: false
  },
  ptr_deref: {
    preamble: `var<function> f = 1.f;
               let p: ptr<function, f32> = &f;`,
    arg: '*p',
    pass: true
  },
  sampler: { arg: 's', pass: false },
  texture: { arg: 't', pass: false }
};

g.test('arguments').
desc(
  `
Test compilation validation of ${builtin} with variously typed arguments
  - Note that this passes the same type for all args. Mismatching types are tested separately above.
`
).
params((u) => u.combine('type', keysOf(kInputArgTypes))).
fn((t) => {
  const type = kInputArgTypes[t.params.type];
  t.expectCompileResult(
    type.pass,
    `alias f32_alias = f32;

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
        ${type.preamble ? type.preamble : ''}
        _ = ${builtin}(${type.arg},${type.arg},${type.arg});
        return vec4<f32>(.4, .2, .3, .1);
      }`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1.f,0.f,1.f); }`);
});