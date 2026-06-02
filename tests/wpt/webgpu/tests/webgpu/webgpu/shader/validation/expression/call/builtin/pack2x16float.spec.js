/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const kFn = 'pack2x16float';export const description = `Validate ${kFn}`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { kValue } from '../../../../../../webgpu/util/constants.js';
import { f32, vec2 } from '../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import { validateConstOrOverrideBuiltinEval } from './const_override_validation.js';

const kArgCases = {
  good: '(vec2f())',
  good_vec2_abstract_float: '(vec2(0.1))',
  bad_0args: '()',
  bad_2args: '(vec2f(),vec2f())',
  bad_abstract_int: '(1)',
  bad_i32: '(1i)',
  bad_f32: '(1f)',
  bad_u32: '(1u)',
  bad_abstract_float: '(0.1)',
  bad_bool: '(false)',
  bad_vec4f: '(vec4f())',
  bad_vec4u: '(vec4u())',
  bad_vec4i: '(vec4i())',
  bad_vec4b: '(vec4<bool>())',
  bad_vec3f: '(vec3f())',
  bad_array: '(array(1.0, 2.0, 3.0, 4.0))',
  bad_struct: '(modf(1.1))'
};
const kGoodArgs = kArgCases['good'];
const kReturnType = 'u32';

export const g = makeTestGroup(ShaderValidationTest);

g.test('args').
desc(`Test compilation failure of ${kFn} with various numbers of and types of arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.expectCompileResult(
    t.params.arg === 'good' || t.params.arg === 'good_vec2_abstract_float',
    `const c = ${kFn}${kArgCases[t.params.arg]};`
  );
});

g.test('return').
desc(`Test ${kFn} return value type`).
params((u) => u.combine('type', ['u32', 'i32', 'f32', 'bool', 'vec2u'])).
fn((t) => {
  t.expectCompileResult(
    t.params.type === kReturnType,
    `const c: ${t.params.type} = ${kFn}${kGoodArgs};`
  );
});

g.test('must_use').
desc(`Result of ${kFn} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${kFn}${kGoodArgs}; }`);
});

g.test('value_range').
desc(
  `Test failures of ${kFn} when at least one of the input value is out of the range of binary16`
).
params((u) =>
u.
combine('constantOrOverrideStage', ['constant', 'override']).
combine('value0', [
kValue.f16.positive.max,
kValue.f16.positive.max + 1,
kValue.f16.negative.min,
kValue.f16.negative.min - 1]
).
combine('value1', [
kValue.f16.positive.max,
kValue.f16.positive.max + 1,
kValue.f16.negative.min,
kValue.f16.negative.min - 1]
)
).
fn((t) => {
  const { constantOrOverrideStage, value0, value1 } = t.params;

  const success =
  value0 >= kValue.f16.negative.min &&
  value0 <= kValue.f16.positive.max &&
  value1 >= kValue.f16.negative.min &&
  value1 <= kValue.f16.positive.max;

  validateConstOrOverrideBuiltinEval(
    t,
    kFn,
    success,
    [vec2(f32(value0), f32(value1))],
    constantOrOverrideStage
  );
});