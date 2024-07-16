/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'dot';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { Type, kConvertableToFloatVectors, scalarTypeOf } from '../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  ConstantOrOverrideValueChecker,
  fullRangeForType,
  kConstantAndOverrideStages,
  stageSupportsType,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidArgumentTypes = objectsToRecord(kConvertableToFloatVectors);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() never errors
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValidArgumentTypes)).
filter((u) => stageSupportsType(u.stage, kValidArgumentTypes[u.type])).
beginSubcases().
expand('a', (u) => fullRangeForType(kValidArgumentTypes[u.type], 5)).
expand('b', (u) => fullRangeForType(kValidArgumentTypes[u.type], 5))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValidArgumentTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const scalarType = scalarTypeOf(kValidArgumentTypes[t.params.type]);
  const vCheck = new ConstantOrOverrideValueChecker(t, scalarType);

  // Dot product equation: (a[0]*b[0]) + (a[1]*b[1]) + ... (a[N]*b[N])
  // Should be invalid if the dot product calculations result in intermediate
  // values that exceed the maximum representable float value for the given type.
  if (scalarType === Type.abstractInt) {
    // Need to handle the AbstractInt case separately becasue all values are
    // treated as BigInt in that case.
    const a = BigInt(t.params.a);
    const b = BigInt(t.params.b);
    const vecSize = BigInt(kValidArgumentTypes[t.params.type].width);
    const ab = vCheck.checkedResultBigInt(a * b);
    vCheck.checkedResultBigInt(ab * vecSize);
  } else {
    const a = Number(t.params.a);
    const b = Number(t.params.b);
    const vecSize = kValidArgumentTypes[t.params.type].width;
    const ab = vCheck.checkedResult(a * b);
    vCheck.checkedResult(ab * vecSize);
  }

  const type = kValidArgumentTypes[t.params.type];

  // Validates dot(vecN(a), vecN(b));
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    vCheck.allChecksPassed(),
    [type.create(t.params.a), type.create(t.params.b)],
    t.params.stage
  );
});

const kArgCases = {
  good: '(vec3(0), vec3(1))',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_1arg: '(vec3(0))',
  bad_3arg: '(vec3(0), vec3(1), vec3(2))',
  // Mismatched vector size
  bad_vec_size: '(vec2(0), vec3(1))',
  // Bad value for arg 0
  bad_0bool: '(false, vec3(1))',
  bad_0array: '(array(1.1,2.2), vec3(1))',
  bad_0struct: '(modf(2.2), vec3(1))',
  bad_0int: '(0i, vec3(1))',
  bad_0uint: '(0u, vec3(1))',
  bad_0f32: '(0.0, vec3(1))',
  bad_0f16: '(0.0h, vec3(1))',
  bad_0abstract: '(0, vec3(1))',
  // Bad value type for arg 1
  bad_1bool: '(vec3(0), true)',
  bad_1array: '(vec3(0), array(1.1,2.2))',
  bad_1struct: '(vec3(0), modf(2.2))',
  bad_1int: '(vec3(0), 0i)',
  bad_1uint: '(vec3(0), 0u)',
  bad_1f32: '(vec3(0), 0.0)',
  bad_1f16: '(vec3(0), 0.0h)',
  bad_1abstract: '(vec3(0), 0)'
};

g.test('args').
desc(`Test compilation failure of ${builtin} with variously shaped and typed arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.expectCompileResult(
    t.params.arg === 'good',
    `const c = ${builtin}${kArgCases[t.params.arg]};`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}${kArgCases['good']}; }`);
});