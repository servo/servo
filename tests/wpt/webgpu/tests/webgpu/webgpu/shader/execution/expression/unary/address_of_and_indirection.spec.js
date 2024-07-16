/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for unary address-of and indirection (dereference)
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { GPUTest } from '../../../../gpu_test.js';
import { scalarType } from '../../../../util/conversion.js';
import { sparseScalarF32Range } from '../../../../util/math.js';
import {
  allButConstInputSource,
  basicExpressionWithPredeclarationBuilder,
  run } from
'../expression.js';

export const g = makeTestGroup(GPUTest);

// All the ways to deref an expression
const kDerefCases = {
  deref_address_of_identifier: {
    wgsl: '(*(&a))',
    requires_pointer_composite_access: false
  },
  deref_pointer: {
    wgsl: '(*p)',
    requires_pointer_composite_access: false
  },
  address_of_identifier: {
    wgsl: '(&a)',
    requires_pointer_composite_access: true
  },
  pointer: {
    wgsl: 'p',
    requires_pointer_composite_access: true
  }
};

g.test('deref').
specURL('https://www.w3.org/TR/WGSL/#indirection').
desc(
  `
Expression: *e

Pointer expression dereference.
`
).
params((u) =>
u.
combine('inputSource', allButConstInputSource).
combine('vectorize', [undefined, 2, 3, 4]).
combine('scalarType', ['bool', 'u32', 'i32', 'f32', 'f16']).
combine('derefType', keysOf(kDerefCases)).
filter((p) => !kDerefCases[p.derefType].requires_pointer_composite_access)
).
beforeAllSubcases((t) => {
  if (t.params.scalarType === 'f16') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
}).
fn(async (t) => {
  const ty = scalarType(t.params.scalarType);
  const cases = sparseScalarF32Range().map((e) => {
    return { input: ty.create(e), expected: ty.create(e) };
  });
  const elemType = ty.kind;
  const type = t.params.vectorize ? `vec${t.params.vectorize}<${elemType}>` : elemType;
  const shaderBuilder = basicExpressionWithPredeclarationBuilder(
    (value) => `get_dereferenced_value(${value})`,
    `fn get_dereferenced_value(value: ${type}) -> ${type} {
        var a = value;
        let p = &a;
        return ${kDerefCases[t.params.derefType].wgsl};
      }`
  );
  await run(t, shaderBuilder, [ty], ty, t.params, cases);
});

g.test('deref_index').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: (*e)[index]

Pointer expression dereference as lhs of index accessor expression
`
).
params((u) =>
u.
combine('inputSource', allButConstInputSource).
combine('vectorize', [undefined, 2, 3, 4]).
combine('scalarType', ['bool', 'u32', 'i32', 'f32', 'f16']).
combine('derefType', keysOf(kDerefCases))
).
beforeAllSubcases((t) => {
  if (t.params.scalarType === 'f16') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
}).
fn(async (t) => {
  if (
  kDerefCases[t.params.derefType].requires_pointer_composite_access &&
  !t.hasLanguageFeature('pointer_composite_access'))
  {
    return;
  }

  const ty = scalarType(t.params.scalarType);
  const cases = sparseScalarF32Range().map((e) => {
    return { input: ty.create(e), expected: ty.create(e) };
  });
  const elemType = ty.kind;
  const type = t.params.vectorize ? `vec${t.params.vectorize}<${elemType}>` : elemType;
  const shaderBuilder = basicExpressionWithPredeclarationBuilder(
    (value) => `get_dereferenced_value(${value})`,
    `fn get_dereferenced_value(value: ${type}) -> ${type} {
        var a = array<${type}, 1>(value);
        let p = &a;
        return ${kDerefCases[t.params.derefType].wgsl}[0];
      }`
  );
  await run(t, shaderBuilder, [ty], ty, t.params, cases);
});

g.test('deref_member').
specURL('https://www.w3.org/TR/WGSL/#logical-expr').
desc(
  `
Expression: (*e).member

Pointer expression dereference as lhs of member accessor expression
`
).
params((u) =>
u.
combine('inputSource', allButConstInputSource).
combine('vectorize', [undefined, 2, 3, 4]).
combine('scalarType', ['bool', 'u32', 'i32', 'f32', 'f16']).
combine('derefType', keysOf(kDerefCases))
).
beforeAllSubcases((t) => {
  if (t.params.scalarType === 'f16') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
}).
fn(async (t) => {
  if (
  kDerefCases[t.params.derefType].requires_pointer_composite_access &&
  !t.hasLanguageFeature('pointer_composite_access'))
  {
    return;
  }

  const ty = scalarType(t.params.scalarType);
  const cases = sparseScalarF32Range().map((e) => {
    return { input: ty.create(e), expected: ty.create(e) };
  });
  const elemType = ty.kind;
  const type = t.params.vectorize ? `vec${t.params.vectorize}<${elemType}>` : elemType;
  const shaderBuilder = basicExpressionWithPredeclarationBuilder(
    (value) => `get_dereferenced_value(${value})`,
    `struct S {
        m : ${type}
      }
      fn get_dereferenced_value(value: ${type}) -> ${type} {
        var a = S(value);
        let p = &a;
        return ${kDerefCases[t.params.derefType].wgsl}.m;
      }`
  );
  await run(t, shaderBuilder, [ty], ty, t.params, cases);
});