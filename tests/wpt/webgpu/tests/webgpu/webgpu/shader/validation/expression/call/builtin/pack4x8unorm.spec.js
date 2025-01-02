/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const kFn = 'pack4x8unorm';export const description = `Validate ${kFn}`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

const kArgCases = {
  good: '(vec4f())',
  good_vec4_abstract_float: '(vec4(0.1))',
  bad_0args: '()',
  bad_2args: '(vec4f(),vec4f())',
  bad_abstract_int: '(1)',
  bad_i32: '(1i)',
  bad_f32: '(1f)',
  bad_u32: '(1u)',
  bad_abstract_float: '(0.1)',
  bad_bool: '(false)',
  bad_vec4u: '(vec4u())',
  bad_vec4i: '(vec4i())',
  bad_vec4b: '(vec4<bool>())',
  bad_vec2f: '(vec2f())',
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
    t.params.arg === 'good' || t.params.arg === 'good_vec4_abstract_float',
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