/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validate unpack4xI8`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

const kFeature = 'packed_4x8_integer_dot_product';
const kFn = 'unpack4xI8';
const kGoodArgs = '(1u)';
const kBadArgs = {
  '0args': '()',
  '2args': '(1u,2u)',
  '0i32': '(1i)',
  '0f32': '(1f)',
  '0bool': '(false)',
  '0vec2u': '(vec2u())',
  '0vec3u': '(vec3u())',
  '0vec4u': '(vec4u())'
};

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

g.test('bad_args').
desc(`Test compilation failure of ${kFn} with bad arguments`).
params((u) => u.combine('arg', keysOf(kBadArgs))).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported(kFeature);
  t.expectCompileResult(false, `const c = ${kFn}${kBadArgs[t.params.arg]};`);
});

g.test('must_use').
desc(`Result of ${kFn} must be used`).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported(kFeature);
  t.expectCompileResult(false, `fn f() { ${kFn}${kGoodArgs}; }`);
});