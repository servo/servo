/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'transpose';export const description = `
Validation tests for the ${builtin}() builtin.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  isConvertible,
  kAllMatrices,
  kConcreteFloatScalars,
  kFloatScalars,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  fullRangeForType,
  kConstantAndOverrideStages,
  validateConstOrOverrideBuiltinEval } from
'./const_override_validation.js';

export const g = makeTestGroup(ShaderValidationTest);

const kValidArgumentTypes = objectsToRecord(kAllMatrices);

g.test('values').
desc(
  `
Validates that constant evaluation and override evaluation of ${builtin}() accept valid inputs.
`
).
params((u) =>
u.
combine('stage', kConstantAndOverrideStages).
combine('type', keysOf(kValidArgumentTypes)).
beginSubcases().
expand('value', (u) => fullRangeForType(kValidArgumentTypes[u.type]))
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(kValidArgumentTypes[t.params.type]) === Type.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const expectedResult = true;

  const type = kValidArgumentTypes[t.params.type];
  validateConstOrOverrideBuiltinEval(
    t,
    builtin,
    expectedResult,
    [type.create(t.params.value)],
    t.params.stage
  );
});

const kArgCases = {
  good: '(mat2x2(0, 1, 2, 3))',
  bad_no_parens: '',
  // Bad number of args
  bad_0args: '()',
  bad_2arg: '(1.2, 2.3)',
  // Bad value for arg 0
  bad_0bool: '(false)',
  bad_0array: '(array(1.1,2.2))',
  bad_0struct: '(modf(2.2))',
  bad_0uint: '(1u)',
  bad_0int: '(1i)',
  bad_0vec2i: '(vec2i())',
  bad_0vec2u: '(vec2u())',
  bad_0vec3i: '(vec3i())',
  bad_0vec3u: '(vec3u())',
  bad_0vec4i: '(vec4i())',
  bad_0vec4u: '(vec4u())'
};

g.test('args').
desc(`Test compilation failure of ${builtin} with variously typed arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.expectCompileResult(
    t.params.arg === 'good',
    `const c = ${builtin}${kArgCases[t.params.arg]};`
  );
});

const kValidArgumentScalarTypes = objectsToRecord(kFloatScalars);
const kValidReturnScalarTypes = objectsToRecord(kConcreteFloatScalars);

g.test('return').
desc(`Test compilation pass/failure of ${builtin} with variously shaped inputs and outputs`).
params((u) =>
u.
combine('input_type', keysOf(kValidArgumentScalarTypes)).
combine('input_rows', [2, 3, 4]).
combine('input_cols', [2, 3, 4]).
combine('output_type', keysOf(kValidReturnScalarTypes)).
combine('output_rows', [2, 3, 4]).
combine('output_cols', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.input_type === 'f16' || t.params.output_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const input_type = t.params.input_type;
  const input_cols = t.params.input_cols;
  const input_rows = t.params.input_rows;
  const input_values = Array(input_cols * input_rows).
  fill(kValidArgumentScalarTypes[t.params.input_type].create(0).wgsl()).
  join(', ');
  const input_str = `mat${input_cols}x${input_rows}(${input_values})`;

  const output_type = t.params.output_type;
  const output_cols = t.params.output_cols;
  const output_rows = t.params.output_rows;

  const enables = input_type === 'f16' || output_type === 'f16' ? 'enable f16;' : '';

  const expectedResult =
  input_cols === output_rows &&
  input_rows === output_cols &&
  isConvertible(kValidArgumentScalarTypes[input_type], kValidReturnScalarTypes[output_type]);
  t.expectCompileResult(
    expectedResult,
    `${enables}\nconst c: mat${output_cols}x${output_rows}<${output_type}> = ${builtin}(${input_str});`
  );
});

g.test('must_use').
desc(`Result of ${builtin} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${builtin}${kArgCases['good']}; }`);
});