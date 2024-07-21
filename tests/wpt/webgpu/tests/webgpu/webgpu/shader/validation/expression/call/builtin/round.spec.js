/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'round';export const description = `
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
import { fpTraitsFor } from '../../../../../util/floating_point.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  unique,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kConvertableToFloatScalarsAndVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() inputs rejects invalid values
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValuesTypes)).
filter((u) => stageSupportsType(u.stage, kValuesTypes[u.type])).
beginSubcases().
expand('value', (u) => {
  if (scalarTypeOf(kValuesTypes[u.type]).kind === 'abstract-int') {
    return fullRangeForType(kValuesTypes[u.type]);
  } else {
    const constants = fpTraitsFor(scalarTypeOf(kValuesTypes[u.type])).constants();
    return unique(fullRangeForType(kValuesTypes[u.type]), [
    constants.negative.min + 0.1,
    constants.positive.max - 0.1]
    );
  }
})
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValuesTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = true; // Result should always be representable by the type
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
    [type.create(1)],
    'constant'
  );
});

const kTests =








{
  valid: {
    args: '(1.0f)',
    pass: true
  },
  // Number of arguments.
  no_parens: {
    args: '',
    pass: false
  },
  too_few_args: {
    args: '()',
    pass: false
  },
  too_many_args: {
    args: '(1.f,2.f)',
    pass: false
  },
  // Arguments types (only 1 argument for this builtin).
  alias: {
    args: '(f32_alias(1.f))',
    pass: true
  },
  bool: {
    args: '(false)',
    pass: false
  },
  vec_bool: {
    args: '(vec2<bool>(false,true))',
    pass: false
  },
  matrix: {
    args: '(mat2x2(1.f,1.f,1.f,1.f))',
    pass: false
  },
  atomic: {
    args: '(a)',
    pass: false
  },
  array: {
    preamble: 'var arry: array<f32, 5>;',
    args: '(arry)',
    pass: false
  },
  array_runtime: {
    args: '(k.arry)',
    pass: false
  },
  struct: {
    preamble: 'var x: A;',
    args: '(x)',
    pass: false
  },
  enumerant: {
    args: '(read_write)',
    pass: false
  },
  ptr: {
    preamble: `var<function> f = 1.f;
                     let p: ptr<function, f32> = &f;`,
    args: '(p)',
    pass: false
  },
  ptr_deref: {
    preamble: `var<function> f = 1.f;
                     let p: ptr<function, f32> = &f;`,
    args: '(*p)',
    pass: true
  },
  sampler: {
    args: '(s)',
    pass: false
  },
  texture: {
    args: '(t)',
    pass: false
  }
};

g.test('arguments').
desc(`Test compilation validation of ${builtin} with variously shaped and typed arguments`).
params((u) => u.combine('test', keysOf(kTests))).
fn((t) => {
  const test = kTests[t.params.test];
  t.expectCompileResult(
    test.pass,
    `alias f32_alias = f32;

            @group(0) @binding(0) var s: sampler;
            @group(0) @binding(1) var t: texture_2d<f32>;

            var<workgroup> a: atomic<u32>;

            struct A {
              i: f32,
            }
            struct B {
              arry: array<f32>,
            }
            @group(0) @binding(3) var<storage> k: B;


            @vertex
            fn main() -> @builtin(position) vec4<f32> {
              ${test.preamble ? test.preamble : ''}
              _ = ${builtin}${test.args};
              return vec4<f32>(.4, .2, .3, .1);
            }`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1.0f); }`);
});