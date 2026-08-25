/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'reverseBits';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kConcreteIntegerScalarsAndVectors,
  kFloatScalarsAndVectors } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValuesTypes = objectsToRecord(kConcreteIntegerScalarsAndVectors);

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
fn((t) => {
  const expectedResult = true; // reverseBits() should never error
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [kValuesTypes[t.params.type].create(t.params.value)],
    t.params.stage
  );
});

// u32 is included here to confirm that validation is failing due to a type issue and not something else.
const kFloatTypes = objectsToRecord([Type.u32, ...kFloatScalarsAndVectors]);

g.test('float_argument').
desc(
  `
Validates that float arguments are rejected by ${builtin}()
`
).
params((u) => u.combine('type', keysOf(kFloatTypes))).
fn((t) => {
  const type = kFloatTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    /* expectedResult */type === Type.u32,
    [type.create(0)],
    'constant'
  );
});

const kTests =








{
  valid: {
    args: '(1u)',
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
    args: '(1u,2u)',
    pass: false
  },
  // Arguments types (only 1 argument for this builtin).
  alias: {
    args: '(u32_alias(1))',
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
    args: '(mat2x2(1,1,1,1))',
    pass: false
  },
  atomic: {
    args: '(a)',
    pass: false
  },
  array: {
    preamble: 'var arry: array<u32, 5>;',
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
    preamble: `var<function> f = 1u;
               let p: ptr<function, u32> = &f;`,
    args: '(p)',
    pass: false
  },
  ptr_deref: {
    preamble: `var<function> f = 1u;
               let p: ptr<function, u32> = &f;`,
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
    `alias u32_alias = u32;

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
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}(1u); }`);
});