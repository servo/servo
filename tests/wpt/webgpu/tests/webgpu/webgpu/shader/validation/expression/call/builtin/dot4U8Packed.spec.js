/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validate dot4U8Packed`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

const kFeature = 'packed_4x8_integer_dot_product';
const kFn = 'dot4U8Packed';
const kArgCases = {
  good: '(1u,2u)',
  bad_0args: '()',
  bad_1args: '(1u)',
  bad_3args: '(1u,2u,3u)',
  bad_0i32: '(1i,2u)',
  bad_0f32: '(1f,2u)',
  bad_0bool: '(false,2u)',
  bad_0vec2u: '(vec2u(),2u)',
  bad_1i32: '(1u,2i)',
  bad_1f32: '(1u,2f)',
  bad_1bool: '(1u,true)',
  bad_1vec2u: '(1u,vec2u())',
  bad_bool_bool: '(false,true)',
  bad_bool2_bool2: '(vec2<bool>(),vec2(false,true))',
  bad_0array: '(array(1))',
  bad_0struct: '(modf(1.1))'
};
const kGoodArgs = kArgCases['good'];

export const g = makeTestGroup(ShaderValidationTest);

g.test('unsupported').
desc(`Test absence of ${kFn} when ${kFeature} is not supported.`).
params((u) => u.combine('requires', [false, true])).
fn((t) => {
  t.skipIfLanguageFeatureSupported(kFeature);
  const preamble = t.params.requires ? `requires ${kFeature}; ` : '';
  const code = `${preamble}const c = ${kFn}${kGoodArgs};`;
  t.expectCompileResult(false, code);
});

g.test('supported').
desc(`Test presence of ${kFn} when ${kFeature} is supported.`).
params((u) => u.combine('requires', [false, true])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported(kFeature);
  const preamble = t.params.requires ? `requires ${kFeature}; ` : '';
  const code = `${preamble}const c = ${kFn}${kGoodArgs};`;
  t.expectCompileResult(true, code);
});

g.test('args').
desc(`Test compilation failure of ${kFn} with various numbers of and types of arguments`).
params((u) => u.combine('arg', keysOf(kArgCases))).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported(kFeature);
  t.expectCompileResult(t.params.arg === 'good', `const c = ${kFn}${kArgCases[t.params.arg]};`);
});

g.test('must_use').
desc(`Result of ${kFn} must be used`).
params((u) => u.combine('use', [true, false])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported(kFeature);
  const use_it = t.params.use ? '_ = ' : '';
  t.expectCompileResult(t.params.use, `fn f() { ${use_it}${kFn}${kGoodArgs}; }`);
});